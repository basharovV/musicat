/**
 * Copyright 2018 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 *
 * A JS FIFO implementation for the AudioWorklet. 3 assumptions for the
 * simpler operation:
 *  1. the push and the pull operation are done by 128 frames. (Web Audio
 *    API's render quantum size in the speficiation)
 *  2. the channel count of input/output cannot be changed dynamically.
 *    The AudioWorkletNode should be configured with the `.channelCount = k`
 *    (where k is the channel count you want) and
 *    `.channelCountMode = explicit`.
 *  3. This is for the single-thread operation. (obviously)
 *
 * @class
 */
export default class RingBuffer {
    _readIndex: number;
    _writeIndex: number;
    _framesAvailable: number;
    _channelCount: any;
    _length: any;
    _channelData: any[];
    /**
     * @constructor
     * @param  {number} length Buffer length in frames.
     * @param  {number} channelCount Buffer channel count.
     */
    constructor(length, channelCount) {
        this._readIndex = 0;
        this._writeIndex = 0;
        this._framesAvailable = 0;

        this._channelCount = channelCount;
        this._length = length;
        this._channelData = [];
        for (let i = 0; i < this._channelCount; ++i) {
            this._channelData[i] = new Float32Array(length);
        }
    }

    /**
     * Getter for Available frames in buffer.
     *
     * @return {number} Available frames in buffer.
     */
    get framesAvailable() {
        return this._framesAvailable;
    }

    /**
     * Push a sequence of Float32Arrays to buffer.
     *
     * @param  {array} arraySequence A sequence of Float32Arrays.
     */
    push(arraySequence) {
        // The channel count of arraySequence and the length of each channel must
        // match with this buffer obejct.

        // Transfer data from the |arraySequence| storage to the internal buffer.
        let sourceLength = arraySequence[0] ? arraySequence[0].length : 0;
        for (let i = 0; i < sourceLength; ++i) {
            let writeIndex = (this._writeIndex + i) % this._length;
            for (let channel = 0; channel < this._channelCount; ++channel) {
                if (arraySequence[channel])
                    this._channelData[channel][writeIndex] =
                        arraySequence[channel][i];
            }
        }

        this._writeIndex += sourceLength;
        if (this._writeIndex >= this._length) {
            this._writeIndex = 0;
        }

        // For excessive frames, the buffer will be overwritten.
        this._framesAvailable += sourceLength;
        if (this._framesAvailable > this._length) {
            this._framesAvailable = this._length;
        }
    }

    /**
     * Pull data out of buffer and fill a given sequence of Float32Arrays.
     *
     * @param  {array} arraySequence An array of Float32Arrays.
     */
    pull(arraySequence) {
        // The channel count of arraySequence and the length of each channel must
        // match with this buffer obejct.

        // If the FIFO is completely empty, do nothing.
        if (this._framesAvailable === 0) {
            return;
        }

        let destinationLength = arraySequence[0].length;

        // Transfer data from the internal buffer to the |arraySequence| storage.
        for (let i = 0; i < destinationLength; ++i) {
            let readIndex = (this._readIndex + i) % this._length;
            for (let channel = 0; channel < this._channelCount; ++channel) {
                arraySequence[channel][i] =
                    this._channelData[channel][readIndex];
            }
        }

        this._readIndex += destinationLength;
        if (this._readIndex >= this._length) {
            this._readIndex = 0;
        }

        this._framesAvailable -= destinationLength;
        if (this._framesAvailable < 0) {
            this._framesAvailable = 0;
        }
    }
}
