import type { AudioSourceNodeOptions } from "./App";

/**
 * Purpose:
 * - All-in-one source node
 *
 * Features:
 * - Buffer can be dynamically updated
 *   - Use case: Decode first block, then write here,
 *     and can be used for playing already!
 *     Afterwards, next blocks can be written and
 *     no need to disconnect/reconnect nodes
 * - Support buffer looping.
 *   - NOTE: currently loop end = buffer end,
 *     but support can be added for custom loop end in the future
 * - Support seeking to certain time
 * - Query accurate playback timestamp
 */
class AudioSourceNode extends AudioWorkletProcessor {
    src = null;

    /** segments of multi-channel samples */
    samples: Float32Array[][];
    samples2: Float32Array[][]; // switch between them
    sample2Offset: number = 0; // length of samples 1

    channels = 2;

    /** list of offsets, each for each segment in this.samples */
    samplesOffsets: Array<number> = [0];

    trackStates;
    trackDescriptions;

    shouldLoop = false;

    totalSamples = 0;
    sampleRate;
    loopStartSample = 0;

    /** Sample at playhead */
    _bufferHead = 0;

    isFinished = false;
    /**
     *
     * @param {AudioSourceNodeOptions} options
     */
    constructor(options: AudioSourceNodeOptions) {
        super();
        this.log("starting worklet", options);
        this.samples = options?.processorOptions?.initialSamples;
        this.channels = this.samples.length;
        this.samplesOffsets = [0];

        this.trackStates = options?.processorOptions?.trackStates || {};
        this.trackDescriptions =
            options?.processorOptions?.trackDescriptions || [];
        this.shouldLoop = options?.processorOptions?.shouldLoop || false;

        // Samples for this first chunk
        this.totalSamples = options?.processorOptions?.totalSamples || 0;
        this.log("samples len", this.samples[0].length);
        this.log("example smpl ", this.samples[0][128]);

        this.sampleRate = options.processorOptions.inputSampleRate;
        this.loopStartSample = options?.processorOptions?.loopStartSample || 0;

        /** @type {number} */
        this._bufferHead = 0;

        this.port.onmessage = (event) => {
            switch (event.data.type) {
                case "ADD_SAMPLES": {
                    this.log(
                        "[AudioSourceNode] ADD_SAMPLES",
                        event.data.payload.samples[0].length
                    );
                    const samples: Float32Array[][] =
                        event.data.payload.samples;

                    this.samples2 = samples;
                    this.sample2Offset = this.samples[0].length; // When the process() function gets to here, it will start reading from samples2

                    this.totalSamples = this.totalSamples + samples[0].length;

                    this.log(
                        "[AudioSourceNode] Play next chunk from ",
                        this.samples[0].length,
                        "until",
                        this.totalSamples,
                        ""
                    );
                    break;
                }
                case "UPDATE_TRACK_STATES": {
                    this.trackStates =
                        /** @type {AudioPlayerTrackStates} */ event.data.payload.trackStates;
                    console.log(
                        "[AudioSourceNode] UPDATE_TRACK_STATES",
                        this.trackStates
                    );
                    break;
                }
                case "SEEK": {
                    const playbackTimeInS =
                        /** @type {number} */ event.data.payload
                            .playbackTimeInS;
                    this._bufferHead = Math.floor(
                        playbackTimeInS * this.sampleRate
                    );
                    console.log("[AudioSourceNode] SEEK", playbackTimeInS);
                    break;
                }
                case "UPDATE_SHOULD_LOOP": {
                    this.shouldLoop =
                        /** @type {boolean} */ event.data.payload.shouldLoop;
                    console.log(
                        "[AudioSourceNode] UPDATE_SHOULD_LOOP",
                        this.shouldLoop
                    );
                    break;
                }
                case "TIMESTAMP_QUERY": {
                    !this.isFinished &&
                        this.port.postMessage({
                            type: "TIMESTAMP_REPLY",
                            payload: {
                                timestamp: this._bufferHead / this.sampleRate
                            }
                        });
                    break;
                }
            }
        };
    }

    /**
     *
     * @param {number} s sample index
     * @returns {number} segment index
     */
    getSegmentIndex(s) {
        // https://en.wikipedia.org/wiki/Binary_search_algorithm#Alternative_procedure
        let l = 0,
            r = this.samplesOffsets.length - 1;
        while (l !== r) {
            let mid = Math.ceil((l + r) / 2);
            if (this.samplesOffsets[mid] > s) {
                r = mid - 1;
            } else {
                l = mid;
            }
        }
        return l;
    }

    log(...statements) {
        this.port.postMessage({
            type: "LOGGER",
            payload: [...statements].join(", ")
        });
    }

    logOnce = false;

    /**
     *
     * @param {Array<Array<Float32Array>>} _inputs
     * @param {Array<Array<Float32Array>>} outputs
     * @param {Object} _parameters
     * @returns {boolean}
     */
    process(_inputs, outputs, _parameters) {
        const output = outputs[0];
        let absoluteSampleIndex = this._bufferHead;

        // Iterate through channels
        for (let ch = 0; ch <= this.channels - 1; ch++) {
            // Iterate through at most 128 samples (copy over 128 samples to output)
            for (let smpl = 0; smpl < 128; smpl++) {
                if (absoluteSampleIndex >= this.totalSamples) {
                    // We've reached the end, just output 0
                    output[ch][smpl] = 0;
                    continue;
                } else {
                    if (absoluteSampleIndex < 128) {
                        // this.log("output smpl ", smpl, this.samples[ch][absoluteSampleIndex + smpl]);
                    }

                    if (
                        this.samples2 &&
                        absoluteSampleIndex + smpl >= this.sample2Offset
                    ) {
                        if (!this.logOnce) {
                            this.log(
                                "[AudioSourceNode] Playing samples2 from ",
                                absoluteSampleIndex - this.sample2Offset + smpl,
                                absoluteSampleIndex + smpl >=
                                    this.sample2Offset,
                                absoluteSampleIndex + smpl,
                                this.samples[0].length
                            );
                            this.logOnce = true;
                        }

                        // read from sample 2 instead
                        // this.log("offset", absoluteSampleIndex - this.sample2Offset + smpl) // Should start from 0)
                        output[ch][smpl] =
                            this.samples2[ch][
                                absoluteSampleIndex - this.sample2Offset + smpl // Should start from 0
                            ];
                    } else {
                        output[ch][smpl] =
                            this.samples[ch][absoluteSampleIndex + smpl];
                    }
                }
            }
        }

        absoluteSampleIndex += 128;

        if (absoluteSampleIndex >= this.totalSamples) {
            this.isFinished = true;
            if (this.shouldLoop) {
                absoluteSampleIndex = this.loopStartSample;
                this.port.postMessage({
                    type: "BUFFER_LOOPED"
                });
            } else {
                this.port.postMessage({
                    type: "BUFFER_ENDED"
                });
            }
        }

        this._bufferHead = absoluteSampleIndex;
        return true;
    }
}

/**
 *
 * @param {number} value
 * @param {number} min
 * @param {number} max
 * @returns {number}
 */
function clamp(value, min, max) {
    return value <= min ? min : value >= max ? max : value;
}

registerProcessor("audio-source-processor", AudioSourceNode);
