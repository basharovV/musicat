import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import type { ArtworkSrc, Song, ToImport } from "../../App";
import { db } from "../../data/db";
import {
    current,
    currentSongArtworkSrc,
    isPlaying,
    isShuffleEnabled,
    isSongReady,
    nextUpSong,
    os,
    playerTime,
    queue,
    seekTime,
    shuffledQueue,
    userSettings,
    volume,
} from "../../data/store";
import { shuffleArray } from "../../utils/ArrayUtils";

import type { Event } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import WebRTCReceiver from "./WebRTCReceiver";
import webAudioPlayer, { isIAPlaying } from "./WebAudioPlayer";
const appWindow = getCurrentWebviewWindow();
import { listen } from "@tauri-apps/api/event";
import { TauriEvent } from "@tauri-apps/api/event";

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
    seek: number = 0;
    isAlreadyLoadingSong = false; // for when the 'ended' event fires
    queue: Song[];
    shouldPlay = false; // Whether to play immediately after loading playlist
    shouldShuffle = true; // Whether to suffle immediately after loading playlist
    isRunningTransition = false;
    isInit = true;
    isStopped = true;
    // WebRTC
    webRTCReceiver: WebRTCReceiver;
    packet_n: number = 0;
    stats = {
        packetDropCounter: 0,
        packetReceivedCounter: 0,
        totalPacketCounter: 0,
    };
    flowControlSendInterval;

    private constructor() {
        this.webRTCReceiver = new WebRTCReceiver();

        seekTime.subscribe((time) => {
            // Tell Rust to play file with seek position
            if (this.currentSong) {
                this.setSeek(time);
            }
        });

        this.setupMediaSession();
        currentSongArtworkSrc.subscribe((artwork) => {
            this.artworkSrc = artwork;
            this.setMediaSessionData();
        });

        isShuffleEnabled.subscribe((enabled) => {
            const queue = get(shuffledQueue);
            let newCurrentSongIdx = 0;
            // If shuffle is enabled but not yet shuffled
            if (enabled && queue.length === 0) {
                this.shuffle();
                // If shuffle is disabled, need to unshuffle
            } else if (!enabled && queue.length > 0) {
                this.unshuffle();
                // Restore position
                const id = this.currentSong?.id;
                if (id) {
                    newCurrentSongIdx = this.queue.findIndex(
                        (s) => s.id === id,
                    );
                } else {
                    newCurrentSongIdx = 0;
                }
            }

            this.currentSongIdx = newCurrentSongIdx;

            current.set({ ...get(current), index: newCurrentSongIdx });

            this.setNextUpSong();
        });

        current.subscribe(async ({ song, position }) => {
            if (this.currentSong) {
                if (this.currentSong !== song) {
                    this.seek = 0;
                }
            } else {
                this.seek = position || 0;
            }
            if (song) {
                this.currentSong = song;
            }
        });

        queue.subscribe(async (queue) => {
            if (queue.length === 0) {
                this.queue = queue;
                this.currentSongIdx = -1;
                return;
            }

            if (!this.currentSong) {
                this.currentSong = get(current).song;
            }

            const shuffleEnabled = get(isShuffleEnabled);
            let newCurrentSongIdx = 0;

            // If shuffle is enabled but not yet shuffled
            if (shuffleEnabled) {
                if (this.shouldShuffle) {
                    this.shuffle();
                } else {
                    this.queue = get(shuffledQueue);

                    const id = this.currentSong?.id;
                    if (id) {
                        newCurrentSongIdx = this.queue.findIndex(
                            (s) => s.id === id,
                        );
                    } else {
                        newCurrentSongIdx = 0;
                    }

                    this.shouldShuffle = true;
                }
                // If shuffle is disabled, need to unshuffle
            } else if (!shuffleEnabled && get(shuffledQueue).length > 0) {
                this.unshuffle();
                // Restore position
                const id = this.currentSong?.id;
                if (id) {
                    newCurrentSongIdx = this.queue.findIndex(
                        (s) => s.id === id,
                    );
                } else {
                    newCurrentSongIdx = 0;
                }
                // Shuffle was disabled already
            } else {
                this.queue = queue;

                const id = this.currentSong?.id;
                if (id) {
                    newCurrentSongIdx = this.queue.findIndex(
                        (s) => s.id === id,
                    );
                } else {
                    newCurrentSongIdx = 0;
                }
            }

            this.currentSongIdx = newCurrentSongIdx;

            current.set({ ...get(current), index: newCurrentSongIdx });

            console.log(
                "playlist: currentsongindex",
                this.currentSongIdx,
                this.currentSong,
            );

            this.setNextUpSong();

            if (this.shouldPlay) {
                const seek = get(current).position;

                this.playCurrent(seek);
                this.shouldPlay = false;

                if (seek) {
                    current.set({ ...get(current), position: 0 });
                }
            }
        });

        volume.subscribe(async (vol) => {
            await invoke("volume_control", {
                event: {
                    volume: vol * 0.75,
                },
            });
            localStorage.setItem("volume", String(vol));
        });

        isSongReady.subscribe(async () => {
            if (this.currentSong) {
                if (
                    !this.webRTCReceiver.dataChannel ||
                    (this.webRTCReceiver.dataChannel.readyState !== "open" &&
                        this.webRTCReceiver.playerConnection.signalingState !==
                            "stable")
                ) {
                    // Try to reconnect
                    this.webRTCReceiver.playerConnection?.close();
                    this.webRTCReceiver.remoteConnection?.close();
                    this.webRTCReceiver.dataChannel?.close();
                    this.webRTCReceiver.init();
                }

                // If the WebRTC receiver is ready to receive data, invoke the streamer
                this.webRTCReceiver.prepareForNewStream();

                invoke("stream_file", {
                    event: {
                        path: this.currentSong.path,
                        seek: 0,
                        file_info: this.currentSong.fileInfo,
                        volume: get(volume),
                        boot: true,
                    },
                });
            }
        });

        appWindow.listen("song_change", async (event: Event<Song>) => {
            this.currentSong = event.payload;
            this.currentSongIdx += 1;

            current.set({
                song: this.currentSong,
                index: this.currentSongIdx,
                position: 0,
            });

            this.setNextUpSong();
            this.isRunningTransition = false;
        });

        appWindow.listen("timestamp", async (event: any) => {
            playerTime.set(event.payload);
        });

        appWindow.listen("stopped", async (event: any) => {
            this.isStopped = true;
            playerTime.set(0);
            isPlaying.set(false);
        });

        appWindow.listen("paused", async (event: any) => {
            this.isStopped = false;
            isPlaying.set(false);
        });

        appWindow.listen("playing", async (event: any) => {
            this.isStopped = false;
            isPlaying.set(true);
        });

        appWindow.listen("audio_device_changed", async (event: any) => {
            userSettings.update((userSettings) => {
                userSettings.outputDevice = event.payload;
                return userSettings;
            });
        });

        if (get(os) === "macos") {
            document.addEventListener("keydown", (e) => {
                if (e.key === "q" && e.metaKey) {
                    current.set({ ...get(current), position: get(playerTime) });
                }
            });
        } else {
            listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
                current.set({ ...get(current), position: get(playerTime) });
            });
        }

        appWindow.listen("toggle_play", async (event: any) => {
            console.log("toggle_play");
            this.togglePlay();
        });

        appWindow.listen("play_next", async (event: any) => {
            console.log("play_next");
            this.playNext();
        });

        appWindow.listen("play_previous", async (event: any) => {
            console.log("play_previous");
            this.playPrevious();
        });
    }

    async setupBuffers() {
        console.log("player::setupBuffers()");
    }

    shuffle() {
        let toShuffle = [...this.queue];
        toShuffle.splice(this.currentSongIdx, 1);
        const shuffled = [this.currentSong, ...shuffleArray(toShuffle)];
        shuffledQueue.set(shuffled);
        var { position } = get(current);
        this.queue = shuffled;
        this.currentSongIdx = 0;
        current.set({
            song: this.currentSong,
            index: this.currentSongIdx,
            position,
        });
        isShuffleEnabled.set(true);
    }

    unshuffle() {
        this.queue = get(queue);
        shuffledQueue.set([]);
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
        if (this.currentSong) {
            this.playSong(this.currentSong, seek || this.seek);
        } else {
            this.playSong(this.queue[this.currentSongIdx], seek);
        }
    }

    hasNext() {
        return this.currentSongIdx + 1 < this.queue.length;
    }

    playNext() {
        this.currentSongIdx++;
        this.playSong(this.queue[this.currentSongIdx]);
    }

    playPrevious() {
        this.currentSongIdx--;
        this.playSong(this.queue[this.currentSongIdx]);
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
                              type: this.artworkSrc.format,
                          },
                      ]
                    : [],
            });
        }
    }

    setupMediaSession() {
        const actionHandlers = [
            [
                "play",
                () => {
                    console.log("action handler play");
                    this.play(true);
                },
            ],
            [
                "pause",
                () => {
                    console.log("action handler pause");
                    // Pause active playback
                    this.pause();
                },
            ],
            [
                "previoustrack",
                () => {
                    /* ... */
                },
            ],
            [
                "nexttrack",
                () => {
                    /* ... */
                },
            ],
            [
                "stop",
                () => {
                    /* ... */
                },
            ],
            ["seekbackward", null],
            ["seekforward", null],
            [
                "seekto",
                (details) => {
                    /* ... */
                },
            ],
        ];

        for (const [action, handler] of actionHandlers) {
            try {
                navigator.mediaSession.setActionHandler(action, handler);
            } catch (error) {
                console.log(
                    `The media session action "${action}" is not supported yet.`,
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
        if (this.currentSongIdx + 1 < this.queue?.length) {
            const nextSong = this.queue[this.currentSongIdx + 1];
            nextUpSong.set(nextSong);
            invoke("queue_next", {
                event: {
                    path: nextSong.path,
                    seek: 0,
                    file_info: nextSong.fileInfo,
                    volume: get(volume),
                },
            });
        } else {
            nextUpSong.set(null);
            // Tell the backend to clear the queue
            invoke("queue_next", {
                event: {
                    path: null,
                    seek: 0,
                    file_info: null,
                    volume: get(volume),
                },
            });
        }
    }

    setSeek(seek: number) {
        if (get(isPlaying)) {
            this.playSong(this.currentSong, seek);
        } else {
            this.seek = seek;

            playerTime.set(seek);
        }
    }

    async incrementPlayCounter(song: Song) {
        await db.songs.update(song, {
            playCount: song.playCount ? song.playCount + 1 : 1,
        });
    }

    // MEDIA

    async playSong(song: Song, position = 0, play = true, index = null) {
        this.seek = 0;

        if (song) {
            if (get(isIAPlaying)) {
                webAudioPlayer.pause();
            }
            // this.pause();
            this.isRunningTransition = false;
            this.currentSong = song;
            console.log("play", play, this.shouldPlay);
            if (play) {
                this.isStopped = false;
                this.onPlay();

                playerTime.set(position);
                console.log(
                    "audioplayer::datachannel::",
                    this.webRTCReceiver.dataChannel?.readyState,
                );

                if (
                    !this.webRTCReceiver.dataChannel ||
                    this.webRTCReceiver.dataChannel.readyState !== "open"
                ) {
                    // Try to reconnect
                    this.webRTCReceiver.playerConnection?.close();
                    this.webRTCReceiver.remoteConnection?.close();
                    this.webRTCReceiver.dataChannel?.close();
                    this.webRTCReceiver.init();
                }

                // If the WebRTC receiver is ready to receive data, invoke the streamer
                this.webRTCReceiver.prepareForNewStream();
                invoke("stream_file", {
                    event: {
                        path: this.currentSong.path,
                        seek: position,
                        file_info: this.currentSong.fileInfo,
                        volume: get(volume),
                    },
                });
                this.incrementPlayCounter(song);
            }
            this.shouldPlay = play;
            let newCurrentSongIdx =
                index !== null
                    ? index
                    : this.queue.findIndex(
                          (s) => s.id === this.currentSong?.id,
                      );
            if (newCurrentSongIdx === -1) {
                newCurrentSongIdx = 0;
            }
            this.currentSongIdx = newCurrentSongIdx;
            current.set({ song, index: newCurrentSongIdx, position });
            this.setNextUpSong();
            this.setMediaSessionData();
        } else {
            this.currentSong = null;
            current.set({ song: null, index: 0, position: 0 });
        }
    }

    async handleOpenedUrls(openedUrls: string) {
        window["openedUrls"] = null;
        console.log("handleOpenedUrls", openedUrls);
        const paths = openedUrls.split(",").map((p) => {
            // Strip file:// prefix
            if (p.startsWith("file://")) {
                p = p.slice(7);
            }
            return p.trim();
        });

        console.log("handleOpenedUrls paths", paths);

        const response = await invoke<ToImport>("scan_paths", {
            event: {
                paths: paths,
                recursive: false,
                process_albums: false,
                is_async: false,
            },
        });
        console.log("scan_paths response", response);
        if (response.songs) {
            this.shouldPlay = true;
            queue.set(response.songs);
        }
    }

    async play(isResume: boolean) {
        if (this.seek) {
            this.playSong(this.currentSong, this.seek);
        } else {
            this.isStopped = false;

            if (isResume) {
                await invoke("decode_control", {
                    event: {
                        decoding_active: true,
                    },
                });
            }

            this.onPlay();
        }
    }

    async pause() {
        invoke("decode_control", {
            event: {
                decoding_active: false,
            },
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
}

const audioPlayer = AudioPlayer.Instance;

export default audioPlayer;
