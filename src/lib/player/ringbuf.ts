"use strict";

/**
 * Interleaved -> Planar audio buffer conversion
 *
 * This is useful to get data from a codec, the network, or anything that is
 * interleaved, into a planar format, for example a Web Audio API AudioBuffer or
 * the output parameter of an AudioWorkletProcessor.
 *
 * @param {Float32Array} input is an array of n*128 frames arrays, interleaved,
 * where n is the channel count.
 * @param {Float32Array} output is an array of 128-frames arrays.
 */
function deinterleave(input, output) {
    const channel_count = input.length / 256;
    if (output.length !== channel_count) {
        throw RangeError("not enough space in output arrays");
    }
    for (let i = 0; i < channel_count; i++) {
        const out_channel = output[i];
        let interleaved_idx = i;
        for (let j = 0; j < 128; ++j) {
            out_channel[j] = input[interleaved_idx];
            interleaved_idx += channel_count;
        }
    }
}

/**
 * Planar -> Interleaved audio buffer conversion
 *
 * This function is useful to get data from the Web Audio API (that uses a
 * planar format), into something that a codec or network streaming library
 * would expect.
 *
 * @param {Float32Array} input An array of n*128 frames Float32Array that hold the audio data.
 * @param {Float32Array} output A Float32Array that is n*128 elements long.
 */
function interleave(input, output) {
    if (input.length * 128 !== output.length) {
        throw RangeError("input and output of incompatible sizes");
    }
    let out_idx = 0;
    for (let i = 0; i < 128; i++) {
        for (let channel = 0; channel < input.length; channel++) {
            output[out_idx] = input[channel][i];
            out_idx++;
        }
    }
}

/**
 * Send interleaved audio frames to another thread, wait-free.
 *
 * These classes allow communicating between a non-real time thread (browser
 * main thread or worker) and a real-time thread (in an AudioWorkletProcessor).
 * Write and Reader cannot change role after setup, unless externally
 * synchronized.
 *
 * GC _can_ happen during the initial construction of this object when hopefully
 * no audio is being output. This depends on how implementations schedule GC
 * passes. After the setup phase no GC is triggered on either side of the queue.
 */
class AudioWriter {
    /**
     * From a RingBuffer, build an object that can enqueue enqueue audio in a ring
     * buffer.
     * @constructor
     */
    constructor(ringbuf) {
        if (ringbuf.type() !== "Float32Array") {
            throw TypeError(
                "This class requires a ring buffer of Float32Array"
            );
        }
        this.ringbuf = ringbuf;
    }
    /**
     * Enqueue a buffer of interleaved audio into the ring buffer.
     *
     *
     * Care should be taken to enqueue a number of samples that is a multiple of the
     * channel count of the audio stream.
     *
     * @param {Float32Array} buf An array of interleaved audio frames.
     *
     * @return The number of samples that have been successfuly written to the
     * queue. `buf` is not written to during this call, so the samples that
     * haven't been written to the queue are still available.
     */
    enqueue(buf) {
        return this.ringbuf.push(buf);
    }

    /**
     * @return The free space in the ring buffer. This is the amount of samples
     * that can be queued, with a guarantee of success.
     */
    available_write() {
        return this.ringbuf.available_write();
    }
}

/**
 * Receive interleaved audio frames to another thread, wait-free.
 *
 * GC _can_ happen during the initial construction of this object when hopefully
 * no audio is being output. This depends on how implementations schedule GC
 * passes. After the setup phase no GC is triggered on either side of the queue.
 */
class AudioReader {
    /**
     * From a RingBuffer, build an object that can dequeue audio in a ring
     * buffer.
     * @constructor
     */
    constructor(ringbuf) {
        if (ringbuf.type() !== "Float32Array") {
            throw TypeError(
                "This class requires a ring buffer of Float32Array"
            );
        }
        this.ringbuf = ringbuf;
    }
    /**
     * Attempt to dequeue at most `buf.length` samples from the queue. This
     * returns the number of samples dequeued. If greater than 0, the samples are
     * at the beginning of `buf`.
     *
     * Care should be taken to dequeue a number of samples that is a multiple of the
     * channel count of the audio stream.
     *
     * @param {Float32Array} buf A buffer in which to copy the dequeued
     * interleaved audio frames.
     * @return The number of samples dequeued.
     */
    dequeue(buf) {
        if (this.ringbuf.empty()) {
            return 0;
        }
        return this.ringbuf.pop(buf);
    }
    /**
     * Query the occupied space in the queue.
     *
     * @return The amount of samples that can be read with a guarantee of success.
     *
     */
    available_read() {
        return this.ringbuf.available_read();
    }
}

