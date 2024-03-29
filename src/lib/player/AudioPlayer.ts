import type { ArtworkSrc, LastPlayedInfo, Song } from "src/App";
import { get } from "svelte/store";
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
    queriedSongs,
    seekTime,
    shuffledPlaylist,
    volume
} from "../../data/store";
import { shuffleArray } from "../../utils/ArrayUtils";
import { db } from "../../data/db";
import { convertFileSrc } from "@tauri-apps/api/tauri";

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

    private constructor() {
        // let AudioContext = window.AudioContext || window.webkitAudioContext;
        // const audioCtx: AudioContext = new AudioContext();
        // this.gainNode = audioCtx.createGain();
        // this.gainNode.gain.value = 1;
        // this.gainNode.connect(audioCtx.destination);

        this.audioFile = new Audio();
        this.audioFile2 = new Audio();
        this.audioFile.crossOrigin = "anonymous";
        this.audioFile2.crossOrigin = "anonymous";
        // this.source = audioCtx.createMediaElementSource(this.audioFile);
        // this.source.connect(this.gainNode);

        seekTime.subscribe((time) => {
            if (this.getCurrentAudioFile()) {
                this.getCurrentAudioFile().currentTime = time;
                playerTime.set(time);
            }
        });

        this.audioFile.addEventListener(
            "pause",
            (() => this.onPause(1)).bind(this)
        );
        this.audioFile.addEventListener(
            "play",
            (() => this.onPlay(1)).bind(this)
        );
        this.audioFile.addEventListener(
            "ended",
            (async () => this.onEnded(1)).bind(this)
        );
        this.audioFile.addEventListener(
            "timeupdate",
            (() => this.onTimeUpdate(1)).bind(this)
        );

        this.audioFile2.addEventListener(
            "pause",
            (() => this.onPause(2)).bind(this)
        );
        this.audioFile2.addEventListener(
            "play",
            (() => this.onPlay(2)).bind(this)
        );
        this.audioFile2.addEventListener(
            "ended",
            (() => this.onEnded(2)).bind(this)
        );
        this.audioFile2.addEventListener(
            "timeupdate",
            (async () => this.onTimeUpdate(1)).bind(this)
        );

        // volume.set(this.audioFile.volume);

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
        this.setupAnalyserAudio();
    }

    shuffle() {
        const shuffled = shuffleArray(this.playlist);
        shuffledPlaylist.set(shuffled);
        this.playlist = shuffled;
        isShuffleEnabled.set(true);
    }

    unshuffle() {
        this.playlist = get(playlist);
        shuffledPlaylist.set([]);
        isShuffleEnabled.set(false);
    }

    getCurrentAudioFile() {
        if (this.currentAudioFile === 1) return this.audioFile;
        return this.audioFile2;
    }

    getOtherAudioFile() {
        if (this.currentAudioFile === 1) return this.audioFile2;
        return this.audioFile;
    }

    switchAudioFile() {
        if (this.currentAudioFile === 1) {
            this.currentAudioFile = 2;
        } else {
            this.currentAudioFile = 1;
        }
    }

    setVolume(vol) {
        // this.audioFile.volume = volume;
        volume.set(vol);
    }

    onPlay(player: number) {
        if (player === this.currentAudioFile) {
            isPlaying.set(true);
        }
        if (navigator.mediaSession)
            navigator.mediaSession.playbackState = "playing";
    }

    onPause(player: number) {
        if (player === this.currentAudioFile) {
            isPlaying.set(false);
        }
        if (navigator.mediaSession)
            navigator.mediaSession.playbackState = "paused";
    }

    onEnded(player: number) {
        if (player === this.currentAudioFile) {
            console.log("ENDED!");
            isPlaying.set(false);
            console.log("callback", this.isFinishedCallback);
            this.isFinishedCallback && this.isFinishedCallback();
            if (!this.isRunningTransition) this.onTimeUpdate(player, true);
        } else {
            // The other player has finished playing
        }
    }

    async onTimeUpdate(player: number, ended = false) {
        // console.log("player 1 time: " + this.audioFile.currentTime);
        // console.log("player 2 time: " + this.audioFile2.currentTime);

        const currentTime =
            this.currentAudioFile === 1
                ? this.audioFile.currentTime
                : this.audioFile2.currentTime;
        const duration =
            this.currentAudioFile === 1
                ? this.audioFile.duration
                : this.audioFile2.duration;

        // Reached the end of the playlist, exit here
        if (this.currentSongIdx === this.playlist.length - 1) {
            return;
        }

        /*
         * Playing a short sample - don't try to do gapless
         * because the sample won't play in its entirety
         */
        if (duration < 30 && !this.isRunningTransition && ended) {
            this.currentSongIdx = this.currentSongIdx + 1;
            currentSongIdx.set(get(currentSongIdx) + 1);
            playerTime.set(0);
            currentSong.set(get(nextUpSong));
            this.currentSong = get(nextUpSong);
            this.setMediaSessionData();
            this.isRunningTransition = false;
            this.playCurrent();
        } else if (
            /*
             * For long audios (songs), attempt to do gapless.
             * It's not perfect since we don't have control over the
             * buffer and sample-level precision.
             */
            duration > 30 &&
            currentTime > duration - 1.1 &&
            !this.isRunningTransition
        ) {
            this.isRunningTransition = true;

            const chunk = 0.3;
            let diff = duration - currentTime;
            console.log("2 SECS LEFT! STARTING TRANSITION...");
            console.log("diff", diff);
            console.log("duration", duration);
            console.log("currentTime", currentTime);

            // setTimeout(async () => {
            //     this.getOtherAudioFile().volume = 0;
            //     this.getCurrentAudioFile().volume = get(volume);
            // }, (diff - 0.177) * 1000);

            this.queueGapless(diff);
        }
    }

    async queueGapless(diff) {
        const playDelay = 0.1;
        setTimeout(
            async () => {
                this.pause(true);
                this.switchAudioFile();
                console.log("currentAudioFile", this.currentAudioFile);
                console.log("currentAudioFile switched", this.currentAudioFile);
                currentSongIdx.set(get(currentSongIdx) + 1);
                playerTime.set(0);
                currentSong.set(get(nextUpSong));
                this.currentSong = get(nextUpSong);
                this.setMediaSessionData();
                this.play(false);
                this.setNextUpSong();
                this.isRunningTransition = false;
            },
            (diff - 0.41) * 1000
        );
    }

    playCurrent() {
        this.playSong(this.playlist[this.currentSongIdx]);
    }

    playNext() {
        console.log("currentidx", this.currentSongIdx);
        currentSongIdx.set(get(currentSongIdx) + 1);
        console.log("currentidx", this.currentSongIdx);
        this.playSong(this.playlist[this.currentSongIdx]);
    }

    playPrevious() {
        currentSongIdx.set(get(currentSongIdx) - 1);
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
        if (this.getCurrentAudioFile().src) {
            if (this.currentSongIdx + 1 < this.playlist.length) {
                const nextSong = this.playlist[this.currentSongIdx + 1];
                nextUpSong.set(nextSong);

                // Set up gapless playback
                this.getOtherAudioFile().src = convertFileSrc(
                    nextSong.path.replace("?", "%3F")
                );
                this.getOtherAudioFile().preload = "auto";
                this.getOtherAudioFile().load();
                this.getOtherAudioFile().currentTime = 0.00001;
            } else {
                nextUpSong.set(null);
            }
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
        console.log("playSong", song);
        if (this.getCurrentAudioFile() && song) {
            this.pause();
            this.getCurrentAudioFile().currentTime = 0;
            playerTime.set(this.getCurrentAudioFile().currentTime);
            this.getCurrentAudioFile().src = convertFileSrc(
                song.path.replace("?", "%3F")
            );
            play && this.play();
            currentSong.set(song);
            this.currentSong = song;
            let newCurrentSongIdx = this.playlist.findIndex(
                (s) => s.id === this.currentSong?.id
            );
            if (newCurrentSongIdx === -1) {
                newCurrentSongIdx = 0;
            }
            currentSongIdx.set(newCurrentSongIdx);
            this.setNextUpSong();
            this.setMediaSessionData();
            this.incrementPlayCounter(song);
            if (position > 0) {
                seekTime.set(position);
            }
            lastPlayedInfo.set({
                songId: this.currentSong.id,
                position: 0
            });
        }
    }

    play(other: boolean = false) {
        if (other) {
            this.doPlay(this.currentAudioFile === 1 ? 2 : 1);
            this.ticker = setInterval(() => {
                lastPlayedInfo.set({
                    songId: this.currentSong.id,
                    position: this.getCurrentAudioFile().currentTime
                });
                playerTime.set(this.getCurrentAudioFile().currentTime);
            }, 1000);
        } else {
            if (this.getCurrentAudioFile()) {
                this.doPlay(this.currentAudioFile);

                this.ticker = setInterval(() => {
                    lastPlayedInfo.set({
                        songId: this.currentSong.id,
                        position: this.getCurrentAudioFile().currentTime
                    });
                    playerTime.set(this.getCurrentAudioFile().currentTime);
                }, 1000);
            }
        }
    }

    pause(other: boolean = false) {
        if (other && this.getOtherAudioFile()) {
            this.doPause(this.currentAudioFile === 1 ? 2 : 1);
        } else {
            if (this.getCurrentAudioFile()) {
                this.doPause(this.currentAudioFile);
            }
        }
    }

    togglePlay() {
        if (get(isPlaying)) {
            this.pause();
        } else {
            this.play();
        }
    }

    async restoreLastPlayed(lastPlayed: LastPlayedInfo) {
        const song = await db.songs.get(lastPlayed.songId);
        this.playSong(song, lastPlayed.position, false);
    }

    private _audioContext: AudioContext;

    audioElement: HTMLAudioElement;
    audioElement2: HTMLAudioElement;

    // Nodes
    private _audioSource: MediaElementAudioSourceNode;
    private _audioSource2: MediaElementAudioSourceNode;
    audioAnalyser: AnalyserNode;
    private _gainNode: GainNode;
    private _gainNode2: GainNode;

    /**
     * Connect the nodes together, so sound is flowing
     * into the analyser node from the source node
     */
    setupAnalyserAudio() {
        this.audioElement = this.getCurrentAudioFile();
        this.audioElement2 = this.getOtherAudioFile();
        let AudioContext = window.AudioContext || window.webkitAudioContext;
        this._audioContext = new AudioContext({ sampleRate: 48000 });
        this._audioSource = this._audioContext.createMediaElementSource(
            this.audioElement
        );

        this._audioSource2 = this._audioContext.createMediaElementSource(
            this.audioElement2
        );
        this.audioAnalyser = new AnalyserNode(this._audioContext, {
            fftSize: 2048,
            smoothingTimeConstant: 0.9
        });
        this._gainNode = new GainNode(this._audioContext, {
            gain: get(volume)
        });
        this._gainNode2 = new GainNode(this._audioContext, {
            gain: get(volume)
        });
        this._audioSource.mediaElement.volume = get(volume);
        this._audioSource2.mediaElement.volume = get(volume);
        // Source -> Gain -> Analyser -> Destination (master)
        this._audioSource.connect(this._gainNode);
        this._audioSource2.connect(this._gainNode2);
        this._gainNode.connect(this.audioAnalyser);
        this._gainNode2.connect(this.audioAnalyser);
        this.audioAnalyser.connect(this._audioContext.destination);

        volume.subscribe((vol) => {
            this._gainNode.gain.value = vol * 0.75;
            this._gainNode2.gain.value = vol * 0.75;
            localStorage.setItem("volume", String(vol));
        });
    }
    doPlay = (number) => {
        console.log('doplay ', number);
        if (number === 1) {
            this._audioSource.mediaElement.play();
        } else {
            this._audioSource2.mediaElement.play();
        }
    };
    doPause = (number) => {
        console.log('dopause ', number);
        if (number === 1) {
            this._audioSource.mediaElement.pause();
        } else {
            this._audioSource2.mediaElement.pause();
        }
    };
}

const audioPlayer = AudioPlayer.Instance;

export default audioPlayer;
