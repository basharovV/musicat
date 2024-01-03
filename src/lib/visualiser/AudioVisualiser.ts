import { get } from "svelte/store";
import { volume } from "../../data/store";
import audioPlayer from "../AudioPlayer";

export interface IAnimation {
    draw: (
        audioBufferData: Uint8Array,
        canvasElement: CanvasRenderingContext2D
    ) => void;
}

/**
 * Visualiser for playing audio using Canvas
 */
export class AudioVisualiser {
    audioElement: HTMLAudioElement;
    audioElement2: HTMLAudioElement;
    canvas: HTMLCanvasElement;

    private _audioContext: AudioContext;

    // Nodes
    private _audioSource: MediaElementAudioSourceNode;
    private _audioSource2: MediaElementAudioSourceNode;
    private _audioAnalyser: AnalyserNode;
    private _gainNode: GainNode;
    private _gainNode2: GainNode;

    private _canvasContext: CanvasRenderingContext2D;
    private _activeAnimations: IAnimation[] = [];

    timeDomain: Uint8Array;
    freqDomain: Uint8Array;

    constructor(audioElement: HTMLAudioElement, canvas: HTMLCanvasElement) {
        this.audioElement = audioPlayer.getCurrentAudioFile();
        this.audioElement2 = audioPlayer.getOtherAudioFile();
        this.canvas = canvas;
        this._canvasContext = this.canvas.getContext("2d");
        this.setupAnalyserAudio();
        this.setupAnalyserAnimation();

        audioPlayer.connectOtherAudioFile = (
            toConnect,
            transitionPointInTime
        ) => {
            this.setAudioElement(toConnect, transitionPointInTime);
        };

        audioPlayer.doPlay = (number) => {
            if (number === 1) {
                this._audioSource.mediaElement.play();
            } else {
                this._audioSource2.mediaElement.play();
            }
        };

        audioPlayer.doPause = (number) => {
            if (number === 1) {
                this._audioSource.mediaElement.pause();
            } else {
                this._audioSource2.mediaElement.pause();
            }
        };

        volume.subscribe((vol) => {
            this._gainNode.gain.value = vol;
            this._gainNode2.gain.value = vol;
            localStorage.setItem("volume", String(vol));
        });
    }

    setCanvas(canvas: HTMLCanvasElement) {
        this.canvas = canvas;
        this._canvasContext = canvas.getContext("2d");
    }

    setAudioElement(toConnect, transitionPointInTime) {
        // console.log('Connecting other audio file', transitionPointInTime);
        // if (toConnect === 1) {
        //     setTimeout(() => {
        //         this._gainNode.gain.value = get(volume);
        //         this._gainNode2.gain.value = 0;
        //     }, transitionPointInTime);
        // } else {
        //     setTimeout(() => {
        //         this._gainNode2.gain.value = get(volume);
        //         this._gainNode.gain.value = 0;
        //     }, transitionPointInTime);
        // }
    }

    /**
     * Draw an oscilloscope
     */
    drawOscilloscope() {
        const step = this._canvasContext.canvas.width / this.timeDomain.length;

        this._canvasContext.beginPath();
        // drawing loop (skipping every second record)
        for (let i = 0; i < this.timeDomain.length; i += 2) {
            const percent = this.timeDomain[i] / 256;
            const x = i * step;
            const y = this._canvasContext.canvas.height * percent;
            this._canvasContext.shadowColor = "#14D8BD";
            this._canvasContext.shadowBlur = 10;
            this._canvasContext.shadowOffsetX = 2;
            this._canvasContext.shadowOffsetY = 2;
            this._canvasContext.strokeStyle = "#14D8BD";
            this._canvasContext.lineTo(x, y);
        }

        this._canvasContext.stroke();
    }

    /**
     * Connect the nodes together, so sound is flowing
     * into the analyser node from the source node
     */
    setupAnalyserAudio() {
        let AudioContext = window.AudioContext || window.webkitAudioContext;
        this._audioContext = new AudioContext({sampleRate: 48000});
        this._audioSource = this._audioContext.createMediaElementSource(
            this.audioElement
        );

        this._audioSource2 = this._audioContext.createMediaElementSource(
            this.audioElement2
        );
        this._audioAnalyser = new AnalyserNode(this._audioContext, {
            fftSize: 2048,
            smoothingTimeConstant: 0.9
        });
        this._gainNode = new GainNode(this._audioContext, {
            gain: get(volume)
        });
        this._gainNode2 = new GainNode(this._audioContext, {
            gain: get(volume)
        });
        this._audioSource.mediaElement.volume = get(volume);
        this._audioSource2.mediaElement.volume = get(volume);
        // Source -> Gain -> Analyser -> Destination (master)
        this._audioSource.connect(this._gainNode);
        this._audioSource2.connect(this._gainNode2);
        this._gainNode.connect(this._audioAnalyser);
        this._gainNode2.connect(this._audioAnalyser);
        this._audioAnalyser.connect(this._audioContext.destination);
    }

    setupAnalyserAnimation() {
        this.freqDomain = new Uint8Array(this._audioAnalyser.frequencyBinCount);
        this.timeDomain = new Uint8Array(this._audioAnalyser.fftSize);

        let tick = () => {
            this._audioAnalyser.getByteFrequencyData(this.freqDomain);
            this._audioAnalyser.getByteTimeDomainData(this.timeDomain);
            this._canvasContext.clearRect(
                0,
                0,
                this._canvasContext.canvas.clientWidth,
                this._canvasContext.canvas.clientHeight
            );
            this.drawOscilloscope();
            window.requestAnimationFrame(tick);
        };
        tick();
    }

    public addAnimation(animation: IAnimation): void {
        this._activeAnimations.push(animation);
    }

    public clearAnimations(): void {
        this._activeAnimations = [];
    }
}