/**
 * Send parameter changes, lock free, no gc, between a UI thread (browser
 * main thread or worker) and a real-time thread (in an AudioWorkletProcessor).
 * Write and Reader cannot change role after setup, unless externally
 * synchronized.
 *
 * GC _can_ happen during the initial construction of this object when hopefully
 * no audio is being output. This depends on the implementation.
 *
 * Parameter changes are like in the VST framework: an index and a float value
 * (no restriction on the value).
 *
 * This class supports up to 256 parameters, but this is easy to extend if
 * needed.
 *
 * An element is an index, that is an unsigned byte, and a float32, which is 4
 * bytes.
 */
class ParameterWriter {
    /**
     * From a RingBuffer, build an object that can enqueue a parameter change in
     * the queue.
     * @param {RingBuffer} ringbuf A RingBuffer object of Uint8Array.
     * @constructor
     */
    constructor(ringbuf) {
        if (ringbuf.type() !== "Uint8Array") {
            throw TypeError("This class requires a ring buffer of Uint8Array");
        }
        const SIZE_ELEMENT = 5;
        this.ringbuf = ringbuf;
        this.mem = new ArrayBuffer(SIZE_ELEMENT);
        this.array = new Uint8Array(this.mem);
        this.view = new DataView(this.mem);
    }
    /*
     * Enqueue a parameter change for parameter of index `index`, with a new value
     * of `value`.
     *
     * @param {number} index The index of the parameter.
     * @param {number} value The value of the parameter.
     * @return True if enqueuing suceeded, false otherwise.
     */
    enqueue_change(index, value) {
        const SIZE_ELEMENT = 5;
        this.view.setUint8(0, index);
        this.view.setFloat32(1, value);
        if (this.ringbuf.available_write() < SIZE_ELEMENT) {
            return false;
        }
        return this.ringbuf.push(this.array) === SIZE_ELEMENT;
    }
}

/**
 * Receive parameter changes, lock free, no gc, between a UI thread (browser
 * main thread or worker) and a real-time thread (in an AudioWorkletProcessor).
 * Write and Reader cannot change role after setup, unless externally
 * synchronized.
 *
 * GC _can_ happen during the initial construction of this object when hopefully
 * no audio is being output. This depends on the implementation.
 *
 * Parameter changes are like in the VST framework: an index and a float value
 * (no restriction on the value).
 *
 * This class supports up to 256 parameters, but this is easy to extend if
 * needed.
 *
 * An element is an index, that is an unsigned byte, and a float32, which is 4
 * bytes.
 */
class ParameterReader {
    /**
     * @constructor
     * @param {RingBuffer} ringbuf A RingBuffer setup to hold Uint8.
     */
    constructor(ringbuf) {
        const SIZE_ELEMENT = 5;
        this.ringbuf = ringbuf;
        this.mem = new ArrayBuffer(SIZE_ELEMENT);
        this.array = new Uint8Array(this.mem);
        this.view = new DataView(this.mem);
    }
    /**
     * Attempt to dequeue a single parameter change.
     * @param {Object} o An object with two attributes: `index` and `value`.
     * @return true if a parameter change has been dequeued, false otherwise.
     */
    dequeue_change(o) {
        if (this.ringbuf.empty()) {
            return false;
        }
        const rv = this.ringbuf.pop(this.array);
        o.index = this.view.getUint8(0);
        o.value = this.view.getFloat32(1);

        return rv === 2;
    }
}

/** The base RingBuffer class
 *
 * A Single Producer - Single Consumer thread-safe wait-free ring buffer.
 *
 * The producer and the consumer can be on separate threads, but cannot change roles,
 * except with external synchronization.
 */
