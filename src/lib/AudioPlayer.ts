import { get } from "svelte/store";
import {
  currentSong,
  isPlaying,
  playerTime,
  seekTime,
  volume,
} from "../data/store";

class AudioPlayer {
  private static _instance: AudioPlayer;

  public static get Instance() {
    // Do you need arguments? Make it a regular static method instead.
    return this._instance || (this._instance = new this());
  }

  audioFile: HTMLAudioElement;
  source: MediaElementAudioSourceNode;
  duration: number;
  gainNode: GainNode;
  ticker;
  isFinishedCallback;

  private constructor() {
    const audioCtx = new AudioContext();
    this.gainNode = audioCtx.createGain();
    this.gainNode.gain.value = 1;
    this.gainNode.connect(audioCtx.destination);

    this.audioFile = new Audio();
    this.source = audioCtx.createMediaElementSource(this.audioFile);
    this.source.connect(this.gainNode);

    seekTime.subscribe((playerTime) => {
      if (this.audioFile) {
        console.log("seeking to ", playerTime);
        this.audioFile.currentTime = playerTime;

        console.log("seeked  ", this.audioFile.currentTime);
      }
    });

    this.audioFile.addEventListener("pause", this.onPause.bind(this));
    this.audioFile.addEventListener("play", this.onPlay.bind(this));
    this.audioFile.addEventListener("ended", this.onEnded.bind(this));
    volume.set(this.audioFile.volume);
    volume.subscribe((vol) => {
      this.gainNode.gain.value = vol;
    });
  }

  setVolume(volume) {
    this.audioFile.volume = volume;
  }

  onPlay() {
    isPlaying.set(true);
    console.log("callback in play", this.isFinishedCallback);
  }

  onPause() {
    isPlaying.set(false);
  }

  onEnded() {
    console.log("ENDED!");
    isPlaying.set(false);
    console.log("callback", this.isFinishedCallback);
    this.isFinishedCallback && this.isFinishedCallback();
  }

  setAudioFinishedCallback(callback) {
    console.log("passing in callback", callback);
    this.isFinishedCallback = callback;
  }

  // MEDIA
  playSong(song: Song) {
    console.log("playSong", song);
    const convertedPath = window.__TAURI__.tauri.convertFileSrc(song.path);
    if (this.audioFile) {
      this.pause();
      this.audioFile.currentTime = 0;
      this.audioFile.src = "asset://" + song.path.replace('?', '%3F');
      this.play();
      currentSong.set(song);
    }
  }

  play() {
    if (this.audioFile) {
      this.audioFile.play();
      this.ticker = setInterval(() => {
        playerTime.set(this.audioFile.currentTime);

      }, 1000);
    }
  }

  pause() {
    if (this.audioFile) {
      this.audioFile.pause();
      clearInterval(this.ticker);
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

const audioPlayer = AudioPlayer.Instance;

export default audioPlayer;
