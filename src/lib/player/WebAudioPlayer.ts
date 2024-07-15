import { get, writable } from "svelte/store";
import type { Writable } from "svelte/store";
import type { IAItem } from "../../App";

export const isPlaying = writable(false);
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
    onTimeUpdate = (time: number) => {};

    private constructor() {
        this.audioFile = new Audio();
        this.audioFile.crossOrigin = "anonymous";

        this.audioFile.addEventListener('ended', () => {
            isPlaying.set(false);
        });
    }

    getCurrentAudioFile() {
        return this.audioFile;
    }

    setVolume(vol) {
        // volume.set(vol);
    }

    async playFromUrl(item: IAItem) {
        this.pause();
        this.getCurrentAudioFile().currentTime = 0;
        this.getCurrentAudioFile().src = item.previewSrc.replace("?", "%3F");
        currentSrc.set(item.previewSrc);
        currentItem.set(item);
        this.play();
    }

    play() {
        this.getCurrentAudioFile().play();
        isPlaying.set(true);
        this.ticker = setInterval(() => {
            this.onTimeUpdate(this.audioFile.currentTime);
        }, 1000);
    }

    pause() {
        this.getCurrentAudioFile().pause();
        isPlaying.set(false);
    }

    seek(time) {
        if (this.getCurrentAudioFile()) {
            this.getCurrentAudioFile().currentTime = time;
        }
    }

    togglePlay() {
        if (get(isPlaying)) {
            this.pause();
        } else {
            this.play();
        }
    }
}

const audioPlayer = WebAudioPlayer.Instance;

export default audioPlayer;
