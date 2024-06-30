// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod file_streamer {
    use std::sync::mpsc::{Receiver, Sender, TryRecvError};
    use std::thread;
    use std::time::Instant;

    use std::collections::HashMap;
    use std::fs::File;
    use std::marker::PhantomData;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::Arc;

    use atomic_wait::{wake_all, wake_one};
    use cpal::traits::{DeviceTrait, HostTrait};
    use log::warn;
    use symphonia::core::audio::{Layout, RawSampleBuffer, SampleBuffer, SignalSpec};
    use symphonia::core::codecs::{DecoderOptions, FinalizeResult, CODEC_TYPE_NULL};
    use symphonia::core::errors::Error::{self, ResetRequired};
    use symphonia::core::formats::{FormatOptions, SeekTo, Track};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::core::units::Time;
    use symphonia::default::get_probe;
    use tauri::{AppHandle, Manager};
    use tokio::sync::{Mutex, MutexGuard};
    use tokio::time::Duration;

    use tokio_util::sync::CancellationToken;
    use webrtc::api::interceptor_registry::register_default_interceptors;
    use webrtc::api::media_engine::MediaEngine;
    use webrtc::api::APIBuilder;
    use webrtc::data_channel::RTCDataChannel;
    use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
    use webrtc::interceptor::registry::Registry;
    use webrtc::peer_connection::configuration::RTCConfiguration;
    use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
    use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
    use webrtc::peer_connection::RTCPeerConnection;

    use crate::output::{self, AudioOutput};
    use crate::resampler::Resampler;
    use crate::{
        GetWaveformRequest, GetWaveformResponse, LoopRegionRequest, SampleOffsetEvent,
        StreamFileRequest, VolumeControlEvent,
    };

    #[derive(Debug)]
    pub enum PlayerControlEvent {
        StreamFile(StreamFileRequest), // path, seekpos
        LoopRegion(LoopRegionRequest),
    }

    pub const PAUSED: u32 = 0;
    pub const ACTIVE: u32 = 1;

    #[derive(Clone)]
    pub struct AudioStreamer<'a> {
        pub peer_connection: Arc<Mutex<Option<Arc<RTCPeerConnection>>>>,
        pub data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
        pub is_open: Arc<AtomicBool>,
        pub path: Arc<String>,
        pub cancel_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
        phantom: PhantomData<&'a RTCPeerConnection>,
        phantom2: PhantomData<&'a RTCDataChannel>,
        pub player_control_receiver: Arc<Mutex<Receiver<PlayerControlEvent>>>,
        pub player_control_sender: Sender<PlayerControlEvent>,
        pub next_track_receiver: Arc<Mutex<Receiver<StreamFileRequest>>>,
        pub next_track_sender: Sender<StreamFileRequest>,
        pub decoding_active: Arc<AtomicU32>,
        pub audio_output: Arc<Mutex<Option<Box<dyn output::AudioOutput + Send + Sync>>>>,
        pub volume_control_receiver: Arc<Mutex<Receiver<VolumeControlEvent>>>,
        pub volume_control_sender: Sender<VolumeControlEvent>,
    }

    impl<'a> AudioStreamer<'a> {
        pub fn create() -> Result<AudioStreamer<'a>, Box<dyn std::error::Error + Send + Sync>> {
            let (sender_vol, receiver_vol) = std::sync::mpsc::channel();

            // set up message passing
            let (sender_tx, receiver_rx): (
                Sender<PlayerControlEvent>,
                Receiver<PlayerControlEvent>,
            ) = std::sync::mpsc::channel();

            let (sender_next, receiver_next): (
                Sender<StreamFileRequest>,
                Receiver<StreamFileRequest>,
            ) = std::sync::mpsc::channel();

            let (sender_transition, receiver_transition): (Sender<bool>, Receiver<bool>) =
                std::sync::mpsc::channel();

            Ok(AudioStreamer {
                peer_connection: Arc::new(Mutex::new(None)),
                data_channel: Arc::new(Mutex::new(None)),
                is_open: Arc::new(AtomicBool::new(false)),
                path: Arc::new(String::new()),
                cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
                phantom: PhantomData,
                phantom2: PhantomData,
                player_control_receiver: Arc::new(Mutex::new(receiver_rx)),
                player_control_sender: sender_tx,
                next_track_receiver: Arc::new(Mutex::new(receiver_next)),
                next_track_sender: sender_next,
                decoding_active: Arc::new(AtomicU32::new(ACTIVE)),
                audio_output: Arc::new(Mutex::new(None)),
                volume_control_receiver: Arc::new(Mutex::new(receiver_vol)),
                volume_control_sender: sender_vol,
            })
        }

        pub fn init(&self, app_handle: AppHandle) -> () {
            let receiver = self.player_control_receiver.clone();
            let next_track_receiver = self.next_track_receiver.clone();
            let cancel_tokens = self.cancel_tokens.clone();
            let decoding_active = self.decoding_active.clone();
            let volume_control_receiver = self.volume_control_receiver.clone();
            let data_channel = self.data_channel.clone();

            std::thread::spawn(move || {
                // AUDIO THREAD!
                // Constantly check for messages on the thread

                start_audio(
                    &decoding_active,
                    &volume_control_receiver,
                    &receiver,
                    &next_track_receiver,
                    data_channel,
                    &app_handle,
                );

                // on file (seek)

                // on pause

                // on play
            });
        }

        pub fn pause(&self) {
            let _ = &self
                .decoding_active
                .store(PAUSED, std::sync::atomic::Ordering::Relaxed);
        }

        pub fn resume(&self) {
            let _ = &self
                .decoding_active
                .store(ACTIVE, std::sync::atomic::Ordering::Relaxed);
            wake_all(self.decoding_active.as_ref());
        }

        pub async fn init_webrtc(
            &self,
            app_handle: AppHandle,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            // Create a MediaEngine object to configure the supported codec
            let mut m = MediaEngine::default();

            // Register default codecs
            m.register_default_codecs()?;
            println!("WEBRTC streamer");

            // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
            // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
            // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
            // for each PeerConnection.
            let mut registry = Registry::new();

            // Use the default set of Interceptors
            registry = register_default_interceptors(registry, &mut m)?;

            // Create the API object with the MediaEngine
            let api = APIBuilder::new()
                .with_media_engine(m)
                .with_interceptor_registry(registry)
                .build();

            // Prepare the configuration
            let config = RTCConfiguration {
                ..Default::default()
            };

            // Create a new RTCPeerConnection
            let peer_connection = Arc::new(api.new_peer_connection(config).await?);

            // Create a datachannel with label 'data'
            let data_channel = peer_connection.create_data_channel("data", None).await?;
            // &self.data_channel.on_close(Box::new(move || {}));

            // Set the handler for Peer connection state
            // This will notify you when the peer has connected/disconnected
            peer_connection.on_peer_connection_state_change(Box::new(
                move |s: RTCPeerConnectionState| {
                    println!("Peer Connection State has changed: {s}");

                    if s == RTCPeerConnectionState::Failed {
                        // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                        // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                        // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                        println!("Peer Connection has gone to failed exiting");
                    }

                    Box::pin(async {})
                },
            ));

            // Listen for ICE candidates
            peer_connection.on_ice_candidate(Box::new(move |c| {
                println!("on_ice_candidate {:?}", c);
                if let Some(cand) = c {
                    // let candidate = serde_json::to_string(&cand.to_json().unwrap());

                    let _ =
                        app_handle.emit_all("webrtc-icecandidate-client", &cand.to_json().unwrap());
                }
                Box::pin(async {})
            }));

            // Set the new peer connection and data channel
            if let Ok(mut conn) = self.peer_connection.try_lock() {
                conn.replace(peer_connection);
            }

            // Set the new datachannel
            if let Ok(mut dc) = self.data_channel.try_lock() {
                dc.replace(data_channel);
            }

            Ok(())
        }

        pub async fn handle_ice_candidate(
            self,
            candidate: Option<&str>,
        ) -> Result<(), Box<dyn std::error::Error + Send>> {
            println!("handle_ice_candidate {:?}", candidate);
            if let Some(candidate) = candidate {
                let parsed: RTCIceCandidateInit = serde_json::from_str(candidate).unwrap();

                if let Ok(pc_mutex) = self.peer_connection.try_lock() {
                    if let Some(pc) = pc_mutex.clone().or(None) {
                        pc.add_ice_candidate(parsed).await;
                    }
                }
            }
            Ok(())
        }

        pub async fn handle_signal(self, answer: Option<&str>) -> Option<RTCSessionDescription> {
            // self.peer_connection.close()
            // Apply the answer as the remote description
            // self.peer_connection.set_remote_description(answer).await?;
            if let Some(answer) = answer {
                let parsed_description: RTCSessionDescription =
                    serde_json::from_str(answer).unwrap();
                println!("handle_signal {:?}", parsed_description);

                if let Ok(pc_mutex) = self.peer_connection.try_lock() {
                    if let Some(pc) = pc_mutex.clone().or(None) {
                        let _ = pc.set_remote_description(parsed_description).await;
                        let ans = Some(pc.create_answer(None).await.unwrap());
                        let _ = pc.set_local_description(ans.clone().unwrap()).await;
                        return ans;
                    }
                }
            }
            None
        }

        pub async fn reset(&self) {
            println!("Resetting streamer");

            if let Ok(dc_mutex) = self.data_channel.try_lock() {
                if let Some(dc) = dc_mutex.clone().or(None) {
                    println!("Closing data channel...");
                    match dc.close().await {
                        Ok(()) => {
                            println!("Closed data channel");
                        }
                        Err(err) => {
                            println!("Error closing data channel{}", err);
                        }
                    }
                }
            }

            if let Ok(pc_mutex) = self.peer_connection.try_lock() {
                if let Some(pc) = pc_mutex.clone().or(None) {
                    println!("Closing peer connection...");
                    match pc.close().await {
                        Ok(()) => {
                            println!("Closed peer connection");
                        }
                        Err(err) => {
                            println!("Error closing peer connection {}", err);
                        }
                    }
                }
            }
        }

        // /**
        //  * Grab the PCM samples to send to the webview.
        //  */
        // pub fn get_stream_response(
        //     path: &str,
        //     boundary_id: &Arc<Mutex<i32>>,
        // ) -> Result<tauri::http::Response, Box<dyn std::error::Error>> {
        //     println!("File chunk handler:path {:?}", &path);
        //     let mut file = std::fs::File::open(&path)?;

        //     // get file length
        //     let len = {
        //         let old_pos = file.stream_position()?;
        //         let len = file.seek(SeekFrom::End(0))?;
        //         file.seek(SeekFrom::Start(old_pos))?;
        //         len
        //     };

        //     println!("File chunk handler: Length: {:?}", len);
        // }
    }

    // Peeking channel receiver without consuming
    // From https://stackoverflow.com/questions/59447577/how-to-peek-on-channels-without-blocking-and-still-being-able-to-detect-hangup
    pub struct TryIterResult<'a, T: 'a> {
        rx: &'a Receiver<T>,
    }

    pub fn try_iter_result<'a, T>(rx: &'a MutexGuard<Receiver<T>>) -> TryIterResult<'a, T> {
        TryIterResult { rx: &rx }
    }

    impl<'a, T> Iterator for TryIterResult<'a, T> {
        type Item = Result<T, TryRecvError>;

        fn next(&mut self) -> Option<Result<T, TryRecvError>> {
            match self.rx.try_recv() {
                Ok(data) => Some(Ok(data)),
                Err(TryRecvError::Empty) => Some(Err(TryRecvError::Empty)),
                Err(TryRecvError::Disconnected) => None,
            }
        }
    }

    pub fn start_audio(
        decoding_active: &Arc<AtomicU32>,
        volume_control_receiver: &Arc<Mutex<Receiver<VolumeControlEvent>>>,
        player_control_receiver: &Arc<Mutex<Receiver<PlayerControlEvent>>>,
        next_track_receiver: &Arc<Mutex<Receiver<StreamFileRequest>>>,
        data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
        app_handle: &AppHandle,
    ) {
        let decoding_active = decoding_active.clone();
        let vol_receiver = volume_control_receiver.clone();

        decoding_active.store(ACTIVE, std::sync::atomic::Ordering::Relaxed);

        wake_all(decoding_active.as_ref());

        decode_loop(
            vol_receiver,
            player_control_receiver,
            next_track_receiver,
            decoding_active,
            data_channel,
            app_handle,
        );
    }

    fn decode_loop(
        volume_control_receiver: Arc<Mutex<Receiver<VolumeControlEvent>>>,
        player_control_receiver: &Arc<Mutex<Receiver<PlayerControlEvent>>>,
        next_track_receiver: &Arc<Mutex<Receiver<StreamFileRequest>>>,
        decoding_active: Arc<AtomicU32>,
        data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
        app_handle: &AppHandle,
    ) {
        // These will be reset when changing tracks
        let mut path_str: Option<String> = None;
        let mut seek = None;
        let mut end_pos = None; // for loop region
        let mut volume = None;
        let mut file = Arc::new(Mutex::new(None));

        let mut spec: SignalSpec = SignalSpec::new_with_layout(44100, Layout::Stereo);
        let mut duration: u64 = 512;
        let mut previous_sample_rate = 44100;

        let (playback_state_sender, playback_state_receiver) = std::sync::mpsc::channel();
        let (reset_control_sender, reset_control_receiver) = std::sync::mpsc::channel();
        let (sender_sample_offset, receiver_sample_offset) = std::sync::mpsc::channel();
        let sample_offset_receiver = Arc::new(Mutex::new(receiver_sample_offset));

        let playback_state = Arc::new(Mutex::new(playback_state_receiver));
        let reset_control = Arc::new(Mutex::new(reset_control_receiver));

        let mut audio_output: Option<
            Result<Arc<Mutex<dyn AudioOutput>>, output::AudioOutputError>,
        > = None;

        let mut cancel_token;

        let mut is_transition = false; // This is set to speed up decoding during transition (last 5s)
        let mut is_reset = true; // Whether the playback has been 'reset' (i.e double click on new track, next btn)

        // Loop here!
        loop {
            cancel_token = CancellationToken::new();
            if let None = path_str {
                is_transition = false;
                let event = player_control_receiver
                    .try_lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(1));

                println!("audio: waiting for file! {:?}", event);
                if let Ok(result) = event {
                    match result {
                        PlayerControlEvent::StreamFile(request) => {
                            println!("audio: got file request! {:?}", request);
                            path_str.replace(request.path.unwrap());
                            seek.replace(request.seek.unwrap());
                            volume.replace(request.volume.unwrap());
                            file.try_lock().unwrap().replace(request.file_info.unwrap());
                        }
                        PlayerControlEvent::LoopRegion(request) => {
                            println!("audio: loop region! {:?}", request);
                            seek.replace(request.start_pos.unwrap());
                            end_pos.replace(request.end_pos.unwrap());
                            cancel_token.cancel();
                        }
                    }
                }
            } else if let Some(ref p) = path_str.clone() {
                let path = Path::new(p.as_str());
                let file_info = file.try_lock().unwrap().clone().unwrap();

                // Create a hint to help the format registry guess what format reader is appropriate.
                let mut hint = Hint::new();
                let source = Box::new(File::open(path).unwrap());
                println!("source {:?}", source);

                // Provide the file extension as a hint.
                println!("extension: {:?}", path.extension());
                if let Some(extension) = path.extension() {
                    if let Some(extension_str) = extension.to_str() {
                        hint.with_extension(extension_str);
                    }
                }

                // Create the media source stream using the boxed media source from above.
                let mut mss = MediaSourceStream::new(source, Default::default());

                // Use the default options for format readers other than for gapless playback.
                let format_opts = FormatOptions {
                    enable_gapless: true,
                    ..Default::default()
                };

                // Use the default options for metadata readers.
                let metadata_opts: MetadataOptions = MetadataOptions {
                    limit_metadata_bytes: symphonia::core::meta::Limit::Maximum(50),
                    limit_visual_bytes: symphonia::core::meta::Limit::Maximum(0),
                };

                // Get the value of the track option, if provided.
                let track: Option<Track> = None;
                println!("probing {:?}", hint);
                println!("opts {:?}", format_opts);
                println!("meta {:?}", metadata_opts);

                let probe_result = get_probe().format(&hint, mss, &format_opts, &metadata_opts);
                println!("probe format {:?}", probe_result.is_ok());

                if probe_result.is_err() {
                    path_str = None;
                    continue;
                }

                path_str = None;

                let mut reader = probe_result.unwrap().format;

                let track = reader.default_track().unwrap().clone();

                if let Some(frames) = track.codec_params.n_frames {
                    app_handle.emit_all("file-samples", frames);
                }

                let mut track_id = track.id;

                // If seeking, seek the reader to the time or timestamp specified and get the timestamp of the
                // seeked position. All packets with a timestamp < the seeked position will not be played.
                //
                // Note: This is a half-baked approach to seeking! After seeking the reader, packets should be
                // decoded and *samples* discarded up-to the exact *sample* indicated by required_ts. The
                // current approach will discard excess samples if seeking to a sample within a packet.
                let seek_ts = if let Some(sk) = seek {
                    let seek_to = SeekTo::Time {
                        time: Time::from(sk),
                        track_id: Some(track_id),
                    };

                    // Attempt the seek. If the seek fails, ignore the error and return a seek timestamp of 0 so
                    // that no samples are trimmed.
                    match reader.seek(symphonia::core::formats::SeekMode::Accurate, seek_to) {
                        Ok(seeked_to) => seeked_to.required_ts,
                        Err(ResetRequired) => {
                            track_id = first_supported_track(reader.tracks()).unwrap().id;
                            0
                        }
                        Err(err) => {
                            // Don't give-up on a seek error.
                            warn!("seek error: {}", err);
                            0
                        }
                    }
                } else {
                    // If not seeking, the seek timestamp is 0.
                    0
                };

                println!("codec params: {:?}", &track.codec_params);

                // Create a decoder for the track.
                let mut decoder = symphonia::default::get_codecs()
                    .make(&track.codec_params, &DecoderOptions { verify: false })
                    .unwrap();

                let spec = SignalSpec {
                    rate: decoder.codec_params().sample_rate.unwrap(),
                    channels: decoder.codec_params().channels.unwrap(),
                };

                let mut should_reset_audio = false;
                let mut new_duration = 1152;

                let max_frames = decoder.codec_params().max_frames_per_packet;
                if let Some(dur) = max_frames {
                    new_duration = dur;
                }

                // Check if audio device changed
                let host = cpal::default_host();
                let output_device = host.default_output_device();

                // If we have a default audio device (we always should, but just in case)
                // we check if the track spec differs from the output device
                // if it does - resample the decoded audio using Symphonia.

                // Check if track sample rate differs from current OS config
                if let Some(device) = output_device {
                    println!("cpal: Default device {:?}", device.name());
                    // Only resample when audio device doesn't support file sample rate
                    // so we can't switch the device rate to match.
                    let supports_sample_rate = device
                        .supported_output_configs()
                        .unwrap()
                        .find(|c| {
                            return c
                                .try_with_sample_rate(cpal::SampleRate(spec.rate))
                                .is_some();
                        })
                        .is_some();
                    // If sample rate changed - reinit the audio device with the new sample rate
                    // (if this sample rate isn't supported, it will be resampled)
                    should_reset_audio = supports_sample_rate && spec.rate != previous_sample_rate;
                }

                previous_sample_rate = spec.rate;

                if (audio_output.is_none() || should_reset_audio) {
                    // Try to open the audio output.
                    audio_output.replace(output::try_open(
                        spec,
                        volume_control_receiver.clone(),
                        sample_offset_receiver.clone(),
                        playback_state.clone(),
                        reset_control.clone(),
                        data_channel.clone(),
                        volume.clone(),
                        app_handle.clone(),
                    ));
                }

                let mut last_sent_time = Instant::now();

                if !is_transition {
                    let _ = reset_control_sender.send(true);
                    let _ = sender_sample_offset.send(SampleOffsetEvent {
                        sample_offset: Some(
                            seek_ts * track.codec_params.channels.unwrap().count() as u64,
                        ),
                    });
                }

                let end_pos_frame_idx = if end_pos.is_some() {
                    (end_pos.unwrap() * track.codec_params.sample_rate.unwrap() as f64) as u64
                } else {
                    0
                };

                let receiver = player_control_receiver.try_lock().unwrap();

                if let Some(ref audio) = audio_output {
                    if let Ok(ao) = audio {
                        if let Ok(mut guard) = ao.try_lock() {
                            let mut transition_time = Instant::now();
                            let mut started_transition = false;

                            // Resampling stuff
                            guard.resume();
                            guard.update_resampler(spec, new_duration);

                            // Until all samples have been flushed - don't start decoding
                            // Keep checking until all samples have been played (buffer is empty)
                            if is_reset {
                                while guard.has_remaining_samples() {
                                    guard.flush();
                                    println!("Buffer is not empty yet, waiting to continue...");
                                }
                                println!("Buffer is now empty. Continuing decoding...");
                            }

                            // Decode all packets, ignoring all decode errors.
                            let result = loop {
                                let event = receiver.try_recv();
                                // debug!("audio: waiting for event {:?}", event);
                                if let Ok(result) = event {
                                    match result {
                                        PlayerControlEvent::StreamFile(request) => {
                                            println!(
                                                "audio: source changed during decoding! {:?}",
                                                request
                                            );
                                            path_str.replace(request.path.unwrap());
                                            seek.replace(request.seek.unwrap());
                                            end_pos = None;
                                            volume.replace(request.volume.unwrap());
                                            file.try_lock()
                                                .unwrap()
                                                .replace(request.file_info.unwrap());
                                            cancel_token.cancel();
                                            guard.flush();
                                            is_reset = true;
                                        }
                                        PlayerControlEvent::LoopRegion(request) => {
                                            println!("audio: loop region! {:?}", request);
                                            if (request.enabled.unwrap()) {
                                                seek.replace(request.start_pos.unwrap());
                                                end_pos.replace(request.end_pos.unwrap());
                                            } else {
                                                end_pos = None;
                                            }
                                            cancel_token.cancel();
                                            guard.flush();
                                        }
                                    }
                                }

                                let mut is_paused = false;
                                if (decoding_active.load(std::sync::atomic::Ordering::Relaxed)
                                    == PAUSED)
                                {
                                    is_paused = true;
                                    println!("Sending paused state to output");
                                    guard.pause();
                                    let _ = playback_state_sender.send(false);
                                    app_handle.emit_all("paused", {});
                                }

                                // waits while the value is PAUSED (0)
                                atomic_wait::wait(&decoding_active, PAUSED);

                                app_handle.emit_all("playing", {});

                                if (is_paused) {
                                    is_paused = false;
                                    guard.resume();
                                    let ctrl_event = receiver.try_recv();
                                    if let Ok(result) = ctrl_event {
                                        match result {
                                            PlayerControlEvent::StreamFile(request) => {
                                                println!(
                                                    "audio: source changed during decoding! {:?}",
                                                    request
                                                );
                                                path_str.replace(request.path.unwrap());
                                                seek.replace(request.seek.unwrap());
                                                end_pos = None;
                                                volume.replace(request.volume.unwrap());
                                                file.try_lock()
                                                    .unwrap()
                                                    .replace(request.file_info.unwrap());
                                                cancel_token.cancel();
                                                guard.flush();
                                                is_reset = true;
                                            }
                                            PlayerControlEvent::LoopRegion(request) => {
                                                println!("audio: loop region! {:?}", request);
                                                if (request.enabled.unwrap()) {
                                                    seek.replace(request.start_pos.unwrap());
                                                    end_pos.replace(request.end_pos.unwrap());
                                                } else {
                                                    end_pos = None;
                                                }
                                                cancel_token.cancel();
                                                guard.flush();
                                            }
                                        }
                                    }
                                }
                                let _ = playback_state_sender.send(true);

                                if cancel_token.is_cancelled() {
                                    break Ok(());
                                }

                                let packet = match reader.next_packet() {
                                    Ok(packet) => packet,
                                    Err(err) => break Err(err),
                                };

                                // If the packet does not belong to the selected track, skip over it.
                                if packet.track_id() != track_id {
                                    continue;
                                }

                                // Loop region mode: If this packet is past the loop region,
                                // seek the reader back to the start point
                                if (end_pos.is_some() && packet.ts > end_pos_frame_idx) {
                                    let seek_to = SeekTo::Time {
                                        time: Time::from(seek.unwrap()),
                                        track_id: Some(track_id),
                                    };
                                    match reader
                                        .seek(symphonia::core::formats::SeekMode::Accurate, seek_to)
                                    {
                                        Ok(seeked_to) => seeked_to.required_ts,
                                        Err(ResetRequired) => {
                                            // Don't give-up on a seek error.
                                            warn!("reset required:");
                                            0
                                        }
                                        Err(err) => {
                                            // Don't give-up on a seek error.
                                            warn!("seek error: {}", err);
                                            0
                                        }
                                    };
                                    is_transition = true; // To delay sending sample offset by 5s
                                }

                                // Decode the packet into audio samples.
                                match decoder.decode(&packet) {
                                    Ok(_decoded) => {
                                        last_sent_time = Instant::now();

                                        /*
                                        The transition is 5 seconds long, since that is the size of the buffer.
                                        So decoding starts 5 seconds before playback, and we can delay the song
                                        change in the UI by this time.
                                         */
                                        if is_transition && !started_transition {
                                            started_transition = true;
                                            transition_time = last_sent_time;
                                        } else if is_transition && started_transition {
                                            if transition_time.elapsed().as_secs() >= 5 {
                                                if (end_pos.is_some()) {
                                                    let _ = sender_sample_offset.send(
                                                        SampleOffsetEvent {
                                                            sample_offset: Some(
                                                                seek_ts
                                                                    * file_info.channels.unwrap()
                                                                        as u64,
                                                            ),
                                                        },
                                                    );
                                                }

                                                if let Some(song) =
                                                    crate::metadata::extract_metadata(&Path::new(
                                                        &p.clone().as_str(),
                                                    ))
                                                {
                                                    app_handle.emit_all("song_change", Some(song));

                                                    let _ = reset_control_sender.send(true);
                                                    let _ = sender_sample_offset.send(
                                                        SampleOffsetEvent {
                                                            sample_offset: Some(
                                                                seek_ts
                                                                    * track
                                                                        .codec_params
                                                                        .channels
                                                                        .unwrap()
                                                                        .count()
                                                                        as u64,
                                                            ),
                                                        },
                                                    );
                                                } else {
                                                    println!("ERROR getting song");
                                                }
                                                is_transition = false;
                                                started_transition = false;
                                            }
                                        }

                                        /*
                                        Write packet to audio ring buffer here
                                        Because the audio playback uses the ringbuffer, we are effectively
                                        "slowing down" decoding to allow the audio stream to read from the
                                        buffer as it's playing.
                                         */
                                        if (!cancel_token.is_cancelled()) {
                                            // Write the decoded audio samples to the audio output if the presentation timestamp
                                            // for the packet is >= the seeked position (0 if not seeking).
                                            if packet.ts() >= seek_ts {
                                                guard.write(_decoded);
                                            }
                                        }

                                        continue;
                                    }
                                    Err(symphonia::core::errors::Error::DecodeError(err)) => {
                                        println!("decode error: {}", err)
                                    }
                                    Err(err) => break Err(err),
                                }
                            };

                            // Return if a fatal error occured.
                            let _ = match result {
                                Err(symphonia::core::errors::Error::IoError(err))
                                    if err.kind() == std::io::ErrorKind::UnexpectedEof
                                        && err.to_string() == "end of stream" =>
                                {
                                    println!("End of stream!!");
                                    let mut next_track = None;
                                    while let Ok(value) =
                                        next_track_receiver.try_lock().unwrap().try_recv()
                                    {
                                        println!("received {:?}", value);
                                        next_track.replace(value);
                                    }
                                    if let Some(request) = next_track {
                                        if let Some(path) = request.path.clone() {
                                            is_transition = true;
                                            println!("player: next track received! {:?}", request);
                                            path_str.replace(path);
                                            seek.replace(request.seek.unwrap());
                                            volume.replace(request.volume.unwrap());
                                            file.try_lock()
                                                .unwrap()
                                                .replace(request.file_info.unwrap());
                                            is_reset = false;
                                        } else {
                                            println!("player: nothing else in the queue");

                                            // Keep checking until all samples have been played (buffer is empty)
                                            while guard.has_remaining_samples() {
                                                println!(
                                                    "Buffer is not empty yet, waiting to pause..."
                                                );
                                                thread::sleep(Duration::from_millis(500));
                                            }
                                            println!("Buffer is now empty. Pausing stream...");
                                            guard.pause();
                                            app_handle.emit_all("stopped", Some(0.0f64));
                                        }
                                    }
                                    // Do not treat "end of stream" as a fatal error. It's the currently only way a
                                    // format reader can indicate the media is complete.
                                    Ok(())
                                }
                                _ => result,
                            };
                        }
                    }
                }
            };
        }
        // Finalize the decoder and return the verification result if it's been enabled.
        // do_verification(decoder.finalize())
    }

    fn first_supported_track(tracks: &[Track]) -> Option<&Track> {
        tracks
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
    }

    pub fn get_peaks(
        event: GetWaveformRequest,
        app_handle: &AppHandle,
        cancel_token: CancellationToken,
    ) -> Result<Vec<f32>, symphonia::core::errors::Error> {
        let binding = event.path.unwrap();
        let path = Path::new(binding.as_str());

        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();
        let source = Box::new(File::open(path).unwrap());
        println!("source {:?}", source);

        // Provide the file extension as a hint.
        println!("extension: {:?}", path.extension());
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        // Create the media source stream using the boxed media source from above.
        let mut mss = MediaSourceStream::new(source, Default::default());

        // Use the default options for format readers other than for gapless playback.
        let format_opts = FormatOptions {
            enable_gapless: false,
            ..Default::default()
        };

        // Use the default options for metadata readers.
        let metadata_opts: MetadataOptions = MetadataOptions {
            limit_metadata_bytes: symphonia::core::meta::Limit::Maximum(50),
            limit_visual_bytes: symphonia::core::meta::Limit::Maximum(0),
        };

        // Get the value of the track option, if provided.
        let track: Option<Track> = None;
        println!("probing {:?}", hint);
        println!("opts {:?}", format_opts);
        println!("meta {:?}", metadata_opts);

        let probe_result = get_probe().format(&hint, mss, &format_opts, &metadata_opts);
        println!("probe format {:?}", probe_result.is_ok());

        if probe_result.is_err() {
            return Err(probe_result.err().unwrap());
        }
        let mut reader = probe_result.unwrap().format;

        let track = reader.default_track().unwrap().clone();

        let mut track_id = track.id;

        let seek_ts = 0;

        println!("codec params: {:?}", &track.codec_params);

        // Create a decoder for the track.
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions { verify: false })
            .unwrap();

        let mut spec_changed = false;
        let mut new_spec = SignalSpec::new_with_layout(44100, Layout::Stereo);
        let mut new_duration = 1152;

        let expected_peaks_size = (track.codec_params.n_frames.unwrap()
            * new_spec.channels.count() as u64
            / 256) as usize;
        let mut peaks: Vec<f32> = Vec::new();

        let mut avg = 0f32;
        let mut total = 0f32;
        let mut count = 0;
        let mut total_count = 0;
        let mut n_frames = 0;
        let mut n_adds = 0;

        let result = loop {
            let packet = match reader.next_packet() {
                Ok(packet) => packet,
                Err(err) => break Err(err),
            };

            // If the packet does not belong to the selected track, skip over it.
            if packet.track_id() != track_id {
                continue;
            }
            // Decode the packet into audio samples.
            match decoder.decode(&packet) {
                Ok(_decoded) => {
                    if (cancel_token.is_cancelled()) {
                        break Err(symphonia::core::errors::Error::LimitError("cancelled"));
                    }
                    // Create a raw sample buffer that matches the parameters of the decoded audio buffer.
                    let mut sample_buf =
                        SampleBuffer::<f32>::new(_decoded.capacity() as u64, *_decoded.spec());

                    // Copy the contents of the decoded audio buffer into the sample buffer whilst performing
                    // any required conversions.
                    sample_buf.copy_interleaved_ref(_decoded);
                    let mut i = 0;
                    sample_buf.samples().iter().for_each(|f| {
                        if i % 2 == 0 {
                            total += f;
                            count += 1;
                            n_frames += 1;
                        }
                        if (n_frames % 256 == 0) {
                            avg = total / count as f32;
                            peaks.push(avg);
                            total = 0f32;
                            count = 0;
                            n_adds += 1;
                        }
                        i += 1;
                    });

                    total_count += 1;
                    if (total_count > 100) {
                        total_count = 0;
                        let len = expected_peaks_size.saturating_sub(peaks.len());
                        // println!("expected peaks size: {}, len: {}, n_adds: {}", expected_peaks_size, peaks.len(), n_adds);
                        let cln = [peaks.clone().as_slice(), vec![0f32; len].as_slice()].concat();
                        app_handle.emit_all("waveform", GetWaveformResponse { data: Some(cln) });
                    }

                    // Get waveform here
                    continue;
                }
                Err(symphonia::core::errors::Error::DecodeError(err)) => {
                    println!("decode error: {}", err)
                }
                Err(err) => break Err(err),
            }
        };

        // Return if a fatal error occured.
        let res = match result {
            Err(symphonia::core::errors::Error::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof
                    && err.to_string() == "end of stream" =>
            {
                println!("End of stream!!");
                println!(
                    "Number of frames: {} (actual), {} (expected)",
                    n_frames,
                    track.codec_params.n_frames.unwrap()
                );
                // Do not treat "end of stream" as a fatal error. It's the currently only way a
                // format reader can indicate the media is complete.
                Ok(peaks)
            }
            _ => result,
        };
        res
    }
}
