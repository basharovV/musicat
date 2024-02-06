import { get } from "svelte/store";
import { isPlaying, volume } from "../../data/store";
import audioPlayer from "./AudioPlayer";

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
    canvas: HTMLCanvasElement;

    private _canvasContext: CanvasRenderingContext2D;
    private _activeAnimations: IAnimation[] = [];
    isEnabled = true;
    shouldStopAnimation = false;
    timeDomain: Uint8Array;
    freqDomain: Uint8Array;

    constructor(canvas: HTMLCanvasElement) {
        this.canvas = canvas;
        this._canvasContext = this.canvas.getContext("2d");

        isPlaying.subscribe((playing) => {
            this.shouldStopAnimation = !playing;
            if (playing) {
                this.setupAnalyserAnimation();
            } else {
                this.clearCanvas();
            }
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
            this._canvasContext.shadowColor = "#14D8BD";
            this._canvasContext.shadowBlur = 10;
            this._canvasContext.shadowOffsetX = 2;
            this._canvasContext.shadowOffsetY = 2;
            this._canvasContext.strokeStyle = "#14D8BD";
            this._canvasContext.lineTo(x, y);
        }

        this._canvasContext.stroke();
    }

    clearCanvas() {
        this._canvasContext.clearRect(
            0,
            0,
            this._canvasContext.canvas.clientWidth,
            this._canvasContext.canvas.clientHeight
        );
    }
    
    setupAnalyserAnimation() {
        this.freqDomain = new Uint8Array(
            audioPlayer.audioAnalyser.frequencyBinCount
        );
        this.timeDomain = new Uint8Array(audioPlayer.audioAnalyser.fftSize);

        let tick = () => {
            audioPlayer.audioAnalyser.getByteFrequencyData(this.freqDomain);
            audioPlayer.audioAnalyser.getByteTimeDomainData(this.timeDomain);
            this.clearCanvas();
            if (!this.shouldStopAnimation) {
                this.drawOscilloscope();
                window.requestAnimationFrame(tick);
            }
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
