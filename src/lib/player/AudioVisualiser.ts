import { get } from "svelte/store";
import { isPlaying, uiPreferences } from "../../data/store";
import { currentThemeObject } from "../../theming/store";
import audioPlayer from "./AudioPlayer";
import type { AnalyzerType } from "../../App";
import { invoke } from "@tauri-apps/api/core";

export interface IAnimation {
    draw: (
        audioBufferData: Uint8Array,
        canvasElement: CanvasRenderingContext2D,
    ) => void;
}

/**
 * Visualiser for playing audio using Canvas
 */ export class AudioVisualiser {
    canvas: HTMLCanvasElement;
    private _canvasContext: CanvasRenderingContext2D;
    private _activeAnimations: IAnimation[] = [];
    isEnabled = true;
    analyzerType: AnalyzerType = "time";
    shouldStopAnimation = false;
    timeDomain: Uint8Array;
    freqDomain: Uint8Array; // Added frequency storage
    lastTick = performance.now();
    color: string;
    private _animationFrameId: number | null = null;

    constructor(canvas: HTMLCanvasElement) {
        this.canvas = canvas;
        this._canvasContext = this.canvas.getContext("2d");

        // Initialize with neutral values (128 for time, 0 for freq)
        this.timeDomain = new Uint8Array(256).fill(128);
        this.freqDomain = new Uint8Array(256).fill(1);

        isPlaying.subscribe((playing) => {
            this.shouldStopAnimation = !playing;
            if (playing) {
                this.setupAnalyserAnimation();
            } else {
                this.timeDomain = new Uint8Array(256).fill(128);
                this.freqDomain = new Uint8Array(256).fill(1);

                this.clearCanvas();

                if (this.analyzerType === "frequency") {
                    this.drawFrequencyBars();
                } else {
                    this.drawOscilloscope();
                }
            }
        });

        currentThemeObject.subscribe((theme) => {
            this.color = theme.oscilloscope;
        });

        uiPreferences.subscribe((preferences) => {
            this.isEnabled = preferences.audioAnalyzer.isEnabled;
            this.analyzerType = preferences.audioAnalyzer.analyzerType;

            invoke("analyzer_control", {
                event: {
                    enabled: this.isEnabled,
                    analyzer_type: this.analyzerType,
                },
            });
        });
    }

    receivedPerSecond = 0;
    setupAnalyserAnimation() {
        // Kill any existing loop before starting a new one
        if (this._animationFrameId !== null) {
            cancelAnimationFrame(this._animationFrameId);
        }
        if (audioPlayer.webRTCReceiver) {
            audioPlayer.webRTCReceiver.onSampleData = (samples: Uint8Array) => {
                // check if second has passed
                if (performance.now() - this.lastTick < 1000) {
                    this.receivedPerSecond++;
                } else {
                    console.log(
                        "received per second",
                        this.receivedPerSecond,
                        "FPS",
                    );
                    this.receivedPerSecond = 0;
                    this.lastTick = performance.now();
                }
                this.timeDomain = samples;
                this.freqDomain = samples;
            };
        }

        let tick = () => {
            if (this.shouldStopAnimation) {
                this._animationFrameId = null;
                return;
            }

            this.clearCanvas();

            if (this.analyzerType === "frequency") {
                this.drawFrequencyBars();
            } else {
                this.drawOscilloscope();
            }

            this._animationFrameId = window.requestAnimationFrame(tick);
        };

        this._animationFrameId = window.requestAnimationFrame(tick);
    }

    /**
     * Draw Frequency Bars
     */
    drawFrequencyBars() {
        if (!this.freqDomain || this.freqDomain.length === 0) return;

        this._canvasContext.shadowBlur = 0;
        const barWidth = this.canvas.width / this.freqDomain.length;
        const centerY = this.canvas.height / 2; // Find the vertical middle

        for (let i = 0; i < this.freqDomain.length; i++) {
            // Map the 0-255 value to the canvas height
            const totalHeight = (this.freqDomain[i] / 255) * this.canvas.height;

            // For symmetry, the bar "starts" half-way above the center
            const x = i * barWidth;
            const y = centerY - totalHeight / 2;

            this._canvasContext.fillStyle = this.color;

            // Draw the rectangle (x, y, width, height)
            // By starting at y and drawing down 'totalHeight',
            // it perfectly straddles the center line.
            this._canvasContext.fillRect(x, y, barWidth - 1, totalHeight);
        }
    }
    drawOscilloscope() {
        if (!this.timeDomain || this.timeDomain.length === 0) return;

        const step = this._canvasContext.canvas.width / this.timeDomain.length;

        this._canvasContext.beginPath();
        this._canvasContext.lineWidth = 2;
        this._canvasContext.strokeStyle = this.color;
        this._canvasContext.shadowColor = this.color;
        this._canvasContext.shadowBlur = 10;

        for (let i = 0; i < this.timeDomain.length; i += 1) {
            const percent = this.timeDomain[i] / 256;
            const x = i * step;
            const y = this._canvasContext.canvas.height * percent;

            if (i === 0) {
                this._canvasContext.moveTo(x, y);
            } else {
                this._canvasContext.lineTo(x, y);
            }
        }
        this._canvasContext.stroke();
    }

    clearCanvas() {
        this._canvasContext.clearRect(
            0,
            0,
            this.canvas.width,
            this.canvas.height,
        );
    }

    public addAnimation(animation: IAnimation): void {
        this._activeAnimations.push(animation);
    }

    public clearAnimations(): void {
        this._activeAnimations = [];
    }

    tearDown() {
        this.clearCanvas();
    }
}