class RingBuffer {
    /** Allocate the SharedArrayBuffer for a RingBuffer, based on the type and
     * capacity required
     * @param {number} capacity The number of elements the ring buffer will be
     * able to hold.
     * @param {TypedArray} type A typed array constructor, the type that this ring
     * buffer will hold.
     * @return {SharedArrayBuffer} A SharedArrayBuffer of the right size.
     * @static
     */
    static getStorageForCapacity(capacity, type) {
        if (!type.BYTES_PER_ELEMENT) {
            throw TypeError("Pass in a ArrayBuffer subclass");
        }
        const bytes = 8 + (capacity + 1) * type.BYTES_PER_ELEMENT;
        return new SharedArrayBuffer(bytes);
    }
    /**
     * @constructor
     * @param {SharedArrayBuffer} sab A SharedArrayBuffer obtained by calling
     * {@link RingBuffer.getStorageFromCapacity}.
     * @param {TypedArray} type A typed array constructor, the type that this ring
     * buffer will hold.
     */
    constructor(sab, type) {
        if (type.BYTES_PER_ELEMENT === undefined) {
            throw TypeError(
                "Pass a concrete typed array class as second argument"
            );
        }

        // Maximum usable size is 1<<32 - type.BYTES_PER_ELEMENT bytes in the ring
        // buffer for this version, easily changeable.
        // -4 for the write ptr (uint32_t offsets)
        // -4 for the read ptr (uint32_t offsets)
        // capacity counts the empty slot to distinguish between full and empty.
        this._type = type;
        this._capacity = (sab.byteLength - 8) / type.BYTES_PER_ELEMENT;
        this.buf = sab;
        this.write_ptr = new Uint32Array(this.buf, 0, 1);
        this.read_ptr = new Uint32Array(this.buf, 4, 1);
        this.storage = new type(this.buf, 8, this._capacity);
    }
    /**
     * @return the type of the underlying ArrayBuffer for this RingBuffer. This
     * allows implementing crude type checking.
     */
    type() {
        return this._type.name;
    }

    /**
     * Push elements to the ring buffer.
     * @param {TypedArray} elements A typed array of the same type as passed in the ctor, to be written to the queue.
     * @param {Number} length If passed, the maximum number of elements to push.
     * If not passed, all elements in the input array are pushed.
     * @param {Number} offset If passed, a starting index in elements from which
     * the elements are read. If not passed, elements are read from index 0.
     * @return the number of elements written to the queue.
     */
    push(elements, length, offset = 0) {
        const rd = Atomics.load(this.read_ptr, 0);
        const wr = Atomics.load(this.write_ptr, 0);

        if ((wr + 1) % this._storage_capacity() === rd) {
            // full
            return 0;
        }

        const len = length !== undefined ? length : elements.length;

        const to_write = Math.min(this._available_write(rd, wr), len);
        const first_part = Math.min(this._storage_capacity() - wr, to_write);
        const second_part = to_write - first_part;

        this._copy(elements, offset, this.storage, wr, first_part);
        this._copy(elements, offset + first_part, this.storage, 0, second_part);

        // publish the enqueued data to the other side
        Atomics.store(
            this.write_ptr,
            0,
            (wr + to_write) % this._storage_capacity()
        );

        return to_write;
    }

    /**
     * Write bytes to the ring buffer using callbacks. This create wrapper
     * objects and can GC, so it's best to no use this variant from a real-time
     * thread such as an AudioWorklerProcessor `process` method.
     * The callback is passed two typed arrays of the same type, to be filled.
     * This allows skipping copies if the API that produces the data writes is
     * passed arrays to write to, such as `AudioData.copyTo`.
     * @param {number} amount The maximum number of elements to write to the ring
     * buffer. If amount is more than the number of slots available for writing,
     * then the number of slots available for writing will be made available: no
     * overwriting of elements can happen.
     * @param {Function} cb A callback with two parameters, that are two typed
     * array of the correct type, in which the data need to be copied. It is
     * necessary to write exactly the number of elements determined by the size
     * of the two typed arrays.
     * @return The number of elements written to the queue.
     */
    writeCallback(amount, cb) {
        const rd = Atomics.load(this.read_ptr, 0);
        const wr = Atomics.load(this.write_ptr, 0);

        if ((wr + 1) % this._storage_capacity() === rd) {
            // full
            return 0;
        }

        const to_write = Math.min(this._available_write(rd, wr), amount);
        const first_part = Math.min(this._storage_capacity() - wr, to_write);
        const second_part = to_write - first_part;

        // This part will cause GC: don't use in the real time thread.
        const first_part_buf = new this._type(
            this.storage.buffer,
            8 + wr * 4,
            first_part
        );
        const second_part_buf = new this._type(
            this.storage.buffer,
            8 + 0,
            second_part
        );

        cb(first_part_buf, second_part_buf);

        // publish the enqueued data to the other side
        Atomics.store(
            this.write_ptr,
            0,
            (wr + to_write) % this._storage_capacity()
        );

        return to_write;
    }

