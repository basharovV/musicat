import { invoke } from "@tauri-apps/api/tauri";
import { get } from "svelte/store";
import type { ArtworkSrc, LastPlayedInfo, Song } from "../../App";
import { db } from "../../data/db";
import {
    currentSong,
    currentSongArtworkSrc,
    currentSongIdx,
    isPlaying,
    isShuffleEnabled,
    lastPlayedInfo,
    nextUpSong,
    playerTime,
    playlist,
    seekTime,
    shuffledPlaylist,
    volume
} from "../../data/store";
import { shuffleArray } from "../../utils/ArrayUtils";

import type { Event } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";
import WebRTCReceiver from "./WebRTCReceiver";
import { register } from "@tauri-apps/api/globalShortcut";

if (!ReadableStream.prototype[Symbol.asyncIterator]) {
    ReadableStream.prototype[Symbol.asyncIterator] = async function* () {
        const reader = this.getReader();
        try {
            while (true) {
                const { done, value } = await reader.read();
                if (done) {
                    yield { done, value };
                    return;
                }
                yield { done, value };
            }
        } finally {
            reader.releaseLock();
        }
    };
}

class AudioPlayer {
    private static _instance: AudioPlayer;

    public static get Instance() {
        // Do you need arguments? Make it a regular static method instead.
        return this._instance || (this._instance = new this());
    }

    audioFile: HTMLAudioElement;
    audioFile2: HTMLAudioElement; // for gapless playback

    currentAudioFile: 1 | 2 = 1;
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
    shouldPlay = false; // Whether to play immediately after loading playlist
    shouldRestoreLastPlayed: LastPlayedInfo;
    isRunningTransition = false;
    isInit = true;
    isStopped = true;
    // WebRTC
    webRTCReceiver: WebRTCReceiver;
    packet_n: number = 0;
    stats = {
        packetDropCounter: 0,
        packetReceivedCounter: 0,
        totalPacketCounter: 0
    };
    flowControlSendInterval;

    private constructor() {
        this.webRTCReceiver = new WebRTCReceiver();

        seekTime.subscribe((time) => {
            // Tell Rust to play file with seek position
            if (this.currentSong) this.playCurrent(time);
        });

        this.setupMediaSession();
        currentSongArtworkSrc.subscribe((artwork) => {
            this.artworkSrc = artwork;
            this.setMediaSessionData();
        });

        isShuffleEnabled.subscribe((enabled) => {
            const shuffledPlylist = get(shuffledPlaylist);
            let newCurrentSongIdx = 0;
            // If shuffle is enabled but not yet shuffled
            if (enabled && shuffledPlylist.length === 0) {
                this.shuffle();
                // If shuffle is disabled, need to unshuffle
            } else if (!enabled && shuffledPlylist.length > 0) {
                this.unshuffle();
                // Restore position
                newCurrentSongIdx = this.playlist.findIndex(
                    (s) => s.id === this.currentSong?.id
                );
                if (newCurrentSongIdx === -1) {
                    newCurrentSongIdx = 0;
                }
            }

            this.currentSongIdx = newCurrentSongIdx;
            currentSongIdx.set(newCurrentSongIdx);

            this.setNextUpSong();
        });

        playlist.subscribe(async (playlist) => {
            this.playlist = playlist;
            const shuffleEnabled = get(isShuffleEnabled);
            const shuffledPlylist = get(shuffledPlaylist);
            let newCurrentSongIdx = 0;
            console.log("shuffled", shuffledPlylist);
            // If shuffle is enabled but not yet shuffled
            if (shuffleEnabled) {
                this.shuffle();
                // If shuffle is disabled, need to unshuffle
            } else if (!shuffleEnabled && shuffledPlylist.length > 0) {
                this.unshuffle();
                // Restore position
                newCurrentSongIdx = playlist.findIndex(
                    (s) => s.id === this.currentSong?.id
                );
                if (newCurrentSongIdx === -1) {
                    newCurrentSongIdx = 0;
                }
            } else {
                // Shuffle was disabled already
                this.playlist = playlist;

                newCurrentSongIdx = playlist.findIndex(
                    (s) => s.id === this.currentSong?.id
                );
                console.log("found index", playlist, this.currentSong);
                if (newCurrentSongIdx === -1) {
                    newCurrentSongIdx = 0;
                }
            }

            this.currentSongIdx = newCurrentSongIdx;
            currentSongIdx.set(newCurrentSongIdx);
            console.log(
                "playlist: currentsongindex",
                this.currentSongIdx,
                this.currentSong
            );

            this.setNextUpSong();
            if (this.shouldPlay) {
                this.playCurrent();
                this.shouldPlay = false;
            }
            if (this.shouldRestoreLastPlayed) {
                await this.restoreLastPlayed(this.shouldRestoreLastPlayed);
                this.shouldRestoreLastPlayed = null;
            }
        });
        currentSongIdx.subscribe((idx) => {
            this.currentSongIdx = idx;
        });

        volume.subscribe(async (vol) => {
            await invoke("volume_control", {
                event: {
                    volume: vol * 0.75
                }
            });
            localStorage.setItem("volume", String(vol));
        });

        appWindow.listen("song_change", async (event: Event<Song>) => {
            this.currentSong = event.payload;
            currentSong.set(this.currentSong);
            this.currentSongIdx += 1;
            currentSongIdx.set(this.currentSongIdx);
            this.setNextUpSong();
            this.isRunningTransition = false;
        });

        appWindow.listen("timestamp", async (event: any) => {
            playerTime.set(event.payload);
            isPlaying.set(true);
        });
    }

