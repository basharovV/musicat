import { invoke } from "@tauri-apps/api/tauri";
import type { GetFileSizeResponse } from "../../App";
import { MPEGDecoderWebWorker } from "mpg123-decoder";
import { create, ConverterType } from "@alexanderolsen/libsamplerate-js";
import { WaveFile } from 'wavefile';
let wav = new WaveFile();

// import { fetch } from '@tauri-apps/api/http'
/**
 * This class is responsible for fetching chunks of encoded audio,
 * directly from the file using request headers (supported by Tauri's asset protocol).
 *
 */
export default class FileStreamSource {
    // stream: ReadableStream;
    src: string;
    path: string;
    firstChunkSize = 100 * 1000; // 100kb
    desiredSize = 512 * 1000; // 512kb
    isCancelled = false;
    prevChunkSize = 0;
    chunksLeft = true;
    chunkIdx = 0; // Chunk counter.
    bytePos = 0; // Fetched so far
    timePos = 0; // In seconds
    sampleRate = 0; // Of the last fetched chunk
    samplesDecoded = 0;
    fileSize = 0;
    isFetchingChunk = false;
    decoder;
    samples: Float32Array[][];
    
    constructor() {}

    async queueFile(src, path, mimeType, wholeFile = false) {
        this.isCancelled = false;
        this.src = src; // To fetch via asset protocol (converted to asset://[URL encoded file path])
        this.path = path; // To get file size from Rust backend
        if (mimeType === "audio/mpeg" && !this.decoder) {
            this.decoder = new MPEGDecoderWebWorker({enableGapless: true});
            // wait for the WASM to be compiled
            await this.decoder.ready;
        }
        return await this.start(); // Get the file size and the first two bytes
    }

    sleep(ms) {
        return new Promise((resolve) => {
            setTimeout(resolve, ms);
        });
    }

    async start() {
        console.log("Starting stream for ", this.path);

        const fileSizeResponse = await invoke<GetFileSizeResponse>(
            "get_file_size",
            {
                event: { path: this.path }
            }
        );
        this.fileSize = fileSizeResponse.fileSize;
        console.log("filesize", this.fileSize);
        return await this.getNextChunk(this.firstChunkSize);
    }

    async getNextChunk(size?:number) {
        if (!size) size = this.desiredSize;
        let decodedChunk;
        console.log("FileStreamSource::getNextChunk()", this.path);
        console.log("size", size);
        this.isFetchingChunk = true;
        if (this.isCancelled) {
            return;
        }

        if (this.fileSize) {
            let chunkEndIdx = Math.min(this.fileSize, this.bytePos + size);
            let response = await fetch(this.src, {
                headers: {
                    "Range": `bytes=${this.bytePos}-${chunkEndIdx}`
                }
            });
            console.log("chunk response", response);
            this.prevChunkSize = Number(response.headers.get("Content-Length"));
            console.log("bytePos", this.bytePos, "size", this.prevChunkSize);
            if (response.ok) {
                for await (const { done, value } of response.body) {
                    if (value) {
                        let currentChunk: Uint8Array = value;

                        if (currentChunk) {
                            console.log("currentChunk", currentChunk);

                            // Decode the chunk according to current codec
                            decodedChunk =
                                await this.decoder.decode(currentChunk);
                            await this.decoder.reset();

                            // const src = await create(
                            //     2,
                            //     decodedChunk.sampleRate,
                            //     44100,
                            //     {
                            //         converterType:
                            //             ConverterType.SRC_SINC_BEST_QUALITY // default SRC_SINC_FASTEST. see API for more
                            //     }
                            // );
                            // decodedChunk.channelData[0] = src.full(decodedChunk.channelData[0]);
                            // decodedChunk.channelData[1] = src.full(decodedChunk.channelData[1]);
                            // src.destroy(); // clean up

                            console.log("decodedChunk", decodedChunk);
                            this.sampleRate = decodedChunk.sampleRate;
                            this.samplesDecoded = decodedChunk.samplesDecoded;
                            this.isFetchingChunk = false;
                            if (this.isCancelled) {
                                return null;
                            }
                        }
                    }
                }
            }
            if (chunkEndIdx === this.fileSize) {
                console.log("END OF STREAM");
                this.chunksLeft = false;
                this.isFetchingChunk = false;
                return null;
            }

            this.chunkIdx += 1;
            this.bytePos += this.prevChunkSize;
            this.timePos = this.samplesDecoded / this.sampleRate;
            console.log('samplesDecoded', this.samplesDecoded);
            this.isFetchingChunk = false;
            if (decodedChunk) {
                return decodedChunk;
            }
        }
    }

    cancel() {
        this.isCancelled = true;
        console.log("Cancelling existing stream for ", this.path);
    }

    async reset() {
        this.cancel();
        this.chunkIdx = 0;
        this.chunksLeft = true;
        this.fileSize = 0;
        this.isFetchingChunk = false;
        this.prevChunkSize = 0;
        if (this.decoder) {
            await this.decoder.reset();
        }
    }
}
