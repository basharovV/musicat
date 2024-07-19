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

    private constructor() {
        this.audioFile = new Audio();
        this.audioFile.crossOrigin = "anonymous";

        this.audioFile.addEventListener("ended", () => {
            isIAPlaying.set(false);
        });

        this.audioFile.addEventListener("canplay", () => {
            webPlayerIsLoading.set(false);
        });
        this.audioFile.addEventListener("canplaythrough", () => {
            webPlayerIsLoading.set(false);
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
}

const webAudioPlayer = WebAudioPlayer.Instance;

export default webAudioPlayer;
