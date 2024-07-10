import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import type { StreamStatus } from "../../App";
import { streamInfo } from "../../data/store";

const THROUGHPUT_SAMPLE_SIZE = 10;

export default class WebRTCReceiver {
    playerConnection: RTCPeerConnection;
    remoteConnection: RTCPeerConnection;
    dataChannel: RTCDataChannel;
    onSampleData: (samples: Uint8Array) => void;
    shouldRestart = false;
    throughputSample: number[] = Array(THROUGHPUT_SAMPLE_SIZE).fill(0);
    lastSnapshotTime;
    packetCount = 0;

    // Flow control (based on resume/pause decoding signals)
    maxBufferSizeSeconds = 10; // When buffer goes past this length, pause decoding
    minBufferSizeSeconds = 5; // When buffer decreases to this length, resume decoding
    isFillingBuffer = true; // While decoding
    currentBufferedSeconds = 0;

    constructor() {
        this.init();
    }

    createDataChannel() {
        this.dataChannel = this.playerConnection.createDataChannel("data", {
            maxRetransmits: 0,
            ordered: false
        });
    }

    async init() {
        this.playerConnection = new RTCPeerConnection({ iceServers: [] });
        await this.checkExistingConnection();
        this.listenForIceCandidate();
        this.createDataChannel();
        await this.createLocalOffer();

        // Setup listeners

        // Setup listeners
        this.playerConnection.addEventListener("icecandidate", (event) => {
            console.log("webrtc::Ice candidate", event);
            // Send ice candidate to server
            // Check if the candidate is a local candidate
            if (
                event.candidate &&
                event.candidate.candidate.match(/(127.0.0.1|localhost|.local)/)
            ) {
                console.log("webrtc::Sending candidate to remote...");
                this.sendIceCandidateToRemote(event.candidate);
            }
        });
        this.playerConnection.addEventListener("datachannel", (event) => {
            console.log("webrtc::Data channel", event);
            this.dataChannel = event.channel;

            // Listener for when the datachannel is opened
            this.dataChannel.addEventListener("open", (event) => {
                // Force the binary type to be ArrayBuffer
                this.dataChannel.binaryType = "arraybuffer";

                // this.sendAudioContextState();
                console.log("webrtc::Data channel opened");
            });

            // Listener for when the datachannel is closed
            this.dataChannel.addEventListener("close", (event) => {
                // Tear down audio
                console.log("webrtc::Data channel closed");
            });

            // on event
            this.dataChannel.addEventListener("message", (event) => {
                // console.log("webrtc::Data channel message", event);
                if (event.data) {
                    this.onSampleData && this.onSampleData(event.data);

                    // Calculate throughput
                    this.throughputSample[this.packetCount] =
                        event.data.byteLength;

                    if (this.packetCount === THROUGHPUT_SAMPLE_SIZE - 1) {
                        // Calculate
                        let totalBytes = this.throughputSample.reduce(
                            (sum, p) => (sum += p),
                            0
                        );
                        let timeToSend =
                            performance.now() - this.lastSnapshotTime;
                        // If 148 bytes took 1.405s to send, that means the rate is
                        // 148 / 1405 = bytes per millisecond,  * 1000 = bytes per 1s
                        // Times 8 to get kbits/sec
                        // console.log("total bytes", totalBytes, timeToSend);
                        let receiveBitrate =
                            ((totalBytes * 8) / timeToSend) * 1000;
                        streamInfo.update((s) => ({
                            ...s,
                            receiveRate: receiveBitrate
                        }));

                        this.packetCount = 0;
                        this.lastSnapshotTime = performance.now();
                    } else {
                        this.packetCount++;
                    }
                    streamInfo.update((s) => ({
                        ...s,
                        bytesReceived: s.bytesReceived + event.data.byteLength
                    }));
                }
            });
        });
        this.playerConnection.addEventListener(
            "connectionstatechange",
            (event) => {
                console.log(
                    "webrtc::Connectionstatechange",
                    this.playerConnection.connectionState
                );
                if (this.playerConnection.connectionState === "disconnected") {
                    // Try to re-establish
                    this.init();
                }
            }
        );
    }

    async checkExistingConnection() {
        let status = await invoke<StreamStatus>("init_streamer");
        console.log("webrtc::is_open::" + status.isOpen);
        this.shouldRestart = status.isOpen;
    }

    async createLocalOffer() {
        try {
            const localOffer = await this.playerConnection.createOffer({
                iceRestart: true
            });
            this.listenForAnswer();
            await this.sendOfferToRemote(localOffer);
            await this.handleLocalDescription(localOffer);
        } catch (e) {
            console.error("webrtc::Failed to create session description: ", e);
        }
    }

    async listenForAnswer() {
        try {
            const unlisten = await appWindow.listen(
                "webrtc-answer",
                async (event) => {
                    console.log("webrtc::answer", event);

                    await this.playerConnection.setRemoteDescription(
                        new RTCSessionDescription(event.payload)
                    );
                    unlisten();
                }
            );
        } catch (e) {
            console.error(e);
        }
    }

    async listenForIceCandidate() {
        try {
            const unlisten = await appWindow.listen(
                "webrtc-icecandidate-client",
                async (event) => {
                    console.log("webrtc::icecandidate-client", event);
                    this.playerConnection.addIceCandidate(event.payload);
                    unlisten();
                }
            );
        } catch (e) {
            console.error(e);
        }
    }

    async sendOfferToRemote(offer: RTCSessionDescriptionInit) {
        try {
            await appWindow.emit("webrtc-signal", offer);
        } catch (e) {
            console.error("webrtc::Failed to send session description: ", e);
        }
    }

    async sendIceCandidateToRemote(candidate: RTCIceCandidateInit) {
        try {
            await appWindow.emit("webrtc-icecandidate-server", candidate);
        } catch (e) {
            console.error("webrtc::Failed to send candidate: ", e);
        }
    }
    async handleLocalDescription(desc) {
        this.playerConnection.setLocalDescription(desc);
        console.log("webrtc::Offer from localConnection:\n", desc.sdp);
        try {
            // const remoteAnswer = await this.remoteConnection.createAnswer();
            // this.handleRemoteAnswer(remoteAnswer);
        } catch (e) {
            console.error("webrtc::Error when creating remote answer: ", e);
        }
    }

    handleRemoteAnswer(desc) {
        console.log("webrtc::Answer from remoteConnection:\n", desc.sdp);
        this.playerConnection.setRemoteDescription(desc);
    }

    prepareForNewStream() {
        this.throughputSample = Array(THROUGHPUT_SAMPLE_SIZE).fill(0);
        this.packetCount = 0;
        streamInfo.update((s) => ({
            ...s,
            bytesReceived: 0,
            receiveRate: 0,
            playedSamples: 0,
            bufferedSamples: 0
        }));
    }

    doFlowControl(bufferedTimeSeconds: number) {
        if (
            this.isFillingBuffer &&
            bufferedTimeSeconds > this.maxBufferSizeSeconds
        ) {
            // Pause decoding for a bit
            invoke("decode_control", {
                event: {
                    decoding_active: false
                }
            });
            this.isFillingBuffer = false;
        } else if (
            !this.isFillingBuffer &&
            bufferedTimeSeconds < this.minBufferSizeSeconds
        ) {
            // Resume decoding again
            invoke("decode_control", {
                event: {
                    decoding_active: true
                }
            });
            this.isFillingBuffer = true;
        }
    }
}
