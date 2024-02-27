import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import type { StreamStatus } from "../../App";
import { streamInfo } from "../../data/store";

export default class WebRTCReceiver {
    playerConnection: RTCPeerConnection;
    remoteConnection: RTCPeerConnection;
    dataChannel: RTCDataChannel;
    onSampleData: (samples: Uint8Array) => void;
    shouldRestart = false;
    last10Packets: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    lastSnapshotTime;
    packetCount = 0;
    constructor() {
        this.playerConnection = new RTCPeerConnection();
        this.init();
        // Setup listeners
        this.playerConnection.addEventListener("icecandidate", (event) => {
            console.log("webrtc::Ice candidate", event);
            // Send ice candidate to server
            if (event.candidate) {
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
                    this.last10Packets[this.packetCount] =
                        event.data.byteLength;

                    if (this.packetCount === 9) {
                        // Calculate
                        let totalBytes = this.last10Packets.reduce(
                            (sum, p) => (sum += p),
                            0
                        );
                        let timeToSend =
                            performance.now() - this.lastSnapshotTime;
                        // If 148 bytes took 1.405s to send, that means the rate is
                        // 148 / 1405 = bytes per millisecond,  * 1000 = bytes per 1s
                        streamInfo.update((s) => ({
                            ...s,
                            receiveRate:
                                (totalBytes / (timeToSend * 1000)) * 1024
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
                console.log("webrtc::Connectionstatechange", event);
            }
        );
    }

    createDataChannel() {
        this.dataChannel = this.playerConnection.createDataChannel("data", {
            maxRetransmits: 0,
            ordered: false
        });
    }

    async init() {
        await this.checkExistingConnection();
        this.listenForIceCandidate();
        this.createDataChannel();
        this.createLocalOffer();
    }

    async checkExistingConnection() {
        let status = await invoke<StreamStatus>("init_streamer");
        console.log("webrtc::is_open::" + status.isOpen);
        this.shouldRestart = status.isOpen;
    }

    async createLocalOffer() {
        try {
            const localOffer = await this.playerConnection.createOffer();
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
        this.last10Packets = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        this.packetCount = 0;
        streamInfo.update((s) => ({
            ...s,
            bytesReceived: 0,
            receiveRate: 0,
            playedSamples: 0,
            bufferedSamples: 0
        }));
    }
}
