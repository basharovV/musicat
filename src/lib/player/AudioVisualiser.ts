import { isPlaying } from "../../data/store";
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
    lastTick = performance.now();

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

    interpolateUint8Array(array, factor) {
        const length = array.length;
        const interpolatedLength = length * factor - (factor - 1);
        const interpolatedArray = new Uint8Array(interpolatedLength);
    
        for (let i = 0; i < length - 1; i++) {
            interpolatedArray[i * factor] = array[i];
            for (let j = 1; j < factor; j++) {
                interpolatedArray[i * factor + j] = (j * array[i] + (factor - j) * array[i + 1]) / factor;
            }
        }
    
        // Copy the last value from the original array
        interpolatedArray[interpolatedLength - 1] = array[length - 1];
        
        return interpolatedArray;
    }

    setupAnalyserAnimation() {
        // this.freqDomain = new Uint8Array(
        //     audioPlayer.audioAnalyser.frequencyBinCount
        // );
        this.timeDomain = new Uint8Array([
            128, 128, 128, 128, 128, 128, 128, 128, 128, 128
        ]);

        // Append new messages to the box of incoming messages
        if (audioPlayer.webRTCReceiver) {
            audioPlayer.webRTCReceiver.onSampleData = (samples) => {
                // Get the ArrayBuffer
                // console.log("samples", samples);
                this.timeDomain = new Uint8Array(samples);
            };
        }

        let tick = () => {
            this.clearCanvas();
            if (!this.shouldStopAnimation) {
                this.drawOscilloscope();
                window.requestAnimationFrame(tick);
            }
        };
        tick();
    }

    /**
     * Draw an oscilloscope
     */
    drawOscilloscope() {
        // Assuming 44100hz, TODO: Make dynamic
        if (this.lastTick < 0.022675737) {
            return;
        }
        this.lastTick = performance.now();
        const step = this._canvasContext.canvas.width / this.timeDomain.length;

        this._canvasContext.beginPath();
        for (let i = 0; i < this.timeDomain.length; i += 1) {
            const percent = this.timeDomain[i] / 256;
            const x = i * step;
            const yL = this._canvasContext.canvas.height * percent;
            this._canvasContext.shadowColor = "#14D8BD";
            this._canvasContext.shadowBlur = 10;
            this._canvasContext.shadowOffsetX = 2;
            this._canvasContext.shadowOffsetY = 2;
            this._canvasContext.strokeStyle = "#14D8BD";
            this._canvasContext.lineTo(x, yL);
        }

        this._canvasContext.stroke();
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
        
    }
}

