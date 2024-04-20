// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod file_streamer {
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::Instant;

    use std::cell::Cell;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs::File;
    use std::io::{SeekFrom, Write};
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::path::Path;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::Arc;
    use std::{cmp, thread};

    use atomic_wait::{wake_all, wake_one};
    use bytes::Bytes;
    use log::{debug, error, info, log, warn};
    use symphonia::core::audio::{Channels, Layout, RawSampleBuffer, SignalSpec};
    use symphonia::core::codecs::{DecoderOptions, FinalizeResult, CODEC_TYPE_NULL};
    use symphonia::core::errors::Error::ResetRequired;
    use symphonia::core::formats::{FormatOptions, FormatReader, SeekTo, Track};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::{Hint, Probe, ProbeResult};
    use symphonia::core::units::Time;
    use symphonia::default::{get_probe, register_enabled_formats};
    use tauri::{AppHandle, Manager};
    use tokio::select;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, Duration};

    use ringbuffer::{AllocRingBuffer, RingBuffer};
    use tokio_util::sync::CancellationToken;
    use webrtc::api::interceptor_registry::register_default_interceptors;
    use webrtc::api::media_engine::MediaEngine;
    use webrtc::api::APIBuilder;
    use webrtc::data_channel::data_channel_message::DataChannelMessage;
    use webrtc::data_channel::data_channel_state::RTCDataChannelState;
    use webrtc::data_channel::RTCDataChannel;
    use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
    use webrtc::ice_transport::ice_server::RTCIceServer;
    use webrtc::interceptor::registry::Registry;
    use webrtc::peer_connection::configuration::RTCConfiguration;
    use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
    use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
    use webrtc::peer_connection::{math_rand_alpha, RTCPeerConnection};

    use crate::output::{self, AudioOutput};
    use crate::{
        FileInfo, FlowControlEvent, SampleOffsetEvent, StreamFileRequest, VolumeControlEvent,
    };

    enum SeekPosition {
        Time(f64),
        Timetamp(u64),
    }

    #[derive(Debug)]
    pub enum PlayerControlEvent {
        StreamFile(StreamFileRequest), // path, seekpos
        Play,
        Pause,
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
        pub prepare_transition_receiver: Arc<Mutex<Receiver<bool>>>,
        pub prepare_transition_sender: Sender<bool>,
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
                prepare_transition_receiver: Arc::new(Mutex::new(receiver_transition)),
                prepare_transition_sender: sender_transition,
            })
        }

        pub fn init(&self, app_handle: AppHandle) -> () {
            let receiver = self.player_control_receiver.clone();
            let next_track_receiver = self.next_track_receiver.clone();
            let cancel_tokens = self.cancel_tokens.clone();
            let decoding_active = self.decoding_active.clone();
            let volume_control_receiver = self.volume_control_receiver.clone();
            let prepare_transition_receiver = self.prepare_transition_receiver.clone();
            let data_channel = self.data_channel.clone();

            std::thread::spawn(move || {
                // AUDIO THREAD!
                // Constantly check for messages on the thread

                start_audio(
                    &decoding_active,
                    &volume_control_receiver,
                    &receiver,
                    &next_track_receiver,
                    &prepare_transition_receiver,
                    data_channel,
                    &app_handle,
                );

                // on file (seek)

                // on pause

                // on play
            });
        }

        pub fn pause(&self) {
            &self
                .decoding_active
                .store(PAUSED, std::sync::atomic::Ordering::Relaxed);
        }

        pub fn resume(&self) {
            &self
                .decoding_active
                .store(ACTIVE, std::sync::atomic::Ordering::Relaxed);
            wake_one(self.decoding_active.as_ref());
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
                        pc.set_remote_description(parsed_description).await;
                        let ans = Some(pc.create_answer(None).await.unwrap());
                        pc.set_local_description(ans.clone().unwrap()).await;
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

    pub fn start_audio(
        decoding_active: &Arc<AtomicU32>,
        volume_control_receiver: &Arc<Mutex<Receiver<VolumeControlEvent>>>,
        player_control_receiver: &Arc<Mutex<Receiver<PlayerControlEvent>>>,
        next_track_receiver: &Arc<Mutex<Receiver<StreamFileRequest>>>,
        prepare_transition_receiver: &Arc<Mutex<Receiver<bool>>>,
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
            prepare_transition_receiver,
            decoding_active,
            data_channel,
            app_handle,
        );
    }

    fn decode_loop(
        volume_control_receiver: Arc<Mutex<Receiver<VolumeControlEvent>>>,
        player_control_receiver: &Arc<Mutex<Receiver<PlayerControlEvent>>>,
        next_track_receiver: &Arc<Mutex<Receiver<StreamFileRequest>>>,
        prepare_transition_receiver: &Arc<Mutex<Receiver<bool>>>,
        decoding_active: Arc<AtomicU32>,
        data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
        app_handle: &AppHandle,
    ) {
        // Request will be None at first!

        // These will be reset when changing tracks
        let mut path_str: Option<String> = None;
        let mut seek = None;
        let mut volume = None;
        let mut file = Arc::new(Mutex::new(None));

        let mut spec: SignalSpec = SignalSpec::new_with_layout(44100, Layout::Stereo);
        let mut duration: u64 = 512;

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

        // Loop here!
        loop {
            cancel_token = CancellationToken::new();
            if let None = path_str {
                is_transition = false;
                let event = player_control_receiver
                    .try_lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(1));

                // println!("audio: waiting for file! {:?}", event);
                if let Ok(result) = event {
                    match result {
                        PlayerControlEvent::StreamFile(request) => {
                            println!("audio: got file request! {:?}", request);
                            path_str.replace(request.path.unwrap());
                            seek.replace(request.seek.unwrap());
                            volume.replace(request.volume.unwrap());
                            file.try_lock().unwrap().replace(request.file_info.unwrap());
                        }
                        PlayerControlEvent::Pause => {
                            // self.pause();
                        }
                        PlayerControlEvent::Play => {
                            // self.resume();
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
                    return;
                }
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
                let mut printed = false;

                // Create a decoder for the track.
                let mut decoder = symphonia::default::get_codecs()
                    .make(&track.codec_params, &DecoderOptions { verify: false })
                    .unwrap();

                let mut spec_changed = false;
                let mut new_spec = SignalSpec::new_with_layout(44100, Layout::Stereo);
                let mut new_duration = 1152;
                if let Some(channels) = decoder.codec_params().channels {
                    new_spec =
                        SignalSpec::new(decoder.codec_params().sample_rate.unwrap(), channels);
                    let max_frames = decoder.codec_params().max_frames_per_packet;

                    if let Some(new_dur) = max_frames {
                        new_duration = new_dur;
                        // Check if spec has changed, reinit audio
                        if (new_spec != spec) || new_dur != duration {
                            spec_changed = true;
                        }
                    }
                }

                if (audio_output.is_none() || spec_changed) {
                    // Try to open the audio output.
                    audio_output.replace(output::try_open(
                        new_spec,
                        new_duration,
                        volume_control_receiver.clone(),
                        sample_offset_receiver.clone(),
                        playback_state.clone(),
                        reset_control.clone(),
                        data_channel.clone(),
                        volume.clone(),
                        app_handle.clone(),
                    ));
                }

                /*
                We're sending the PCM as bytes in chunks of 10kb,
                which is just below the maximum limit on Chrome (16kb per message)
                */
                let mut byte_buf: AllocRingBuffer<u8> = AllocRingBuffer::new(1024 * 10); // 10KB
                let mut to_send = Bytes::new();
                let mut last_sent_time = Instant::now();
                let mut packet_counter = 0;
                let target_bitrate =
                    file_info.bit_depth.unwrap() as u32 * file_info.sample_rate.unwrap();
                println!("Target kbits/second: {}", target_bitrate as f64 / 1000f64);
                let max_packet_size = &decoder.codec_params().max_frames_per_packet;
                let max_block_size = &decoder.codec_params().frames_per_block;

                let mut fastest_send_interval = Duration::new(0, 0);
                let mut slowest_send_interval = Duration::new(0, 0);

                println!("Max packet size: {:?} frames", max_packet_size);
                println!("Max block size: {:?} frames", max_block_size);
                // data_channel.on_buffered_amount_low(|f| {

                // });
                let mut is_first_packet = true;
                let mut send_interval = Duration::from_millis(1); // Initial send interval

                let mut last_ticker_time = Instant::now();

                if !is_transition {
                    reset_control_sender.send(true);
                    sender_sample_offset.send(SampleOffsetEvent {
                        sample_offset: Some(seek_ts * file_info.channels.unwrap() as u64),
                    });
                }

                let receiver = player_control_receiver.try_lock().unwrap();

                if let Some(ref audio) = audio_output {
                    if let Ok(ao) = audio {
                        if let Ok(mut guard) = ao.try_lock() {
                            let mut transition_time = Instant::now();
                            let mut started_transition = false;

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
                                            volume.replace(request.volume.unwrap());
                                            file.try_lock()
                                                .unwrap()
                                                .replace(request.file_info.unwrap());
                                            cancel_token.cancel();
                                            guard.flush();
                                        }
                                        PlayerControlEvent::Pause => {
                                            // self.pause();
                                        }
                                        PlayerControlEvent::Play => {
                                            // self.resume();
                                        }
                                    }
                                }

                                if (decoding_active.load(std::sync::atomic::Ordering::Relaxed)
                                    == PAUSED)
                                {
                                    playback_state_sender.send(false);
                                }

                                atomic_wait::wait(&decoding_active, PAUSED); // waits while the value is PAUSED (0)
                                playback_state_sender.send(true);

                                if cancel_token.is_cancelled() {
                                    break Ok(());
                                }
                                let packet = match reader.next_packet() {
                                    Ok(packet) => packet,
                                    Err(err) => break Err(err),
                                };
                                packet_counter += 1;

                                // if !printed {
                                //     println!("packet size: {:?}", packet.buf().len());
                                // }
                                // If the packet does not belong to the selected track, skip over it.
                                if packet.track_id() != track_id {
                                    continue;
                                }

                                // Decode the packet into audio samples.
                                match decoder.decode(&packet) {
                                    Ok(_decoded) => {
                                        // If the audio output is not open, try to open it.
                                        // Get the audio buffer specification. This is a description of the decoded
                                        // audio buffer's sample format and sample rate.
                                        let spec = *_decoded.spec();

                                        // Get the capacity of the decoded buffer. Note that this is capacity, not
                                        // length! The capacity of the decoded buffer is constant for the life of the
                                        // decoder, but the length is not.
                                        let duration = _decoded.capacity() as u64;
                                        let frames = _decoded.frames();
                                        let frames = _decoded.frames();
                                        // Create a raw sample buffer that matches the parameters of the decoded audio buffer.
                                        let mut sample_buf = RawSampleBuffer::<f32>::new(
                                            _decoded.capacity() as u64,
                                            *_decoded.spec(),
                                        );
                                        let frames = _decoded.frames();
                                        // Create a raw sample buffer that matches the parameters of the decoded audio buffer.
                                        let mut sample_buf = RawSampleBuffer::<f32>::new(
                                            _decoded.capacity() as u64,
                                            *_decoded.spec(),
                                        );

                                        // Copy the contents of the decoded audio buffer into the sample buffer whilst performing
                                        // any required conversions.
                                        // sample_buf.copy_interleaved_ref(_decoded);

                                        // The interleaved f32 samples can be accessed as a slice of bytes as follows.
                                        // sample_buf.as_bytes().to_vec().iter().for_each(|b| {
                                        //     byte_buf.push(*b);
                                        //     if (byte_buf.is_full()) {
                                        //         to_send = Bytes::from(byte_buf.to_vec());
                                        //         // Clear the buffer to make space for remaining bytes
                                        //         byte_buf.clear();
                                        //     }
                                        // });

                                        // Calculate decode/send rate
                                        let elapsed = last_sent_time.elapsed();

                                        if (packet_counter % 20 == 0) {
                                            if (is_first_packet) {
                                                // Target time between packets = bitrate
                                                let frames: u32 = u32::try_from(frames).unwrap();
                                                let bits =
                                                    frames * file_info.bit_depth.unwrap() as u32;

                                                // Never send frames slower than 0.75x playback!
                                                slowest_send_interval = Duration::from_secs_f64(
                                                    bits as f64 / target_bitrate as f64,
                                                )
                                                .mul_f64(1.1f64);

                                                // Never send faster than 3x playback
                                                fastest_send_interval = Duration::from_secs_f64(
                                                    bits as f64 / target_bitrate as f64,
                                                )
                                                .div_f64(3.0f64);
                                                send_interval = slowest_send_interval.clone();
                                                println!(
                                                    "decode (interval): {} every {}s",
                                                    bits,
                                                    slowest_send_interval.as_secs_f64()
                                                );
                                                is_first_packet = false;
                                            }

                                            // Target time between packets = bitrate
                                            let frames: u32 = u32::try_from(frames).unwrap();
                                            let bits = frames * file_info.bit_depth.unwrap() as u32;

                                            let decode_rate_micros =
                                                bits as f64 / elapsed.as_micros() as f64;
                                            let decode_rate_second =
                                                decode_rate_micros * 1000000f64;
                                            println!(
                                                "decode rate (seconds): {:.2} kb/s",
                                                decode_rate_second / 1000f64
                                            );
                                        }

                                        last_sent_time = Instant::now();

                                        if is_transition && !started_transition {
                                            started_transition = true;
                                            transition_time = last_sent_time;
                                        } else if is_transition && started_transition {
                                            if transition_time.elapsed().as_secs() >= 5 {
                                                // Check if 5s have passed
                                                if let Some(song) =
                                                    crate::metadata::extract_metadata(&Path::new(
                                                        &p.clone().as_str(),
                                                    ))
                                                {
                                                    app_handle.emit_all("song_change", Some(song));

                                                    reset_control_sender.send(true);
                                                    sender_sample_offset.send(SampleOffsetEvent {
                                                        sample_offset: Some(
                                                            seek_ts
                                                                * file_info.channels.unwrap()
                                                                    as u64,
                                                        ),
                                                    });
                                                }
                                                is_transition = false;
                                                started_transition = false;
                                            }
                                        }

                                        // Send packet to JS here
                                        if (!cancel_token.is_cancelled()) {
                                            if !is_transition {
                                                // thread::sleep(send_interval);
                                            }
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

                                    // TODO: Gapless here
                                    let next_track =
                                        next_track_receiver.try_lock().unwrap().try_recv();
                                    if let Ok(request) = next_track {
                                        let path = request.path.clone().unwrap();

                                        is_transition = true;
                                        println!("audio: next track received! {:?}", request);
                                        path_str.replace(request.path.unwrap());
                                        seek.replace(request.seek.unwrap());
                                        volume.replace(request.volume.unwrap());
                                        file.try_lock()
                                            .unwrap()
                                            .replace(request.file_info.unwrap());
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

    fn do_verification(finalization: FinalizeResult) -> symphonia::core::errors::Result<i32> {
        match finalization.verify_ok {
            Some(is_ok) => {
                // Got a verification result.
                println!("verification: {}", if is_ok { "passed" } else { "failed" });

                Ok(i32::from(!is_ok))
            }
            // Verification not enabled by user, or unsupported by the codec.
            _ => Ok(0),
        }
    }
}
