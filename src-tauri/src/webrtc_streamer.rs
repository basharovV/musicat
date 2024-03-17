// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod web_rtcstreamer {
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
    use log::{error, info, warn};
    use symphonia::core::audio::RawSampleBuffer;
    use symphonia::core::codecs::{DecoderOptions, FinalizeResult, CODEC_TYPE_NULL};
    use symphonia::core::errors::Error::ResetRequired;
    use symphonia::core::formats::{FormatOptions, FormatReader, SeekTo, Track};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::{Hint, Probe};
    use symphonia::core::units::Time;
    use symphonia::default::register_enabled_formats;
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

    use crate::{FileInfo, FlowControlEvent, StreamFileRequest};

    enum SeekPosition {
        Time(f64),
        Timetamp(u64),
    }

    const PAUSED: u32 = 0;
    const ACTIVE: u32 = 1;

    #[derive(Clone)]
    pub struct AudioStreamer<'a> {
        pub peer_connection: Arc<Mutex<Option<Arc<RTCPeerConnection>>>>,
        pub data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
        pub is_open: Arc<AtomicBool>,
        pub path: Arc<String>,
        pub cancel_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
        phantom: PhantomData<&'a RTCPeerConnection>,
        phantom2: PhantomData<&'a RTCDataChannel>,
        pub flow_control_receiver: Arc<Mutex<Receiver<FlowControlEvent>>>,
        pub flow_control_sender: Sender<FlowControlEvent>,
        pub decoding_active: Arc<AtomicU32>,
    }

    impl<'a> AudioStreamer<'a> {
        pub async fn create() -> Result<AudioStreamer<'a>, Box<dyn std::error::Error + Send + Sync>>
        {
            let (sender_tx, receiver_rx) = std::sync::mpsc::channel();

            Ok(AudioStreamer {
                peer_connection: Arc::new(Mutex::new(None)),
                data_channel: Arc::new(Mutex::new(None)),
                is_open: Arc::new(AtomicBool::new(false)),
                path: Arc::new(String::new()),
                cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
                phantom: PhantomData,
                phantom2: PhantomData,
                flow_control_receiver: Arc::new(Mutex::new(receiver_rx)),
                flow_control_sender: sender_tx,
                decoding_active: Arc::new(AtomicU32::new(ACTIVE)),
            })
        }

        pub async fn init(
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

        pub fn stream_file(&self, request: StreamFileRequest, app_handle: AppHandle) {
            let path = request.path.unwrap();
            println!("start_data_channel {:?}", path);
            let d1 = Arc::clone(&self.data_channel);

            // Cancellation
            let mut tokens = self.cancel_tokens.try_lock().unwrap();
            for (key, val) in tokens.iter() {
                println!("Cancelling stream: {key}");
                val.cancel();
            }
            let token = CancellationToken::new();
            let tk = token.clone();
            tokens.insert(path.clone(), token.clone());
            let decoding_active = self.decoding_active.clone();
            let receiver = self.flow_control_receiver.clone();
            tokio::spawn(async move {
                decoding_active.store(ACTIVE, std::sync::atomic::Ordering::Relaxed);

                wake_all(decoding_active.as_ref());
                // Here decode chunk
                let _d2 = d1
                    .clone()
                    .try_lock()
                    .unwrap()
                    .clone()
                    .expect("Should not be empty");
                // Wait for either cancellation or a very long time

                match stream_file(
                    _d2,
                    path.as_str(),
                    tk,
                    request.seek,
                    request.file_info,
                    receiver,
                    decoding_active,
                    app_handle,
                )
                .await
                {
                    Ok(r) => {
                        println!("Finished streaming {:?}", path.as_str());
                    }
                    Err(err) => {
                        println!("Error streaming {}", err);
                    }
                }
            });
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

    // SYmphonia stuff
    async fn stream_file(
        data_channel: Arc<RTCDataChannel>,
        path_str: &str,
        cancel_token: CancellationToken,
        seek: Option<f64>,
        file_info: FileInfo,
        flow_control_receiver: Arc<Mutex<Receiver<FlowControlEvent>>>,
        decoding_active: Arc<AtomicU32>,
        app_handle: AppHandle,
    ) -> symphonia::core::errors::Result<i32> {
        println!("stream_file");
        let path = Path::new(path_str);

        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();

        let source = Box::new(File::open(path)?);
        println!("source {:?}", source);

        // Provide the file extension as a hint.
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(source, Default::default());

        // Use the default options for format readers other than for gapless playback.
        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };

        // Use the default options for metadata readers.
        let metadata_opts: MetadataOptions = MetadataOptions {
            limit_metadata_bytes: symphonia::core::meta::Limit::Maximum(32),
            limit_visual_bytes: symphonia::core::meta::Limit::Maximum(32),
        };

        // Get the value of the track option, if provided.
        let track: Option<Track> = None;
        println!("probing {:?}", hint);
        println!("opts {:?}", format_opts);
        println!("meta {:?}", metadata_opts);
        let mut probe: Probe = Default::default();
        register_enabled_formats(&mut probe);
        let probe_result = probe.format(&hint, mss, &format_opts, &metadata_opts);
        println!("probe format {:?}", probe_result.is_ok());

        // Verify-only mode decodes and verifies the audio, but does not play it.
        match probe_result {
            Ok(mut probed) => {
                // Wait for either cancellation or a very long time
                select! {
                    _ = cancel_token.cancelled() => {
                        // The token was cancelled
                        println!("Cancelled");
                    }

                _ = decode_only(
                    data_channel,
                    probed.format,
                    &DecoderOptions {
                        verify: true,
                        ..Default::default()
                    },
                    cancel_token.clone(),
                    seek,
                    file_info,
                    flow_control_receiver,
                    decoding_active,
                    app_handle
                ) => {
                    // The token was cancelled
                    println!("Completed");
                }
                }
                Ok(0)
            }
            Err(err) => {
                println!("{}", err);
                // The input was not supported by any format reader.
                Err(err)
            }
        }
    }

    async fn decode_only(
        data_channel: Arc<RTCDataChannel>,
        mut reader: Box<dyn FormatReader>,
        decode_opts: &DecoderOptions,
        cancel_token: CancellationToken,
        seek: Option<f64>,
        file_info: FileInfo,
        flow_control_receiver: Arc<Mutex<Receiver<FlowControlEvent>>>,
        decoding_active: Arc<AtomicU32>,
        app_handle: AppHandle,
    ) -> symphonia::core::errors::Result<()> {
        println!("decode_only");

        // Get the default track.
        // TODO: Allow track selection.
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
        let mut decoder =
            symphonia::default::get_codecs().make(&track.codec_params, decode_opts)?;

        /*
        We're sending the PCM as bytes in chunks of 10kb,
        which is just below the maximum limit on Chrome (16kb per message)
        */
        let mut byte_buf: AllocRingBuffer<u8> = AllocRingBuffer::new(1024 * 10); // 10KB
        let mut to_send = Bytes::new();
        let mut last_sent_time = Instant::now();
        let mut packet_counter = 0;
        let target_bitrate = file_info.bit_depth.unwrap() as u32
            * file_info.sample_rate.unwrap()
            * file_info.channels.unwrap() as u32;
        println!("Target bits/second: {}", target_bitrate);
        let max_packet_size = &track.codec_params.max_frames_per_packet;
        let max_block_size = &track.codec_params.frames_per_block;

        let mut fastest_send_interval = Duration::new(0, 0);
        let mut slowest_send_interval = Duration::new(0, 0);

        println!("Max packet size: {:?} frames", max_packet_size);
        println!("Max block size: {:?} frames", max_block_size);
        // data_channel.on_buffered_amount_low(|f| {

        // });
        let mut is_first_packet = true;
        let mut send_interval = Duration::from_millis(1); // Initial send interval

        // Decode all packets, ignoring all decode errors.
        let result = loop {
            atomic_wait::wait(&decoding_active, PAUSED); // waits while the value is PAUSED (0)

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
                    let frames = _decoded.frames();
                    // Create a raw sample buffer that matches the parameters of the decoded audio buffer.
                    let mut sample_buf =
                        RawSampleBuffer::<f32>::new(_decoded.capacity() as u64, *_decoded.spec());

                    // Copy the contents of the decoded audio buffer into the sample buffer whilst performing
                    // any required conversions.
                    sample_buf.copy_interleaved_ref(_decoded);

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
                            let bits = frames * file_info.bit_depth.unwrap() as u32;

                            // Never send frames slower than 0.75x playback!
                            slowest_send_interval =
                                Duration::from_secs_f64(bits as f64 / target_bitrate as f64)
                                    .mul_f64(1.1f64);

                            // Never send faster than 3x playback
                            fastest_send_interval =
                                Duration::from_secs_f64(bits as f64 / target_bitrate as f64)
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

                        let decode_rate_micros = bits as f64 / elapsed.as_micros() as f64;
                        let decode_rate_second = decode_rate_micros * 1000000f64;
                        println!(
                            "decode rate (seconds): {:.2} kb/s",
                            decode_rate_second / 1000f64
                        );

                        // Adjust the send interval based on signals from the client
                        match flow_control_receiver.try_lock().unwrap().try_recv() {
                            Ok(ev) => {
                                // Calculate the ratio between the playback bitrate
                                // and the receive rate on the client
                                let ratio = (ev.client_bitrate.unwrap()) / target_bitrate as f64;
                                println!("Received client bitrate: {:?}", ev);
                                println!("Client is receiving at {:.2}x the playback speed", ratio);

                                // Adjust the send interval
                                // Saturate at 0 instead of overflowing to negative - this means the client is receiving slower than we can send
                                // and we can't send faster than the decoder is decoding, so the interval is zero.

                                println!(
                                    "Send interval is {:?} microseconds",
                                    send_interval.as_micros()
                                );
                                send_interval = send_interval.mul_f64(ratio);

                                println!(
                                    "New Send interval is {:?} microseconds",
                                    send_interval.as_micros()
                                );
                                send_interval = send_interval
                                    .min(fastest_send_interval)
                                    .max(slowest_send_interval);

                                println!(
                                    "Limited interval is {:?} microseconds",
                                    send_interval.as_micros()
                                );
                            }
                            Err(err) => {
                                println!("Error receiving client bitrate: {:?}", err);
                            }
                        }
                    }

                    last_sent_time = Instant::now();

                    // Send packet to JS here
                    if (data_channel.ready_state() == RTCDataChannelState::Open
                        && !cancel_token.is_cancelled())
                    {
                        thread::sleep(send_interval);
                        match data_channel
                            .send(&Bytes::from(sample_buf.as_bytes().to_vec()))
                            .await
                        {
                            Ok(packet) => {
                                // Sent! Can clear to_send now for the next chunk
                                to_send.clear();
                            }
                            Err(err) => {
                                println!("Error sending {:?}", err);
                            }
                        };
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
        ignore_end_of_stream_error(result)

        // Finalize the decoder and return the verification result if it's been enabled.
        // do_verification(decoder.finalize())
    }

    fn first_supported_track(tracks: &[Track]) -> Option<&Track> {
        tracks
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
    }

    fn ignore_end_of_stream_error(
        result: symphonia::core::errors::Result<()>,
    ) -> symphonia::core::errors::Result<()> {
        match result {
            Err(symphonia::core::errors::Error::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof
                    && err.to_string() == "end of stream" =>
            {
                println!("End of stream!!");
                // Do not treat "end of stream" as a fatal error. It's the currently only way a
                // format reader can indicate the media is complete.
                Ok(())
            }
            _ => result,
        }
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
