import { get } from "svelte/store";
import { volume } from "../../data/store";
import audioPlayer from "./WebAudioPlayer";
import { currentThemeObject } from "../../theming/store";

export interface IAnimation {
    draw: (
        audioBufferData: Uint8Array,
        canvasElement: CanvasRenderingContext2D
    ) => void;
}

/**
 * Visualiser for playing audio using Canvas
 */
export class WebAudioVisualiser {
    audioElement: HTMLAudioElement;
    audioElement2: HTMLAudioElement;
    canvas: HTMLCanvasElement;

    private _canvasContext: CanvasRenderingContext2D;
    private _activeAnimations: IAnimation[] = [];
    shouldStopAnimation = false;

    timeDomain: Uint8Array;
    freqDomain: Uint8Array;
    color: string;

    constructor(canvas: HTMLCanvasElement) {
        this.audioElement = audioPlayer.getCurrentAudioFile();
        this.canvas = canvas;
        this._canvasContext = this.canvas.getContext("2d");
        this.setupAnalyserAudio();
        this.setupAnalyserAnimation();

        currentThemeObject.subscribe((theme) => {
            this.color = theme.oscilloscope;
        });
    }

    setCanvas(canvas: HTMLCanvasElement) {
        this.canvas = canvas;
        this._canvasContext = canvas.getContext("2d");
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
            this._canvasContext.shadowColor = this.color;
            this._canvasContext.shadowBlur = 10;
            this._canvasContext.shadowOffsetX = 2;
            this._canvasContext.shadowOffsetY = 2;
            this._canvasContext.strokeStyle = this.color;
            this._canvasContext.lineTo(x, y);
        }

        this._canvasContext.stroke();
    }

    /**
     * Connect the nodes together, so sound is flowing
     * into the analyser node from the source node
     */
    setupAnalyserAudio() {
        audioPlayer.connectAnalyser();
    }

    setupAnalyserAnimation() {
        this.freqDomain = new Uint8Array(
            audioPlayer._audioAnalyser.frequencyBinCount
        );
        this.timeDomain = new Uint8Array(audioPlayer._audioAnalyser.fftSize);

        let tick = () => {
            audioPlayer._audioAnalyser.getByteFrequencyData(this.freqDomain);
            audioPlayer._audioAnalyser.getByteTimeDomainData(this.timeDomain);
            this.clearCanvas();
            if (!this.shouldStopAnimation) {
                this.drawOscilloscope();
                window.requestAnimationFrame(tick);
            }
        };
        tick();
    }

    clearCanvas() {
        this._canvasContext.clearRect(
            0,
            0,
            this.canvas.clientWidth,
            this.canvas.clientHeight
        );
    }
    public addAnimation(animation: IAnimation): void {
        this._activeAnimations.push(animation);
    }

    public clearAnimations(): void {
        this._activeAnimations = [];
    }

    tearDown() {
        audioPlayer.disconnectAnalyser();
        this.clearCanvas();
    }
}
