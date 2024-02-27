import type { AudioSourceNodeOptions } from "./App";
import RingBuffer from "./lib/player/ringbuf";

function convertRange(value, r1, r2) {
    return ((value - r1[0]) * (r2[1] - r2[0])) / (r1[1] - r1[0]) + r2[0];
}

/**
 * Take an array of bytes (Uint8array) and convert it to a Float32Array for processing
 * @param data The binary data
 */
function convertUInt8ToFloat32(buffer: ArrayBuffer) {
    // let uint8Array = new Uint8Array(buffer);

    var float32Array = new Float32Array(buffer); // Assuming each sample is 2 bytes (16 bits)

    // [28, 48,     18, 40,    28, 48,         18, 40]
    // [smpl1 L     smpl1 L,   smpl1 R         smpl1 R]
    // To:
    // [-0.4, 0.3, -0.53, 0.22] // interleaved samples
    // [s1L, s1R,   s2L,   s2R]
    let floatIdx = 0;
    for (var i = 0; i < float32Array.length; i += 1) {
        // Combine two 8-bit values into a 16-bit signed integer
        // var uint16Value = (uint8Array[i + 1] << 8) + uint8Array[i];
        // var normalized = uint16Value / 0x8000;
        // if (normalized < 0x8000) {
    }
    // Convert to a float value in the range of -1 to 1
    float32Array[i] = float32Array[floatIdx] / 0x8000;

    floatIdx++;

    return float32Array;
}

function deInterleaveFloat32(float32Array) {
    var numChannels = 2; // Assuming stereo audio
    var numSamples = float32Array.length / numChannels;
    var channel1 = new Float32Array(numSamples);
    var channel2 = new Float32Array(numSamples);

    for (var i = 0; i < numSamples; i++) {
        var index = i * numChannels;
        channel1[i] = float32Array[index];
        channel2[i] = float32Array[index + 1];
    }

    return [channel1, channel2];
}

/**
 * Rust sends packets of various sizes, but AudioWorklet can only work with 128 frames
 * This processor will receive samples and buffer them in a circular buffer for consumption by
 * the process() function.
 */
class WebRTCReceiverProcessor extends AudioWorkletProcessor {
    ringBuffer: RingBuffer = new RingBuffer(44100 * 2 * 200, 2);
    shouldStart = false;
    sampleIdx = 0; // Multiply by sample rate to get the current timestamp
    sampleRate = 0; 

    constructor(options: AudioSourceNodeOptions) {
        super(options);
        try {
            this.port.onmessage = (event) => {
                let obj = event.data;

                switch (obj.type) {
                    case "samplerate":
                        this.sampleRate = obj.sampleRate;
                        this.sampleIdx = obj.sampleRate * obj.time; // Just for representation, actual seeking done in Rust streamer
                        break;
                    case "packet":
                        this.addSamples(obj.data);
                        // this.port.postMessage({
                        //     type: "log",
                        //     text:
                        //         "array " +
                        //         obj.data
                        // });
                        if (this.ringBuffer.framesAvailable >= 1024) {
                            this.shouldStart = true;
                        }
                        this.port.postMessage({
                            type: "buffer",
                            bufferedSamples: this.ringBuffer.framesAvailable
                        });
                        break;
                    case "reset":
                        this.shouldStart = false;
                        this.sampleIdx = 0;
                        this.ringBuffer = new RingBuffer(44100 * 2 * 200, 2);
                        break;
                    case "timestamp-query":
                        this.port.postMessage({
                            type: "timestamp",
                            time: (this.sampleIdx / this.sampleRate)
                        })

                    default:
                    // Nothing to do
                }
            };
        } catch (err) {
            this.port.postMessage({
                type: "log",
                text: err
            });
        }
    }

    addSamples(data: ArrayBuffer) {
        let float32Array = convertUInt8ToFloat32(data);
        let deInterleaved = deInterleaveFloat32(float32Array);
        // this.port.postMessage({
        //     type: "log",
        //     text: data.byteLength
        // });
        var numChannels = 2; // Assuming stereo audio
        // var numSamples = float32Array.length / numChannels;

        // for (var i = 0; i < numSamples; i++) {
        //     var index = i * numChannels;
        //     this.queue[0][i] = float32Array[index];
        //     this.queue[1][i] = float32Array[index + 1];
        // }
        this.ringBuffer.push(deInterleaved);
        // this.port.postMessage({
        //     type: "log",
        //     text: "ringbuf " + this.ringBuffer._channelData
        // });
    }

    process(inputs, outputs, parameters) {
        let output = outputs[0];
        // this.port.postMessage({
        //     type: "log",
        //     text: " outputs: " + output.length
        // });
        try {
            if (this.shouldStart && this.ringBuffer.framesAvailable >= 128) {
                this.ringBuffer.pull(output);

                this.port.postMessage({
                    type: "played",
                    playedSamples: 128
                });
                // this.port.postMessage({
                //     type: "log",
                //     text: "playing: " + outputs[0]
                // });
                this.sampleIdx = this.sampleIdx + 128;
            } else {
                for (let i = 0; i < 128; i++) {
                    output[0][i] = 0;
                    output[1][i] = 0;
                }
            }
        } catch (err) {
            this.port.postMessage({
                type: "log",
                text: err
            });
        }

        // To keep this processor alive.
        return true;
    }
}

registerProcessor("webrtc-receiver-processor", WebRTCReceiverProcessor);
