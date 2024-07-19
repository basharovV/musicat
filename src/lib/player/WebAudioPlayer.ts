import { get, writable } from "svelte/store";
import type { Writable } from "svelte/store";
import type { IAFile, IAItem } from "../../App";
import {
    currentIAFile,
    isPlaying,
    webPlayerBufferedRanges,
    webPlayerIsLoading,
    webPlayerVolume
} from "../../data/store";
import audioPlayer from "./AudioPlayer";
import toast from "svelte-french-toast";

export const isIAPlaying = writable(false);
export const currentSrc: Writable<string> = writable(null);
export const currentItem: Writable<IAItem> = writable(null);

class WebAudioPlayer {
    private static _instance: WebAudioPlayer;

    public static get Instance() {
        return this._instance || (this._instance = new this());
    }

    audioFile: HTMLAudioElement;
    currentAudioFile: 1 | 2 = 1;
    duration: number;
    ticker;
    isFullyBuffered = false;
    onTimeUpdate = (time: number) => {};

    private _audioContext: AudioContext;

    // Nodes
    private _audioSource: MediaElementAudioSourceNode;
    private _gainNode: GainNode;
    _audioAnalyser: AnalyserNode;

    private constructor() {
        this.audioFile = new Audio();
        this.audioFile.crossOrigin = "anonymous";

        // Nodes
        let AudioContext = window.AudioContext || window.webkitAudioContext;
        this._audioContext = new AudioContext({ sampleRate: 48000 });
        this._audioSource = this._audioContext.createMediaElementSource(
            this.audioFile
        );

        this._gainNode = new GainNode(this._audioContext, {
            gain: get(webPlayerVolume)
        });
        this._audioSource.mediaElement.volume = get(webPlayerVolume);

        this._audioAnalyser = new AnalyserNode(this._audioContext, {
            fftSize: 2048,
            smoothingTimeConstant: 0.9
        });

        // Source -> Gain -> (Analyser) -> Destination (master)
        this._audioSource.connect(this._gainNode);
        this._gainNode.connect(this._audioContext.destination);

        this.audioFile.addEventListener("ended", () => {
            isIAPlaying.set(false);
        });

        this.audioFile.addEventListener("canplay", () => {
            webPlayerIsLoading.set(false);
        });
        this.audioFile.addEventListener("canplaythrough", () => {
            webPlayerIsLoading.set(false);
        });
        this.audioFile.addEventListener("error", (e) => {
            switch (e.target.error.code) {
                case e.target.error.MEDIA_ERR_ABORTED:
                    break;
                case e.target.error.MEDIA_ERR_NETWORK:
                    toast.error(
                        "A network error caused the audio download to fail."
                    );
                    break;
                case e.target.error.MEDIA_ERR_DECODE:
                    toast.error(
                        "The audio playback was aborted due to a decoding issue."
                    );
                    break;
                case e.target.error.MEDIA_ERR_SRC_NOT_SUPPORTED:
                    toast.error(
                        "The audio not be loaded, either because network failed or due to an issue with the format."
                    );
                    break;
                default:
                    break;
            }
            webPlayerIsLoading.set(false);
            isIAPlaying.set(false);
        });

        webPlayerVolume.subscribe((vol) => {
            this.setVolume(vol);
        });
    }

    getCurrentAudioFile() {
        return this.audioFile;
    }

    setVolume(vol) {
        this.audioFile.volume = vol;
        this._gainNode.gain.value = vol;
    }

    async playFromUrl(file: IAFile) {
        currentIAFile.set(file);
        webPlayerIsLoading.set(true); // Is buffering initial chunk
        this.pause();
        this.getCurrentAudioFile().currentTime = 0;
        this.onTimeUpdate(0);
        this.getCurrentAudioFile().src = file.previewSrc.replace("?", "%3F");
        this.isFullyBuffered = false;
        currentSrc.set(file.previewSrc);
        webPlayerBufferedRanges.set(this.audioFile.buffered);
        this.play();
    }

    play() {
        // Pause main player if currently playing
        if (get(isPlaying)) {
            audioPlayer.pause();
        }
        this.getCurrentAudioFile().play();
        isIAPlaying.set(true);
        this.ticker = setInterval(() => {
            this.onTimeUpdate(this.audioFile.currentTime);

            // Check buffered
            let updateBufferedRanges = false;
            for (let i = 0; i < this.audioFile.buffered.length; i++) {
                if (
                    !this.isFullyBuffered &&
                    this.audioFile.buffered.end(i) <= this.audioFile.duration
                ) {
                    updateBufferedRanges = true;
                }
                if (
                    this.audioFile.buffered.end(i) === this.audioFile.duration
                ) {
                    this.isFullyBuffered = true;
                }
            }
            updateBufferedRanges &&
                webPlayerBufferedRanges.set(this.audioFile.buffered);
        }, 1000);
    }

    pause() {
        this.getCurrentAudioFile().pause();
        isIAPlaying.set(false);
    }

    seek(time) {
        if (this.getCurrentAudioFile()) {
            this.getCurrentAudioFile().currentTime = time;
        }
    }

    togglePlay() {
        if (get(isIAPlaying)) {
            this.pause();
        } else {
            this.play();
        }
    }

    connectAnalyser() {
        try {
            this._gainNode.disconnect(this._audioContext.destination);
            this._gainNode.connect(this._audioAnalyser);
            this._audioAnalyser.connect(this._audioContext.destination);
        } catch (err) {}
    }

    disconnectAnalyser() {
        try {
            this._audioAnalyser.disconnect(this._audioContext.destination);
            this._gainNode.disconnect(this._audioAnalyser);
            this._gainNode.connect(this._audioContext.destination);
        } catch (err) {}
    }
}

const webAudioPlayer = WebAudioPlayer.Instance;

export default webAudioPlayer;