    /**
     * Read up to `elements.length` elements from the ring buffer. `elements` is a typed
     * array of the same type as passed in the ctor.
     * Returns the number of elements read from the queue, they are placed at the
     * beginning of the array passed as parameter.
     * @param {TypedArray} elements An array in which the elements read from the
     * queue will be written, starting at the beginning of the array.
     * @param {Number} length If passed, the maximum number of elements to pop. If
     * not passed, up to elements.length are popped.
     * @param {Number} offset If passed, an index in elements in which the data is
     * written to. `elements.length - offset` must be greater or equal to
     * `length`.
     * @return The number of elements read from the queue.
     */
    pop(elements, length, offset = 0) {
        const rd = Atomics.load(this.read_ptr, 0);
        const wr = Atomics.load(this.write_ptr, 0);

        if (wr === rd) {
            return 0;
        }

        const len = length !== undefined ? length : elements.length;
        const to_read = Math.min(this._available_read(rd, wr), len);

        const first_part = Math.min(this._storage_capacity() - rd, to_read);
        const second_part = to_read - first_part;

        this._copy(this.storage, rd, elements, offset, first_part);
        this._copy(this.storage, 0, elements, offset + first_part, second_part);

        Atomics.store(
            this.read_ptr,
            0,
            (rd + to_read) % this._storage_capacity()
        );

        return to_read;
    }

    /**
     * @return True if the ring buffer is empty false otherwise. This can be late
     * on the reader side: it can return true even if something has just been
     * pushed.
     */
    empty() {
        const rd = Atomics.load(this.read_ptr, 0);
        const wr = Atomics.load(this.write_ptr, 0);

        return wr === rd;
    }

    /**
     * @return True if the ring buffer is full, false otherwise. This can be late
     * on the write side: it can return true when something has just been popped.
     */
    full() {
        const rd = Atomics.load(this.read_ptr, 0);
        const wr = Atomics.load(this.write_ptr, 0);

        return (wr + 1) % this._storage_capacity() === rd;
    }

    /**
     * @return The usable capacity for the ring buffer: the number of elements
     * that can be stored.
     */
    capacity() {
        return this._capacity - 1;
    }

    /**
     * @return The number of elements available for reading. This can be late, and
     * report less elements that is actually in the queue, when something has just
     * been enqueued.
     */
    available_read() {
        const rd = Atomics.load(this.read_ptr, 0);
        const wr = Atomics.load(this.write_ptr, 0);
        return this._available_read(rd, wr);
    }

    /**
     * @return The number of elements available for writing. This can be late, and
     * report less elements that is actually available for writing, when something
     * has just been dequeued.
     */
    available_write() {
        const rd = Atomics.load(this.read_ptr, 0);
        const wr = Atomics.load(this.write_ptr, 0);
        return this._available_write(rd, wr);
    }

    // private methods //

    /**
     * @return Number of elements available for reading, given a read and write
     * pointer.
     * @private
     */
    _available_read(rd, wr) {
        return (wr + this._storage_capacity() - rd) % this._storage_capacity();
    }

    /**
     * @return Number of elements available from writing, given a read and write
     * pointer.
     * @private
     */
    _available_write(rd, wr) {
        return this.capacity() - this._available_read(rd, wr);
    }

    /**
     * @return The size of the storage for elements not accounting the space for
     * the index, counting the empty slot.
     * @private
     */
    _storage_capacity() {
        return this._capacity;
    }

    /**
     * Copy `size` elements from `input`, starting at offset `offset_input`, to
     * `output`, starting at offset `offset_output`.
     * @param {TypedArray} input The array to copy from
     * @param {Number} offset_input The index at which to start the copy
     * @param {TypedArray} output The array to copy to
     * @param {Number} offset_output The index at which to start copying the elements to
     * @param {Number} size The number of elements to copy
     * @private
     */
    _copy(input, offset_input, output, offset_output, size) {
        for (let i = 0; i < size; i++) {
            output[offset_output + i] = input[offset_input + i];
        }
    }
}

export {
    AudioReader,
    AudioWriter,
    ParameterReader,
    ParameterWriter,
    RingBuffer
};
