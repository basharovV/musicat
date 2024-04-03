import { convertFileSrc, invoke } from "@tauri-apps/api/tauri";
import type { ArtworkSrc, LastPlayedInfo, Song } from "src/App";
import { get } from "svelte/store";
import AudioSourceWorkletUrl from "../../WebRTCSourceWorker?url";
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
    streamInfo,
    volume
} from "../../data/store";
import { shuffleArray } from "../../utils/ArrayUtils";
import FileStreamSource from "./FileStreamSource";

import WebRTCReceiver from "./WebRTCReceiver";
import { appWindow } from "@tauri-apps/api/window";

const LOG_DATA = true;

const MAX_PACKETS = 50;

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

    mediaSource: MediaSource;

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
    currentStreamSrc = null; // To see if a stream needs to be loaded
    prevStreamSrc = null;
    fileStreams: { [key: string]: ReadableStream } = {};
    activeStream: string = null;
    sourceBuffer: SourceBuffer;
    readCancelled = false;
    readableStream: ReadableStream;
    gainNode: GainNode;
    fileStreamSource: FileStreamSource;

    private _audioContext: AudioContext;

    audioElement: HTMLAudioElement;
    audioElement2: HTMLAudioElement;

    // Nodes
    oscillatorNode: OscillatorNode;
    audioSourceNode: AudioWorkletNode;
    resamplerNode: AudioWorkletNode;
    private _audioSource: AudioBufferSourceNode;
    private _audioSource2: MediaElementAudioSourceNode;
    audioAnalyser: AnalyserNode;
    private _gainNode: GainNode;
    private _gainNode2: GainNode;

    buffered = 0;

    // Worklet stuff
    hasBufferReachedEnd = false;
    currentTimestamp = 0;
    audioWriter: AudioWriter;

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
        this.audioFile = new Audio();
        this.audioFile2 = new Audio();
        this.audioFile.crossOrigin = "anonymous";
        this.audioFile2.crossOrigin = "anonymous";
        this.mediaSource = new MediaSource();
        this.fileStreamSource = new FileStreamSource();

        this.mediaSource.addEventListener("sourceopen", async () =>
            this.onMediaSourceOpen()
        );

        this.webRTCReceiver = new WebRTCReceiver();

        // this.source = audioCtx.createMediaElementSource(this.audioFile);
        // this.source.connect(this.gainNode);

        seekTime.subscribe((time) => {
            // Tell Rust to play file with seek position
            if (this.currentSong) this.playCurrent(time);
        });

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

        volume.subscribe(async (vol) => {
            // if (this.gainNode) this.gainNode.gain.value = vol * 0.75;
            await invoke("volume_control", {
                event: {
                    volume: vol * 0.75
                }
            });
            localStorage.setItem("volume", String(vol));
        });

        // TODO: Remove
        appWindow.listen("file-samples", async (event) => {
            console.log("file-samples", event);
            console.log("audiosourcenode", this.audioSourceNode.port);
            if (this.audioSourceNode) {
                // Let the worker know about the total number of samples in this track
                this.audioSourceNode.port.postMessage({
                    type: "total-samples",
                    totalSamples: event.payload
                });
            }
        });

        appWindow.listen("timestamp", async (event) => {
            console.log("timestamp", event);
            playerTime.set(event.payload);
        });
        // this.setupAnalyserAudio();
    }

    /**
     * Only set up the audio context once we know the sample rate to set
     * i.e the track has been clicked
     * @param sampleRate The source file sample rate
     */
    async setupAudioContext(sampleRate) {
        await this.tearDownAudioContext();
        let AudioContext = window.AudioContext || window.webkitAudioContext;
        this._audioContext = new AudioContext({
            sampleRate,
            latencyHint: "playback"
        });

        console.log(
            "AudioPlayer::sample rate is " + this._audioContext.sampleRate
        );
        this.gainNode = this._audioContext.createGain();
        this.gainNode.gain.value = get(volume);
        this.gainNode.connect(this._audioContext.destination);
        await this.loadWorkletModules();
        this.setupAnalyserAudio();
    }

    async tearDownAudioContext() {
        // Tear down existing audio context
        if (
            this._audioContext?.state === "running" ||
            this._audioContext?.state === "suspended"
        ) {
            this.gainNode.disconnect();
            this.audioSourceNode.port.close();
            this.audioSourceNode.disconnect();
            await this._audioContext.close();
        }
    }

    async loadWorkletModules() {
        await this._audioContext.audioWorklet.addModule(AudioSourceWorkletUrl);
        console.log("loaded modules");
    }

    async workletSourceOpen(position: number) {
        console.log("player: sourceopen");

        let fileSrc = convertFileSrc(
            this.currentSong.path.replace("?", "%3F"),
            "stream"
        );
        if (fileSrc !== this.currentStreamSrc) {
            this.currentStreamSrc = fileSrc;
        }

        // if (this.gainNode) {
        //     this.gainNode.gain.value = 0; // Avoid clicks during switch
        // }

        // Set up audio nodes to be able to play PCM stream
        // Set up ring buffer
        let initialSetup = await this.startAudioNodes();
        if (!initialSetup) {
            this.audioSourceNode.port.postMessage({
                type: "reset"
            });
        }

        this.audioSourceNode.port.postMessage({
            type: "samplerate",
            sampleRate: this.currentSong.fileInfo.sampleRate,
            time: position
        });

        console.log(
            "audioplayer::datachannel::",
            this.webRTCReceiver.dataChannel?.readyState
        );
        // If the WebRTC receiver is ready to receive data, invoke the streamer
        if (this.webRTCReceiver.dataChannel?.readyState === "open") {
            this.webRTCReceiver.prepareForNewStream();
            invoke("stream_file", {
                event: {
                    path: this.currentSong.path,
                    seek: position,
                    file_info: this.currentSong.fileInfo
                }
            });
        }
    }

    async setupBuffers() {
        console.log("player::setupBuffers()");
    }

    async startAudioNodes(): Promise<boolean> {
        // If the sample rate of the file is different we should recreate the audio context to match it.
        let shouldRecreateContext =
            this._audioContext?.sampleRate !==
            this.currentSong.fileInfo.sampleRate;

        if (shouldRecreateContext)
            console.log(
                "player::switching sample rate from " +
                    this._audioContext?.sampleRate +
                    " to " +
                    this.currentSong.fileInfo.sampleRate
            );

        if (this._audioContext && !shouldRecreateContext) return false; // Already instantiated

        await this.setupAudioContext(this.currentSong.fileInfo.sampleRate);
        console.log("player::startAudioNodes()");

        this.audioSourceNode = new AudioWorkletNode(
            this._audioContext,
            "webrtc-receiver-processor",
            {
                numberOfInputs: 1,
                numberOfOutputs: 1,
                channelCount: 2,
                outputChannelCount: [2]
            }
        );
        this.audioSourceNode.port.start();

        if (this.audioSourceNode.port) {
            this.audioSourceNode.port.addEventListener(
                "message",
                async (ev: MessageEvent) => {
                    switch (ev.data.type) {
                        case "log":
                            console.log("audiosource::log", ev.data.text);
                            break;
                        case "buffer":
                            // console.log("audiosource::play");
                            streamInfo.update((s) => ({
                                ...s,
                                bufferedSamples: ev.data.bufferedSamples
                            }));

                            this.webRTCReceiver.doFlowControl(
                                ev.data.bufferedSamples /
                                    this.currentSong.fileInfo.sampleRate
                            );

                            break;
                        case "played":
                            // console.log("audiosource::play");
                            streamInfo.update((s) => ({
                                ...s,
                                playedSamples:
                                    s.playedSamples + ev.data.playedSamples
                            }));
                            break;
                        case "ended":
                            console.log("audiosource::ended");
                            await this.onEnded();
                            // Play next track
                            if (this.hasNext()) this.playNext();

                            break;
                        case "timestamp":
                            // console.log("audiosource::timestamp", ev.data);
                            // playerTime.set(ev.data.time);
                            streamInfo.update((s) => ({
                                ...s,
                                timestamp: ev.data.time,
                                sampleIdx: ev.data.sampleIdx
                            }));
                            break;
                    }
                }
            );

            this.audioSourceNode.onprocessorerror = (ev) => console.error(ev);

            // Append new messages to the box of incoming messages
            this.webRTCReceiver.onSampleData = (samples) => {
                // Get the ArrayBuffer

                // Get packet number
                this.packet_n++;
                // If packet_n is >= last packet received => send it to the processor
                // Otherwise drop it (to save time)
                if (true) {
                    if (LOG_DATA) {
                        // Save the time at which we receive data
                        performance.mark(`data-received-${this.packet_n}`);
                    }
                    // console.log("audio::onSampleData()", samples);

                    // Process data (tranfer of ownership)
                    let message = {
                        type: "packet",
                        data: samples
                    };
                    this.audioSourceNode.port.postMessage(message);
                    this.stats.packetReceivedCounter++;
                } else {
                    if (LOG_DATA) {
                        // Save the time at which we discard data
                        performance.mark(`data-discarded-${this.packet_n}`);
                    }
                    this.stats.packetDropCounter++;
                }
            };

            // this.resamplerNode.port.start();
            this.audioSourceNode.connect(this.gainNode);
            // this.resamplerNode.connect(this.gainNode);
        }
        return true;
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
    async appendChunk(chunk: Uint8Array) {
        if (this.sourceBuffer.updating) {
            this.sourceBuffer.addEventListener("updateend", () => {
                console.log("mse:onupdateend:appendBuffer");
                this.sourceBuffer.appendBuffer(chunk);
            });
        } else {
            console.log("mse:appendBuffer");
            this.sourceBuffer.appendBuffer(chunk);
        }
    }

    lastChunkDropTime = 0;

    async onTimeUpdate(time: number) {
        console.log("AudioPlayer::player time: " + time);
        console.log("FileStreamer (bytes)", this.fileStreamSource.bytePos);
        console.log("FileStreamer (time)", this.fileStreamSource.timePos);

        lastPlayedInfo.set({
            songId: this.currentSong.id,
            position: time
        });
        playerTime.set(time);

        // Reached the end of the playlist, exit here
        if (this.currentSongIdx === this.playlist.length - 1) {
            return;
        }

        if (
            this.fileStreamSource.timePos - time < 5 &&
            !this.fileStreamSource.isFetchingChunk
        ) {
            // Fetch the next chunk
            const chunk = await this.fileStreamSource.getNextChunk();
            this.audioSourceNode.port.postMessage({
                type: "ADD_SAMPLES",
                payload: { samples: chunk.channelData }
            });
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

    playCurrent(seek: number = 0) {
        this.playSong(this.playlist[this.currentSongIdx], seek);
    }

    hasNext() {
        return this.currentSongIdx + 1 < this.playlist.length;
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
    async webAudioPlaySong(song: Song, position = 0, play = true) {
        console.log("webaudio: playSong", console.trace());
        if (song) {
            // this.pause();
            currentSong.set(song);
            this.currentSong = song;

            playerTime.set(this.getCurrentAudioFile().currentTime);

            if (play) {
                await this.workletSourceOpen(position);
                await this.play();
            }
            this.shouldPlay = play;
            let newCurrentSongIdx = this.playlist.findIndex(
                (s) => s.id === this.currentSong?.id
            );
            if (newCurrentSongIdx === -1) {
                newCurrentSongIdx = 0;
            }
            currentSongIdx.set(newCurrentSongIdx);
            // this.setNextUpSong();
            this.setMediaSessionData();
            this.incrementPlayCounter(song);

            lastPlayedInfo.set({
                songId: this.currentSong.id,
                position: 0
            });
        }
    }

    async playSong(song: Song, position = 0, play = true) {
        console.log("playSong", song);
        this.webAudioPlaySong(song, position, play);
        return;
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

    async play(isResume: boolean) {
        if (this.audioSourceNode) {
            if (this._audioContext?.state === "suspended")
                await this._audioContext.resume();
            this.gainNode.gain.setTargetAtTime(
                get(volume),
                this._audioContext.currentTime,
                0.015
            );

            // this.oscillatorNode.start();
            if (isResume) {
                invoke("decode_control", {
                    event: {
                        decoding_active: true
                    }
                });
            }
            // Clear intervals first
            if (this.ticker) clearInterval(this.ticker);
            if (this.flowControlSendInterval)
                clearInterval(this.flowControlSendInterval);

            this.ticker = setInterval(() => {
                this.audioSourceNode.port.postMessage({
                    type: "timestamp-query"
                });
            }, 500);
            this.flowControlSendInterval = setInterval(async () => {
                // Send the bitrate to the sender for flow control
                let receiveRate = get(streamInfo).receiveRate;
                await invoke("flow_control", {
                    event: {
                        client_bitrate: receiveRate
                    }
                });
            }, 500);
            this.onPlay();
        }
    }

    async pause() {
        // this._audioSource.stop();
        if (this._audioContext && this.gainNode) {
            this.gainNode.gain.setTargetAtTime(
                0,
                this._audioContext.currentTime,
                0.015
            );
        }
        setTimeout(async () => {
            this._audioContext?.state === "running" &&
                (await this._audioContext.suspend());
        }, 0.015);

        // this.oscillatorNode.stop();
        invoke("decode_control", {
            event: {
                decoding_active: false
            }
        });
        if (this.ticker) clearInterval(this.ticker);
        if (this.flowControlSendInterval)
            clearInterval(this.flowControlSendInterval);
        this.onPause();
    }

    async onEnded() {
        this._audioContext?.state === "running" &&
            (await this._audioContext.suspend());
        if (this.ticker) clearInterval(this.ticker);
        if (this.flowControlSendInterval)
            clearInterval(this.flowControlSendInterval);
        this.gainNode.gain.setTargetAtTime(
            0,
            this._audioContext.currentTime,
            0.015
        );
    }

    togglePlay() {
        if (get(isPlaying)) {
            this.pause();
        } else {
            this.play(true);
        }
    }

    async restoreLastPlayed(lastPlayed: LastPlayedInfo) {
        const song = await db.songs.get(lastPlayed.songId);
        this.playSong(song, lastPlayed.position, false);
    }

    /**
     * Connect the nodes together, so sound is flowing
     * into the analyser node from the source node
     */
    setupAnalyserAudio() {
        // this.audioElement = this.getCurrentAudioFile();
        // this.audioElement2 = this.getOtherAudioFile();
        // let AudioContext = window.AudioContext || window.webkitAudioContext;
        // this._audioContext = new AudioContext({ sampleRate: 48000 });
        // this._audioSource = this._audioContext.createMediaElementSource(
        //     this.audioElement
        // );

        // this._audioSource2 = this._audioContext.createMediaElementSource(
        //     this.audioElement2
        // );
        this.audioAnalyser = new AnalyserNode(this._audioContext, {
            fftSize: 2048,
            smoothingTimeConstant: 0.9
        });
        // this._audioSource.mediaElement.volume = get(volume);
        // this._audioSource2.mediaElement.volume = get(volume);
        // Source -> Gain -> Analyser -> Destination (master)
        // this._audioSource.connect(this._gainNode); // DONE WHEN WORKLET IS AVAILABLE
        this.gainNode.connect(this.audioAnalyser);
        this.audioAnalyser.connect(this._audioContext.destination);
    }
    doPlay = (number) => {
        console.log("doplay ", number);
        if (number === 1) {
            this._audioSource.start();
        }
    };
    doPause = (number) => {
        console.log("dopause ", number);
        if (number === 1) {
            this._audioSource.stop();
        }
    };
}

const audioPlayer = AudioPlayer.Instance;

export default audioPlayer;
