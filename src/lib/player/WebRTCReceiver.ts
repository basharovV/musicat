import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { streamInfo } from "../../data/store";
import { emit } from "@tauri-apps/api/event";

const appWindow = getCurrentWebviewWindow();
const THROUGHPUT_SAMPLE_SIZE = 10;

export default class WebRTCReceiver {
    playerConnection = null;
    dataChannel = null;
    onSampleData = null;

    // Track signaling state to prevent race conditions
    #isSettingRemoteDescription = false;
    #iceCandidateQueue = [];

    constructor() {
        this.init();
    }

    async init() {
        // If connection is closed, try to re-initialize
        if (this.playerConnection?.signalingState === "closed") {
            console.log("webrtc::Connection closed, re-initializing");
            await this.setup();
            return;
        }

        // Ignore if already initialized and data channel is open
        if (this.dataChannel && this.dataChannel.readyState === "open") {
            console.log("webrtc::Already initialized");
            return;
        }

        await this.setup();
    }

    async setup() {
        // Close existing connection if any
        if (this.playerConnection) {
            this.playerConnection.close();
        }

        this.playerConnection = new RTCPeerConnection({
            iceServers: [], // Localhost doesn't need STUN
        });

        // 1. Setup ICE handling first
        this.setupIceListeners();

        // 2. Setup DataChannel discovery
        this.playerConnection.addEventListener("datachannel", (event) => {
            console.log("webrtc::Data channel received from Rust");
            this.dataChannel = event.channel;
            this.setupDataChannelListeners();
        });

        // 3. Start listening for the Offer
        // We do this BEFORE invoking Rust so we don't miss the event
        const unlisten = await this.setupOfferListener();

        // 4. Tell Rust to start the WebRTC handshake
        await invoke("init_webrtc");
    }

    setupIceListeners() {
        // Send local candidates to Rust
        this.playerConnection.addEventListener("icecandidate", (event) => {
            if (event.candidate) {
                // Filter for localhost/LAN to keep it clean, but usually fine to send all
                emit("handle_ice_candidate", {
                    candidate: JSON.stringify(event.candidate),
                });
            }
        });

        // Receive candidates from Rust
        appWindow.listen("webrtc-icecandidate-from-rust", async (event) => {
            const candidate = event.payload;

            // CRITICAL: Buffer candidates if the remote description isn't ready
            if (
                !this.playerConnection.remoteDescription ||
                this.#isSettingRemoteDescription
            ) {
                this.#iceCandidateQueue.push(candidate);
            } else {
                await this.playerConnection.addIceCandidate(candidate);
            }
        });
    }

    async setupOfferListener() {
        return await appWindow.listen("webrtc-offer", async (event) => {
            console.log("webrtc::Received offer from Rust");
            const offer = event.payload as RTCSessionDescriptionInit;

            this.#isSettingRemoteDescription = true;
            try {
                await this.playerConnection.setRemoteDescription(
                    new RTCSessionDescription(offer),
                );

                const answer = await this.playerConnection.createAnswer();
                await this.playerConnection.setLocalDescription(answer);

                // Send Answer back
                await invoke("handle_answer", {
                    answer: JSON.stringify(answer),
                });

                this.#isSettingRemoteDescription = false;

                // Process any queued candidates
                while (this.#iceCandidateQueue.length > 0) {
                    const cand = this.#iceCandidateQueue.shift();
                    await this.playerConnection.addIceCandidate(cand);
                }
            } catch (err) {
                console.error("Signaling error:", err);
                this.#isSettingRemoteDescription = false;
            }
        });
    }

    setupDataChannelListeners() {
        this.dataChannel.binaryType = "arraybuffer";

        this.dataChannel.onopen = () =>
            console.log("webrtc::Data Channel OPEN");

        this.dataChannel.onmessage = (event) => {
            if (this.onSampleData)
                this.onSampleData(new Uint8Array(event.data));
        };
    }
}
