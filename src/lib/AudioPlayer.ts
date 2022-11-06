import type { ArtworkSrc, Song } from "src/App";
import { get } from "svelte/store";
import {
    currentSong,
    currentSongArtworkSrc,
    currentSongIdx,
    isPlaying,
    nextUpSong,
    playerTime,
    playlist,
    seekTime,
    volume
} from "../data/store";

class AudioPlayer {
    private static _instance: AudioPlayer;

    public static get Instance() {
        // Do you need arguments? Make it a regular static method instead.
        return this._instance || (this._instance = new this());
    }

    audioFile: HTMLAudioElement;
    // source: MediaElementAudioSourceNode;
    duration: number;
    // gainNode: GainNode;
    ticker;
    isFinishedCallback;
    artworkSrc: ArtworkSrc; // for media session (notifications)
    currentSong: Song;
    currentSongIdx: number;
    isAlreadyLoadingSong = false; // for when the 'ended' event fires
    playlist: Song[];
    private constructor() {
        // let AudioContext = window.AudioContext || window.webkitAudioContext;
        // const audioCtx: AudioContext = new AudioContext();
        // this.gainNode = audioCtx.createGain();
        // this.gainNode.gain.value = 1;
        // this.gainNode.connect(audioCtx.destination);

        this.audioFile = new Audio();
        // this.source = audioCtx.createMediaElementSource(this.audioFile);
        // this.source.connect(this.gainNode);

        seekTime.subscribe((time) => {
            if (this.audioFile) {
                console.log("seeking to ", time);
                this.audioFile.currentTime = time;
                playerTime.set(time);
                console.log("seeked  ", this.audioFile.currentTime);
            }
        });

        this.audioFile.addEventListener("pause", this.onPause.bind(this));
        this.audioFile.addEventListener("play", this.onPlay.bind(this));
        this.audioFile.addEventListener("ended", this.onEnded.bind(this));
        // volume.set(this.audioFile.volume);

        this.setupMediaSession();
        currentSongArtworkSrc.subscribe((artwork) => {
            this.artworkSrc = artwork;
            this.setMediaSessionData();
        });
        playlist.subscribe(async (playlist) => {
            this.playlist = playlist;
            let newCurrentSongIdx = playlist.findIndex(
                (s) => s.id === this.currentSong?.id
            );
            if (newCurrentSongIdx === -1) {
                newCurrentSongIdx = 0;
            }
            this.currentSongIdx = newCurrentSongIdx;
            currentSongIdx.set(newCurrentSongIdx);

            this.setNextUpSong();
        });
        currentSongIdx.subscribe((idx) => {
            this.currentSongIdx = idx;
        });
    }

    setVolume(vol) {
        // this.audioFile.volume = volume;
        volume.set(vol);
    }

    onPlay() {
        isPlaying.set(true);
        console.log("callback in play", this.isFinishedCallback);
        if (navigator.mediaSession)
            navigator.mediaSession.playbackState = "playing";
    }

    onPause() {
        isPlaying.set(false);
        if (navigator.mediaSession)
            navigator.mediaSession.playbackState = "paused";
    }

    onEnded() {
        console.log("ENDED!");
        if (this.isAlreadyLoadingSong) {
            this.isAlreadyLoadingSong = false;
            console.log("already loading song");
        } else {
            isPlaying.set(false);
            console.log("callback", this.isFinishedCallback);
            this.isFinishedCallback && this.isFinishedCallback();
            this.playNext();
        }
    }

    playNext() {
        this.currentSongIdx++;
        this.playSong(this.playlist[this.currentSongIdx]);
        currentSongIdx.set(this.currentSongIdx);
    }

    setAudioFinishedCallback(callback) {
        this.isFinishedCallback = callback;
    }

    /**
     * Set the media session data (for browser / OS notifications)
     */
    setMediaSessionData() {
        if (this.currentSong && "mediaSession" in navigator) {
            navigator.mediaSession.metadata = new MediaMetadata({
                title: this.currentSong.title,
                artist: this.currentSong.artist,
                album: this.currentSong.album,
                artwork: this.artworkSrc
                    ? [
                          {
                              src: this.artworkSrc.src,
                              sizes: this.artworkSrc.size
                                  ? `${this.artworkSrc.size.width}x${this.artworkSrc.size.height}`
                                  : "128x128",
                              type: this.artworkSrc.format
                          }
                      ]
                    : []
            });
        }
    }

    setupMediaSession() {
        const actionHandlers = [
            [
                "play",
                () => {
                    this.play();
                }
            ],
            [
                "pause",
                () => {
                    // Pause active playback
                    this.pause();
                }
            ],
            [
                "previoustrack",
                () => {
                    /* ... */
                }
            ],
            [
                "nexttrack",
                () => {
                    /* ... */
                }
            ],
            [
                "stop",
                () => {
                    /* ... */
                }
            ],
            ["seekbackward", null],
            ["seekforward", null],
            [
                "seekto",
                (details) => {
                    /* ... */
                }
            ]
        ];

        for (const [action, handler] of actionHandlers) {
            try {
                navigator.mediaSession.setActionHandler(action, handler);
            } catch (error) {
                console.log(
                    `The media session action "${action}" is not supported yet.`
                );
            }
        }
    }

    async wait(ms) {
        return new Promise<void>((resolve) => {
            setTimeout(() => {
                resolve();
            }, ms);
        });
    }

    setNextUpSong() {
        if (this.audioFile.src) {
            if (this.currentSongIdx + 1 < this.playlist.length) {
                nextUpSong.set(this.playlist[this.currentSongIdx + 1]);
            } else {
                nextUpSong.set(null);
            }
        } else {
            nextUpSong.set(null);
        }
    }

    // MEDIA
    async playSong(song: Song) {
        console.log("playSong", song);
        if (this.audioFile && song) {
            this.pause();
            this.audioFile.currentTime = 0;
            playerTime.set(this.audioFile.currentTime);
            this.audioFile.src = "asset://" + song.path.replace("?", "%3F");
            this.play();
            currentSong.set(song);
            this.setNextUpSong();
            this.currentSong = song;
            this.setMediaSessionData();
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
