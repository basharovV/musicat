import type { AudioSourceNodeOptions } from "./App";

const { create, ConverterType } = globalThis.LibSampleRate;

class AudioResamplerNode extends AudioWorkletProcessor {
    src = null;
    timedExec = null;

    constructor(options: AudioSourceNodeOptions) {
        super();
        this.log("starting resampler", JSON.stringify(options));
        this.log(
            "target exec time per block",
            128 / options.processorOptions.inputSampleRate
        );
        // somewhere in the declaration of your Processor:
        if (
            options.processorOptions.inputSampleRate !==
            options.processorOptions.outputSampleRate
        ) {
            create(
                options.processorOptions.trackDescriptions[0].numberChannels,
                options.processorOptions.inputSampleRate,
                options.processorOptions.outputSampleRate,
                {
                    converterType: ConverterType.SRC_SINC_FASTEST // or some other quality
                }
            ).then((src) => {
                this.src = src;
            });
        }
    }

    log(...statements) {
        this.port.postMessage({
            type: "LOGGER",
            payload: [...statements].join(", ")
        });
    }

    /**
     *
     * @param {Array<Array<Float32Array>>} _inputs
     * @param {Array<Array<Float32Array>>} outputs
     * @param {Object} _parameters
     * @returns {boolean}
     */
    process(inputs, outputs, _parameters) {
        if (this.timedExec)
            this.log("actual exec ", currentTime - this.timedExec);
        // do something w.r.t. resampling
        this.timedExec = currentTime;
        if (this.src != null) {
            const resampledL = this.src.full(inputs[0][0]);
            const resampledR = this.src.full(inputs[0][1]);
            for (let i = 0; i < resampledL.length - 1; i++) {
                outputs[0][0][i] = resampledL[i];
            }
            for (let i = 0; i < resampledR.length - 1; i++) {
                outputs[0][1][i] = resampledR[i];
            }
            // this.log(
            //     `Resampled to ${inputs[0][0].length} samples to  ${resampledL.length} samples`
            // );
        }
        return true;
    }
}

registerProcessor("resampler-processor", AudioResamplerNode);
