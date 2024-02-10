import type {
    ArtworkSrc,
    LastPlayedInfo,
    Song,
    GetFileSizeResponse
} from "src/App";
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
import { convertFileSrc, invoke } from "@tauri-apps/api/tauri";
import FileStreamSource from "./FileStreamSource";
import AudioSourceWorkletUrl from "../../AudioSourceWorker?url";
import AudioResamplerWorkletUrl from "../../AudioResamplerWorker?url";
import LibSamplerateWorkletUrl from "@alexanderolsen/libsamplerate-js/dist/libsamplerate.worklet?url";
import {
    AudioWriter,
    ParameterReader,
    ParameterWriter,
    RingBuffer
} from "./ringbuf";
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
    paramReader: ParameterReader;
    paramWriter: ParameterWriter;

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

        volume.subscribe((vol) => {
            if (this.gainNode) this.gainNode.gain.value = vol * 0.75;
            localStorage.setItem("volume", String(vol));
        });

        // this.setupAnalyserAudio();
    }

    /**
     * Only set up the audio context once we know the sample rate to set
     * i.e the track has been clicked
     * @param sampleRate The source file sample rate
     */
    async setupAudioContext(sampleRate) {
        this.tearDownAudioContext();
        let AudioContext = window.AudioContext || window.webkitAudioContext;
        this._audioContext = new AudioContext({
            sampleRate,
            latencyHint: "interactive"
        });

        console.log(
            "AudioPlayer::sample rate is " + this._audioContext.sampleRate
        );
        this.gainNode = this._audioContext.createGain();
        this.gainNode.gain.value = get(volume);
        // Buffer which will hold the chunk to play

        // this._audioSource = this._audioContext.createBufferSource();
        // this._audioSource.connect(this.gainNode);
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
        await this._audioContext.audioWorklet.addModule(
            LibSamplerateWorkletUrl
        );
        await this._audioContext.audioWorklet.addModule(
            AudioResamplerWorkletUrl
        );
        console.log("loaded modules");
    }

    getActiveStream() {
        return this.fileStreams[this.currentSong.file];
    }

    setActiveStream(readableStream: ReadableStream) {
        this.fileStreams[this.currentSong.file] = readableStream;
        this.activeStream = this.currentSong.file;
    }

    removeStream(streamId: string) {
        delete this.fileStreams[streamId];
    }

    abortStreamIfNecessary(
        stream,
        reader,
        streamId,
        sourceBuffer: SourceBuffer
    ) {
        if (streamId !== this.activeStream) {
            console.log("ABORTING STREAM");

            reader.releaseLock();
            stream.cancel();
            console.log("Stream changed");
            this.mediaSource.endOfStream();
            this.removeStream(streamId);
            return true;
        }
        return false;
    }

    async onMediaSourceOpen() {
        console.log("mse: sourceopen");
        if (!this.currentSong.file.match(/\.(mp3)$/i)) {
            return; // Only mp3 support for now
        }
        if (MediaSource.isTypeSupported("audio/mpeg")) {
            console.info("Mimetype is", "audio/mpeg");
            // TODO ...
        } else {
            console.error("Mimetype not supported", "audio/mpeg");
            return;
        }

        // Reset
        if (this.currentStreamSrc !== this.prevStreamSrc) {
            // Remove all source buffers
            for (let buffer of this.mediaSource.activeSourceBuffers) {
                this.mediaSource.removeSourceBuffer(buffer);
            }
            this.removeStream(this.activeStream);

            // Add source buffer
            this.sourceBuffer = this.mediaSource.addSourceBuffer("audio/mpeg");
            this.sourceBuffer.onerror = function (ev) {
                console.error("SOURCE BUFFER err", ev);
            };
        }

        // Fetch first chunk
        const firstChunk = await this.fileStreamSource.queueFile(
            this.currentStreamSrc,
            this.currentSong.path,
            "audio/mpeg"
        );
        await this.appendChunk(firstChunk);
    }

    async webAudioSourceOpen() {
        console.log("webaudio: sourceopen");
        if (!this.currentSong.file.match(/\.(mp3)$/i)) {
            return; // Only mp3 support for now
        }
        if (MediaSource.isTypeSupported("audio/mpeg")) {
            console.info("Mimetype is", "audio/mpeg");
            // TODO ...
        } else {
            console.error("Mimetype not supported", "audio/mpeg");
            return;
        }

        await this.fileStreamSource.reset();

        // Fetch first chunk
        const firstChunk = await this.fileStreamSource.queueFile(
            this.currentStreamSrc,
            this.currentSong.path,
            "audio/mpeg"
        );
        console.log("firstChunk", firstChunk);
        if (firstChunk.channelData) {
            const buffer = new AudioBuffer({
                numberOfChannels: firstChunk.channelData.length,
                length: firstChunk.samplesDecoded,
                sampleRate: firstChunk.sampleRate
            });
            this.buffered = buffer.length / buffer.sampleRate; // in seconds
            for (let i = 0; i <= firstChunk.channelData.length - 1; i++) {
                buffer.getChannelData(i).set(firstChunk.channelData[i]);
            }

            if (!this._audioSource.buffer) {
                // Set the buffer in the AudioBufferSourceNode
                this._audioSource.buffer = buffer;
            }

            this._audioSource.onended = async () => {
                console.log("White noise finished.");
            };
        }
    }
    async workletSourceOpen() {
        console.log("worklet: sourceopen");
        if (!this.currentSong.file.match(/\.(mp3)$/i)) {
            return; // Only mp3 support for now
        }
        if (MediaSource.isTypeSupported("audio/mpeg")) {
            console.info("Mimetype is", "audio/mpeg");
            // TODO ...
        } else {
            console.error("Mimetype not supported", "audio/mpeg");
            return;
        }

        if (this.gainNode) {
            this.gainNode.gain.value = 0; // Avoid clicks during switch
        }

        // Fetch first chunk
        const firstChunk = await this.fileStreamSource.queueFile(
            this.currentStreamSrc,
            this.currentSong.path,
            "audio/mpeg"
        );
        console.log("firstChunk", firstChunk);
        if (firstChunk?.channelData) {
            // const buffer = new AudioBuffer({
            //     numberOfChannels: firstChunk.channelData.length,
            //     length: firstChunk.samplesDecoded,
            //     sampleRate: firstChunk.sampleRate
            // });
            // this.buffered = buffer.length / buffer.sampleRate; // in seconds
            // for (let i = 0; i <= firstChunk.channelData.length - 1; i++) {
            //     buffer.getChannelData(i).set(firstChunk.channelData[i]);
            // }

            await this.startAudioNodes(firstChunk);
        }
    }

    async startAudioNodes(chunk) {
        await this.setupAudioContext(chunk.sampleRate);
        console.log("worklet:module loaded");
        // this.oscillatorNode = this._audioContext.createOscillator();

        // this.oscillatorNode.type = "square";
        // this.oscillatorNode.frequency.setValueAtTime(
        //     440,
        //     this._audioContext.currentTime
        // ); // value in hertz
        // this.oscillatorNode.connect(this.gainNode);
        // 50ms of buffer, increase in case of glitches
        // const sab = RingBuffer.getStorageForCapacity(
        //     this._audioContext.sampleRate / 20,
        //     Float32Array
        // );
        // const rb = new RingBuffer(sab, Float32Array);
        // this.audioWriter = new AudioWriter(rb);

        // const sab2 = RingBuffer.getStorageForCapacity(31, Uint8Array);
        // const rb2 = new RingBuffer(sab2, Uint8Array);
        // this.paramWriter = new ParameterWriter(rb2);

        this.audioSourceNode = new AudioWorkletNode(
            this._audioContext,
            "audio-source-processor",
            {
                numberOfInputs: 0,
                numberOfOutputs: 1,
                outputChannelCount: [2],
                processorOptions: {
                    initialSamples: chunk.channelData,
                    loopStartSample: 0,
                    totalSamples: chunk.samplesDecoded,
                    inputSampleRate: chunk.sampleRate,
                    shouldLoop: false,
                    trackDescriptions: [{ numberChannels: 2 }],
                    trackStates: [true]
                }
            }
        );
        this.resamplerNode = new AudioWorkletNode(
            this._audioContext,
            "resampler-processor",
            {
                numberOfInputs: 1,
                numberOfOutputs: 1,
                outputChannelCount: [2],
                processorOptions: {
                    inputSampleRate: chunk.sampleRate,
                    outputSampleRate: this._audioContext.sampleRate,
                    trackDescriptions: [{ numberChannels: 2 }]
                }
            }
        );
        if (this.audioSourceNode.port) {
            this.audioSourceNode.port.addEventListener(
                "message",
                (ev: MessageEvent) => {
                    switch (ev.data.type) {
                        case "BUFFER_LOOPED": {
                            console.log("[AudioPlayer]", ev.data.type);
                            break;
                        }
                        case "BUFFER_ENDED": {
                            console.log("[AudioPlayer]", ev.data.type);
                            this.pause();
                            this.hasBufferReachedEnd = true;
                            break;
                        }
                        case "LOGGER": {
                            console.log("[AudioPlayer]", ev.data.payload);
                            break;
                        }
                        case "TIMESTAMP_REPLY": {
                            console.log(
                                "[AudioPlayer]",
                                ev.data.payload.timestamp
                            );
                            this.onTimeUpdate(
                                ev.data.payload.timestamp as number
                            );
                            break;
                        }
                    }
                }
            );

            if (this.resamplerNode.port) {
                this.resamplerNode.port.addEventListener(
                    "message",
                    (ev: MessageEvent) => {
                        switch (ev.data.type) {
                            case "LOGGER": {
                                console.log(
                                    "[AudioResampler]",
                                    ev.data.payload
                                );
                                break;
                            }
                        }
                    }
                );
            }
            this.audioSourceNode.port.start();
            // this.resamplerNode.port.start();
            this.audioSourceNode.connect(this.gainNode);
            // this.resamplerNode.connect(this.gainNode);
        }
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
    async webAudioPlaySong(song: Song, position = 0, play = true) {
        console.log("webaudio: playSong", song, play);
        if (this.getCurrentAudioFile() && song) {
            this.pause();
            this.getCurrentAudioFile().currentTime = 0;

            currentSong.set(song);
            this.currentSong = song;

            playerTime.set(this.getCurrentAudioFile().currentTime);

            let fileSrc = convertFileSrc(
                this.currentSong.path.replace("?", "%3F"),
                "stream"
            );
            console.log("YO ARE WER HERE");
            if (fileSrc !== this.currentStreamSrc) {
                this.currentStreamSrc = fileSrc;
                // this.getCurrentAudioFile().src = URL.createObjectURL(
                //     this.mediaSource
                // );
            }

            if (play) {
                await this.workletSourceOpen();
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
            if (position > 0) {
                seekTime.set(position);
            }
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

    async play() {
        if (this._audioContext?.state === "suspended" && this.audioSourceNode) {
            this.audioSourceNode.onprocessorerror = (ev) => console.error(ev);
            await this._audioContext.resume();
            // this.oscillatorNode.start();
            this.ticker = setInterval(() => {
                this.audioSourceNode.port.postMessage({
                    type: "TIMESTAMP_QUERY"
                });
            }, 500);
            this.onPlay();
        }
    }

    async pause() {
        // this._audioSource.stop();
        this._audioContext?.state === "running" &&
            (await this._audioContext.suspend());
        // this.oscillatorNode.stop();
        this.onPause();
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