    async setupBuffers() {
        console.log("player::setupBuffers()");
    }

    shuffle() {
        let toShuffle = [...this.playlist];
        toShuffle.splice(get(currentSongIdx), 1);
        const shuffled = [this.currentSong].concat(shuffleArray(toShuffle));
        shuffledPlaylist.set(shuffled);
        this.playlist = shuffled;
        isShuffleEnabled.set(true);
    }

    unshuffle() {
        this.playlist = get(playlist);
        shuffledPlaylist.set([]);
        isShuffleEnabled.set(false);
    }

    onPlay() {
        isPlaying.set(true);
        if (navigator.mediaSession)
            navigator.mediaSession.playbackState = "playing";
    }

    onPause() {
        isPlaying.set(false);
        if (navigator.mediaSession)
            navigator.mediaSession.playbackState = "paused";
    }

    playCurrent(seek: number = 0) {
        this.playSong(this.playlist[this.currentSongIdx], seek);
    }

    hasNext() {
        return this.currentSongIdx + 1 < this.playlist.length;
    }

    playNext() {
        this.currentSongIdx++;
        currentSongIdx.set(this.currentSongIdx);
        this.playSong(this.playlist[this.currentSongIdx]);
    }

    playPrevious() {
        this.currentSongIdx--;
        currentSongIdx.set(this.currentSongIdx);
        this.playSong(this.playlist[this.currentSongIdx]);
        currentSongIdx.set(this.currentSongIdx);
    }

    restart() {}

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
                    console.log("action handler play");
                    this.play();
                }
            ],
            [
                "pause",
                () => {
                    console.log("action handler pause");
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
        if (this.currentSongIdx + 1 < this.playlist?.length) {
            const nextSong = this.playlist[this.currentSongIdx + 1];
            nextUpSong.set(nextSong);
            invoke("queue_next", {
                event: {
                    path: nextSong.path,
                    seek: 0,
                    file_info: nextSong.fileInfo,
                    volume: get(volume)
                }
            });
        } else {
            nextUpSong.set(null);
        }
    }

    async incrementPlayCounter(song: Song) {
        await db.songs.update(song, {
            playCount: song.playCount ? song.playCount + 1 : 1
        });
    }

    // MEDIA

    async playSong(song: Song, position = 0, play = true) {
        if (song) {
            // this.pause();
            this.isRunningTransition = false;
            currentSong.set(song);
            this.currentSong = song;

            if (play) {
                await this.play(false);
                playerTime.set(position);
                console.log(
                    "audioplayer::datachannel::",
                    this.webRTCReceiver.dataChannel?.readyState
                );

                if (
                    !this.webRTCReceiver.dataChannel ||
                    this.webRTCReceiver.dataChannel.readyState !== "open"
                ) {
                    // Try to reconnect
                    this.webRTCReceiver.playerConnection?.close();
                    this.webRTCReceiver.remoteConnection?.close();
                    this.webRTCReceiver.dataChannel.close();
                    this.webRTCReceiver.init();
                }

                // If the WebRTC receiver is ready to receive data, invoke the streamer
                this.webRTCReceiver.prepareForNewStream();
                invoke("stream_file", {
                    event: {
                        path: this.currentSong.path,
                        seek: position,
                        file_info: this.currentSong.fileInfo,
                        volume: get(volume)
                    }
                });
                this.incrementPlayCounter(song);
            }
            this.shouldPlay = play;
            let newCurrentSongIdx = this.playlist.findIndex(
                (s) => s.id === this.currentSong?.id
            );
            if (newCurrentSongIdx === -1) {
                newCurrentSongIdx = 0;
            }
            currentSongIdx.set(newCurrentSongIdx);
            this.setNextUpSong();
            this.setMediaSessionData();

            lastPlayedInfo.set({
                songId: this.currentSong.id,
                position: 0
            });
        }
    }

    async play(isResume: boolean) {
        this.isStopped = false;
        if (isResume) {
            invoke("decode_control", {
                event: {
                    decoding_active: true
                }
            });
        }
        this.onPlay();
    }

    async pause() {
        invoke("decode_control", {
            event: {
                decoding_active: false
            }
        });
        this.onPause();
    }

    async onEnded() {}

    togglePlay() {
        if (get(isPlaying)) {
            this.pause();
        } else {
            if (this.isStopped) {
                this.playCurrent();
            } else {
                this.play(true);
            }
        }
    }

    async restoreLastPlayed(lastPlayed: LastPlayedInfo) {
        const song = await db.songs.get(lastPlayed.songId);
        this.playSong(song, lastPlayed.position, false);
    }
}

const audioPlayer = AudioPlayer.Instance;

export default audioPlayer;
