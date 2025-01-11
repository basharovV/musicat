use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Instant;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use atomic_wait::wake_all;
use cpal::traits::{DeviceTrait, HostTrait};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use symphonia::core::audio::{AudioBufferRef, Layout, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error::ResetRequired;
use symphonia::core::formats::{FormatOptions, SeekTo, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use symphonia::default::get_probe;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use tokio::time::Duration;

use tokio_util::sync::CancellationToken;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::constants::*;
#[cfg(target_os = "macos")]
use crate::mediakeys;
use crate::metadata::Song;
use crate::output::{self, get_device_by_name, AudioOutput, DeviceWithConfig, PlaybackState};
use crate::store::load_settings;
use crate::{dsp, GetWaveformRequest, GetWaveformResponse, SampleOffsetEvent, StreamFileRequest};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoopRegionRequest {
    enabled: Option<bool>,
    start_pos: Option<f64>,
    end_pos: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeAudioDeviceRequest {
    audio_device: Option<String>,
}

#[derive(Debug)]
pub enum PlayerControlEvent {
    StreamFile(StreamFileRequest), // path, seekpos
    LoopRegion(LoopRegionRequest),
    ChangeAudioDevice(ChangeAudioDeviceRequest),
    ChangePlaybackSpeed(PlaybackSpeedControlEvent),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VolumeControlEvent {
    pub volume: Option<f64>, // 0 to 1
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaybackSpeedControlEvent {
    playback_speed: Option<f64>, // 0.3 to 3
}

#[tauri::command]
pub fn loop_region(
    event: LoopRegionRequest,
    state: State<AudioPlayer>,
    _app_handle: tauri::AppHandle,
) {
    info!("Loop region{:?}", event);
    let _ = state
        .player_control_sender
        .send(PlayerControlEvent::LoopRegion(event));
}

#[tauri::command]
pub fn change_audio_device(
    event: ChangeAudioDeviceRequest,
    state: State<AudioPlayer>,
    _app_handle: tauri::AppHandle,
) {
    info!("Change audio device{:?}", event);
    let _ = state
        .player_control_sender
        .send(PlayerControlEvent::ChangeAudioDevice(event));

    // Handle the case where audio device is changed while paused
    state.resume();
}

#[tauri::command]
pub fn volume_control(event: VolumeControlEvent, state: State<AudioPlayer>) {
    info!("Received volume_control event");
    match state.volume_control_sender.send(event) {
        Ok(_) => {
            // info!("Sent control flow info");
        }
        Err(_err) => {
            info!("Error sending volume control info (channel inactive)");
        }
    }
}

#[tauri::command]
pub fn playback_speed_control(event: PlaybackSpeedControlEvent, state: State<AudioPlayer>) {
    info!("Received playback_speed_control event");

    match state
        .player_control_sender
        .send(PlayerControlEvent::ChangePlaybackSpeed(event))
    {
        Ok(_) => {
            // info!("Sent control flow info");
        }
        Err(_err) => {
            info!("Error sending playback_speed_control info (channel inactive)");
        }
    }
}

pub const PAUSED: u32 = 0;
pub const ACTIVE: u32 = 1;

#[derive(Clone)]
pub struct AudioPlayer<'a> {
    pub peer_connection: Arc<Mutex<Option<Arc<RTCPeerConnection>>>>,
    pub data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    pub cancel_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    pub player_control_receiver: Arc<Mutex<Receiver<PlayerControlEvent>>>,
    pub player_control_sender: Sender<PlayerControlEvent>,
    pub next_track_receiver: Arc<Mutex<Receiver<StreamFileRequest>>>,
    pub next_track_sender: Sender<StreamFileRequest>,
    pub decoding_active: Arc<AtomicU32>,
    pub volume_control_receiver: Arc<Mutex<Receiver<VolumeControlEvent>>>,
    pub volume_control_sender: Sender<VolumeControlEvent>,
    phantom: PhantomData<&'a RTCPeerConnection>,
    phantom2: PhantomData<&'a RTCDataChannel>,
}

impl<'a> AudioPlayer<'a> {
    pub fn create() -> Result<AudioPlayer<'a>, Box<dyn std::error::Error + Send + Sync>> {
        let (sender_vol, receiver_vol) = std::sync::mpsc::channel();

        // set up message passing
        let (sender_tx, receiver_rx): (Sender<PlayerControlEvent>, Receiver<PlayerControlEvent>) =
            std::sync::mpsc::channel();

        let (sender_next, receiver_next): (Sender<StreamFileRequest>, Receiver<StreamFileRequest>) =
            std::sync::mpsc::channel();

        Ok(AudioPlayer {
            peer_connection: Arc::new(Mutex::new(None)),
            data_channel: Arc::new(Mutex::new(None)),
            cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
            player_control_receiver: Arc::new(Mutex::new(receiver_rx)),
            player_control_sender: sender_tx,
            next_track_receiver: Arc::new(Mutex::new(receiver_next)),
            next_track_sender: sender_next,
            decoding_active: Arc::new(AtomicU32::new(ACTIVE)),
            volume_control_receiver: Arc::new(Mutex::new(receiver_vol)),
            volume_control_sender: sender_vol,
            phantom: PhantomData,
            phantom2: PhantomData,
        })
    }

    pub fn init(&self, app_handle: AppHandle) -> () {
        let receiver = self.player_control_receiver.clone();
        let next_track_receiver = self.next_track_receiver.clone();
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

    /**
     * Initialize WebRTC to stream the time domain FFT data over Data Channels.
     * The audio callback sends the data during playback.
     * There's be a small amount of latency, but it's acceptable.
     */
    pub async fn init_webrtc(
        &self,
        app_handle: AppHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create a MediaEngine object to configure the supported codec
        let mut m = MediaEngine::default();

        // Register default codecs
        m.register_default_codecs()?;
        info!("WEBRTC streamer");

        // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
        // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
        // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
        // for each PeerConnection.
        let mut registry = Registry::new();

        // Use the default set of Interceptors
        registry = register_default_interceptors(registry, &mut m)?;

        let mut s = SettingEngine::default();
        s.set_include_loopback_candidate(true);

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .with_setting_engine(s)
            .build();

        // Prepare the configuration
        let config = RTCConfiguration {
            ice_servers: Vec::new(),
            ..Default::default()
        };

        // Create a new RTCPeerConnection
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        // Create a datachannel with label 'data'
        let data_channel = peer_connection
            .create_data_channel(
                "data",
                Some(RTCDataChannelInit {
                    max_retransmits: Some(0),
                    ordered: Some(false),
                    ..Default::default()
                }),
            )
            .await?;

        data_channel.on_open(Box::new(move || {
            info!("Data channel opened");
            Box::pin(async {})
        }));

        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                info!("Peer Connection State has changed: {s}");

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    info!("Peer Connection has gone to failed exiting");
                }

                Box::pin(async {})
            },
        ));

        // Listen for ICE candidates
        peer_connection.on_ice_candidate(Box::new(move |c| {
            info!("on_ice_candidate {:?}", c);
            if let Some(cand) = c {
                // let candidate = serde_json::to_string(&cand.to_json().unwrap());
                if cand.address.contains("127.0.0.1") {
                    let _ = app_handle.emit("webrtc-icecandidate-client", &cand.to_json().unwrap());
                }
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
        candidate: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        info!("handle_ice_candidate {:?}", candidate);
        let parsed: RTCIceCandidateInit = serde_json::from_str(candidate).unwrap();

        if let Ok(pc_mutex) = self.peer_connection.try_lock() {
            if let Some(pc) = pc_mutex.clone().or(None) {
                let _ = pc.add_ice_candidate(parsed).await;
            }
        }
        Ok(())
    }

    pub async fn handle_signal(self, answer: &str) -> Option<RTCSessionDescription> {
        // Apply the answer as the remote description
        let parsed_description: RTCSessionDescription = serde_json::from_str(answer).unwrap();
        info!("handle_signal {:?}", parsed_description);

        if let Ok(pc_mutex) = self.peer_connection.try_lock() {
            if let Some(pc) = pc_mutex.clone().or(None) {
                let _ = pc.set_remote_description(parsed_description).await;
                let ans = Some(pc.create_answer(None).await.unwrap());
                let _ = pc.set_local_description(ans.clone().unwrap()).await;
                return ans;
            }
        }
        None
    }

    pub async fn reset(&self) {
        info!("Resetting streamer");

        if let Ok(dc_mutex) = self.data_channel.try_lock() {
            if let Some(dc) = dc_mutex.clone().or(None) {
                info!("Closing data channel...");
                match dc.close().await {
                    Ok(()) => {
                        info!("Closed data channel");
                    }
                    Err(err) => {
                        info!("Error closing data channel{}", err);
                    }
                }
            }
        }

        if let Ok(pc_mutex) = self.peer_connection.try_lock() {
            if let Some(pc) = pc_mutex.clone().or(None) {
                info!("Closing peer connection...");
                match pc.close().await {
                    Ok(()) => {
                        info!("Closed peer connection");
                    }
                    Err(err) => {
                        info!("Error closing peer connection {}", err);
                    }
                }
            }
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

/**
 * The main audio decoding loop. The outer loop is used to switch between tracks.
 * The inner loop is used to decode audio and send packets to the audio device.
 * To pause, we pause the inner decoding loop using [`atomic_wait::wait`] and [`AtomicU32] flags.
 * On every iteration, we check for events such as seeking, pausing, new track, etc.
 */
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
    let mut path_str_clone: Option<String>;
    let mut seek = None;

    /* Previous seek and duration used to time song
     * info change during gapless transition */
    let mut prev_seek = 0.0;
    let mut prev_song: Option<Song> = None;

    /* Current timestamp in seconds received every second from audio callback.
     * Used to restore seek position after audio device change.*/
    let mut timestamp: f64 = 0.0;

    // for loop region
    let mut end_pos = None;

    let mut volume = None;
    let mut playback_speed = 1.0f64;
    let mut audio_device_name = None;
    let mut previous_audio_device_name: String = String::new();
    let mut previous_sample_rate = 44100;
    let mut previous_channels = 2;

    /* Channels for message passing */
    let (playback_state_sender, playback_state_receiver) = std::sync::mpsc::channel();
    let (reset_control_sender, reset_control_receiver) = std::sync::mpsc::channel();
    let (device_change_sender, device_change_receiver) = std::sync::mpsc::channel();
    let (sender_sample_offset, receiver_sample_offset) = std::sync::mpsc::channel();
    let sample_offset_receiver = Arc::new(Mutex::new(receiver_sample_offset));
    let (timestamp_sender, timestamp_receiver) = std::sync::mpsc::channel();
    let timestamp_send = Arc::new(Mutex::new(timestamp_sender));
    let playback_state = Arc::new(Mutex::new(playback_state_receiver));
    let reset_control = Arc::new(Mutex::new(reset_control_receiver));
    let device_change = Arc::new(Mutex::new(device_change_receiver));

    /* Devices are cached to avoid accessing current audio device during playback
    (which can cause an audible glitch when switching tracks automatically ie. gapless transition).
    Therefore, audio devices are only accessed directly when switching tracks manually */
    let mut cached_devices: Option<Vec<DeviceWithConfig>> = None;

    /* Current audio output for writing samples to. Can be reset when switching tracks */
    let mut audio_output: Option<Result<Arc<Mutex<dyn AudioOutput>>, output::AudioOutputError>> =
        None;

    /* This is set during a gapless transition, when we're already decoding the next track
     * while playing the ending of the current one. */
    let mut is_transition = false;
    let mut is_reset = true; // Whether the playback has been 'reset' (i.e double click on new track, next btn)

    let mut resampler_delay = 0.0;

    let mut current_max_frames = 1152;

    // Loop here!
    loop {
        // info!("path_str is {:?}", path_str);
        path_str_clone = path_str.clone(); // Used for looping
        if let None = path_str {
            is_transition = false;
            let event = player_control_receiver.try_lock().unwrap().recv();

            info!("audio: waiting for file! {:?}", event);
            if let Ok(result) = event {
                match result {
                    PlayerControlEvent::StreamFile(request) => {
                        info!("audio: got file request! {:?}", request);
                        path_str.replace(request.path.unwrap());
                        prev_seek = seek.unwrap_or(0.0);
                        seek.replace(request.seek.unwrap());
                        volume.replace(request.volume.unwrap());
                        audio_device_name = request.output_device;
                    }
                    PlayerControlEvent::LoopRegion(request) => {
                        info!("audio: loop region! {:?}", request);
                        path_str.replace(path_str_clone.unwrap());
                        prev_seek = seek.unwrap_or(0.0);
                        seek.replace(request.start_pos.unwrap());
                        end_pos.replace(request.end_pos.unwrap());
                    }
                    PlayerControlEvent::ChangeAudioDevice(request) => {
                        info!("audio: change audio device! {:?}", request);
                        audio_device_name = request.audio_device;
                        if path_str_clone.is_some() && path_str.is_some() {
                            path_str.replace(path_str_clone.clone().unwrap());
                        }
                        is_reset = true;
                        is_transition = false;
                    }
                    PlayerControlEvent::ChangePlaybackSpeed(request) => {
                        info!("audio: change playback speed! {:?}", request);
                        if let Some(speed) = request.playback_speed {
                            playback_speed = speed;
                        }
                    }
                }
            }
        } else if let Some(ref p) = path_str.clone() {
            let path = Path::new(p.as_str());

            // Create a hint to help the format registry guess what format reader is appropriate.
            let mut hint = Hint::new();
            let fl = File::open(path);

            if fl.is_err() {
                path_str = None;
                error!("Error opening file: {}", fl.err().unwrap());
                continue;
            }

            let source = Box::new(fl.unwrap());
            info!("source {:?}", source);

            // Provide the file extension as a hint.
            info!("extension: {:?}", path.extension());
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
                limit_metadata_bytes: symphonia::core::meta::Limit::Maximum(50),
                limit_visual_bytes: symphonia::core::meta::Limit::Maximum(0),
            };

            // Get the value of the track option, if provided.
            info!("probing {:?}", hint);
            info!("opts {:?}", format_opts);
            info!("meta {:?}", metadata_opts);

            let probe_result = get_probe().format(&hint, mss, &format_opts, &metadata_opts);
            info!("probe format {:?}", probe_result.is_ok());

            if probe_result.is_err() {
                error!("Error probing file: {}", probe_result.err().unwrap());
                path_str = None;
                let _ = reset_control_sender.send(true);
                let _ = playback_state_sender.send(PlaybackState {
                    is_playing: false,
                    playback_speed,
                });
                continue;
            }

            info!("Resetting path_str");
            path_str = None;

            let mut reader = probe_result.unwrap().format;

            let track = reader.default_track().unwrap().clone();

            if let Some(frames) = track.codec_params.n_frames {
                let _ = app_handle.emit("file-samples", frames);
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

            info!("codec params: {:?}", &track.codec_params);

            // Create a decoder for the track.
            let mut decoder = symphonia::default::get_codecs()
                .make(&track.codec_params, &DecoderOptions { verify: false })
                .unwrap();

            let mut channels;
            let mut first_packet = None;

            if path.extension().and_then(|ext| ext.to_str()) == Some("m4a") {
                if let Ok(packet) = reader.next_packet() {
                    if let Ok(buffer) = decoder.decode(&packet) {
                        channels = buffer.spec().channels;
                    } else {
                        channels = decoder.codec_params().channels.unwrap();
                    }

                    first_packet = Some(packet);
                } else {
                    channels = decoder.codec_params().channels.unwrap();
                }
            } else {
                channels = decoder.codec_params().channels.unwrap();
            }

            let spec = SignalSpec {
                rate: decoder.codec_params().sample_rate.unwrap(),
                channels,
            };

            let mut should_reset_audio = false;
            let mut max_frames_changed = false;

            let max_frames = decoder.codec_params().max_frames_per_packet;
            info!(
                "max frames: {:?} current: {:?}",
                max_frames, current_max_frames
            );
            if let Some(dur) = max_frames {
                if dur != current_max_frames {
                    max_frames_changed = true;
                }
                current_max_frames = dur;
            }

            // Check if audio device changed
            let mut follow_system_output = false;
            if let Ok(settings) = load_settings(app_handle) {
                audio_device_name = settings.output_device;
                follow_system_output = settings.follow_system_output;
            }

            // Only reenumerate audio devices when manually switching tracks,
            // otherwise use cached to avoid glitches
            if !is_transition || cached_devices.as_ref().is_none() {
                info!("Enumerating audio devices");
                let dvces = output::enumerate_devices();
                cached_devices.replace(dvces);
            };

            let output_device = if cached_devices.as_ref().is_some()
                && !follow_system_output
                && audio_device_name.is_some()
            {
                info!("Using cached audio device: {:?}", audio_device_name);
                cached_devices
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|d| d.device.clone())
                    .find(|device| device.name().unwrap() == audio_device_name.clone().unwrap())
            } else {
                let default = output::default_device();
                info!(
                    "Using default audio device: {:?}",
                    default.clone().unwrap().name()
                );
                default
            };

            let device_name = output_device.clone().unwrap().name().unwrap();
            // If we have a default audio device (we always should, but just in case)
            // we check if the track spec differs from the output device
            // if it does - resample the decoded audio using Symphonia.

            // Check if track sample rate differs from current OS config
            if let Some(mut device) = output_device {
                info!("cpal: Default device {:?}", device.name());
                // Only resample when audio device doesn't support file sample rate
                // so we can't switch the device rate to match.
                // info!(
                //     "cpal: device default config {:?}",
                //     device.default_output_config()
                // );
                let supported_output_configs = if !is_transition || cached_devices.is_none() {
                    device.supported_output_configs().ok().map(|c| c.collect()) // Read from device (may cause small glitch if playing)
                } else {
                    Some(
                        cached_devices
                            .as_ref()
                            .unwrap()
                            .iter()
                            .find(|d| d.device.name().unwrap() == device_name)
                            .unwrap()
                            .config
                            .clone(),
                    )
                };

                let mut supports_sample_rate = false;
                if let Some(output_configs) = supported_output_configs {
                    info!(
                        "cpal: device supported configs {:?}",
                        output_configs
                            .iter()
                            .by_ref()
                            .map(|c| format!(
                                "min: {}, max: {}",
                                c.min_sample_rate().0,
                                c.max_sample_rate().0
                            ))
                            .collect::<Vec<String>>()
                    );
                    supports_sample_rate = output_configs
                        .iter()
                        .find(|c| {
                            return c
                                .try_with_sample_rate(cpal::SampleRate(spec.rate))
                                .is_some();
                        })
                        .is_some();
                } else if supported_output_configs.is_none() {
                    error!("failed to get audio output device config");
                    device = get_device_by_name(None).unwrap();
                }
                // If sample rate or channels changed - reinit the audio device with the new spec
                // (if this sample rate isn't supported, it will be resampled)

                should_reset_audio = previous_audio_device_name != device.name().unwrap()
                    || supports_sample_rate && spec.rate != previous_sample_rate
                    || spec.channels.count() != previous_channels
                    || max_frames_changed;
            }

            previous_sample_rate = spec.rate;
            previous_channels = spec.channels.count();
            previous_audio_device_name = device_name.clone();

            let song = crate::metadata::extract_metadata(
                &Path::new(&p.clone().as_str()),
                false,
                true,
                &app_handle,
            );

            if audio_output.is_none() || should_reset_audio {
                info!("player: Resetting audio device");
                // Try to open the audio output.

                if should_reset_audio {
                    info!("Stopping audio output");
                    if let Some(output) = audio_output {
                        if let Ok(out) = output {
                            if let Ok(mut guard) = out.try_lock() {
                                /* If we determine that audio device should change for the next track, don't stop the stream immediately.
                                Wait until track has finished playing. */
                                if is_transition {
                                    while guard.has_remaining_samples() {
                                        // Wait
                                    }
                                    is_transition = false; // Revert transition mode so that track/seek info is changed straight away

                                    // Send song change event
                                    if let Some(s) = &song {
                                        let _ = app_handle.emit("song_change", Some(s));
                                        let _ = reset_control_sender.send(true);
                                        let _ = sender_sample_offset.send(SampleOffsetEvent {
                                            sample_offset: Some(
                                                seek_ts
                                                    * track.codec_params.channels.unwrap().count()
                                                        as u64,
                                            ),
                                        });
                                    } else {
                                        info!("ERROR getting song");
                                    }
                                }

                                guard.flush();
                                guard.pause();
                                guard.stop_stream();
                            }
                        }
                    }
                }
                audio_output = Some(output::try_open(
                    &previous_audio_device_name,
                    spec,
                    current_max_frames,
                    volume_control_receiver.clone(),
                    sample_offset_receiver.clone(),
                    playback_state.clone(),
                    reset_control.clone(),
                    device_change.clone(),
                    timestamp_send.clone(),
                    data_channel.clone(),
                    volume.clone(),
                    app_handle.clone(),
                ));
            } else {
                info!("player: Re-using existing audio output");
            }

            let mut last_sent_time;

            if !is_transition {
                let clone_device_name = device_name.clone();
                let clone_device_name2 = device_name.clone();
                let _ = reset_control_sender.send(true);
                let _ = device_change_sender.send(clone_device_name);
                let _ = app_handle.emit("audio_device_changed", clone_device_name2);
                let _ = sender_sample_offset.send(SampleOffsetEvent {
                    sample_offset: Some(seek_ts * previous_channels as u64),
                });
            }

            let end_pos_frame_idx = if end_pos.is_some() {
                (end_pos.unwrap() * previous_channels as f64) as u64
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
                        guard.update_resampler(spec, current_max_frames, playback_speed, is_reset);

                        // Until all samples have been flushed - don't start decoding
                        // Keep checking until all samples have been played (buffer is empty)
                        if is_reset {
                            // TODO: Set volume to zero while flushing
                            while guard.has_remaining_samples() {
                                guard.flush();
                                info!("Buffer is not empty yet, waiting to continue...");
                            }
                            info!("Buffer is now empty. Continuing decoding...");
                            is_reset = false;
                        }

                        // Set media keys / now playing
                        if let Some(s) = &song {
                            #[cfg(target_os = "macos")]
                            mediakeys::set_now_playing_info(s);
                        }

                        // Decode all packets, ignoring all decode errors.
                        let result = loop {
                            if let Ok(ts) = timestamp_receiver.try_recv() {
                                timestamp = ts;
                            }
                            let event = receiver.try_recv();
                            // debug!("audio: waiting for event {:?}", event);
                            if let Ok(result) = event {
                                match result {
                                    PlayerControlEvent::StreamFile(request) => {
                                        info!(
                                            "audio: source changed during decoding! {:?}",
                                            request
                                        );
                                        path_str.replace(request.path.unwrap());
                                        prev_seek = seek.unwrap();
                                        prev_song = song.clone();
                                        seek.replace(request.seek.unwrap());
                                        end_pos = None;
                                        volume.replace(request.volume.unwrap());
                                        guard.flush();
                                        is_reset = true;
                                        is_transition = false;
                                    }
                                    PlayerControlEvent::LoopRegion(request) => {
                                        info!("audio: loop region! {:?}", request);
                                        if request.enabled.unwrap() {
                                            seek.replace(request.start_pos.unwrap());
                                            end_pos.replace(request.end_pos.unwrap());
                                        } else {
                                            end_pos = None;
                                        }
                                        path_str.replace(path_str_clone.clone().unwrap());
                                        guard.flush();
                                        is_reset = true;
                                        is_transition = false;
                                    }
                                    PlayerControlEvent::ChangeAudioDevice(request) => {
                                        info!("audio: change audio device! {:?}", request);
                                        audio_device_name = request.audio_device;
                                        path_str.replace(path_str_clone.clone().unwrap());
                                        guard.flush();
                                        guard.pause();
                                        seek.replace(timestamp); // Restore current seek position
                                        is_reset = true;
                                        is_transition = false;
                                    }
                                    PlayerControlEvent::ChangePlaybackSpeed(request) => {
                                        info!("audio: change playback speed! {:?}", request);
                                        if let Some(speed) = request.playback_speed {
                                            playback_speed = speed;
                                        }
                                        // while guard.has_remaining_samples() {
                                        //     guard.flush();
                                        //     info!(
                                        //         "Buffer is not empty yet, waiting to continue..."
                                        //     );
                                        // }
                                        guard.update_resampler(
                                            spec,
                                            current_max_frames,
                                            playback_speed,
                                            false,
                                        );
                                    }
                                }
                            }

                            let mut is_paused = false;
                            if decoding_active.load(std::sync::atomic::Ordering::Relaxed) == PAUSED
                            {
                                is_paused = true;
                                info!("Sending paused state to output");
                                guard.pause();
                                let _ = playback_state_sender.send(PlaybackState {
                                    is_playing: true,
                                    playback_speed,
                                });
                                let _ = app_handle.emit("paused", {});
                                #[cfg(target_os = "macos")]
                                mediakeys::set_paused();
                            }

                            // waits while the value is PAUSED (0)
                            atomic_wait::wait(&decoding_active, PAUSED);

                            // By default we want to resume the output after un-pausing,
                            // unless the audio device has changed, in which case this is just a
                            // temporary trip round the loop until the new device is set, and we pause again
                            // to restore the previous state

                            let mut should_resume = true;

                            if is_paused {
                                let ctrl_event = receiver.try_recv();
                                if let Ok(result) = ctrl_event {
                                    match result {
                                        PlayerControlEvent::StreamFile(request) => {
                                            info!(
                                                "audio: source changed during decoding! {:?}",
                                                request
                                            );
                                            path_str.replace(request.path.unwrap());
                                            prev_seek = seek.unwrap_or(0.0);
                                            prev_song = song.clone();
                                            seek.replace(request.seek.unwrap());
                                            end_pos = None;
                                            volume.replace(request.volume.unwrap());
                                            guard.flush();
                                            is_reset = true;
                                            is_transition = false;
                                        }
                                        PlayerControlEvent::LoopRegion(request) => {
                                            info!("audio: loop region! {:?}", request);
                                            if request.enabled.unwrap() {
                                                seek.replace(request.start_pos.unwrap());
                                                end_pos.replace(request.end_pos.unwrap());
                                            } else {
                                                end_pos = None;
                                            }
                                            path_str.replace(path_str_clone.clone().unwrap());
                                            guard.flush();
                                            is_reset = true;
                                            is_transition = false;
                                        }
                                        PlayerControlEvent::ChangeAudioDevice(request) => {
                                            info!("audio: change audio device! {:?}", request);
                                            audio_device_name = request.audio_device;
                                            path_str.replace(path_str_clone.clone().unwrap());
                                            guard.flush();
                                            guard.pause();
                                            seek.replace(timestamp); // Restore current seek position
                                            is_reset = true;
                                            is_transition = false;
                                            if is_paused {
                                                should_resume = false;
                                                // Restore pause state after device change
                                                let _ = &decoding_active.store(
                                                    PAUSED,
                                                    std::sync::atomic::Ordering::Relaxed,
                                                );
                                                wake_all(decoding_active.as_ref());
                                            }
                                        }
                                        PlayerControlEvent::ChangePlaybackSpeed(request) => {
                                            info!("audio: change playback speed! {:?}", request);
                                            if let Some(speed) = request.playback_speed {
                                                playback_speed = speed;
                                            }
                                            // while guard.has_remaining_samples() {
                                            //     guard.flush();
                                            //     info!("Buffer is not empty yet, waiting to continue...");
                                            // }
                                            guard.update_resampler(
                                                spec,
                                                current_max_frames,
                                                playback_speed,
                                                false,
                                            );
                                        }
                                    }
                                }

                                if should_resume {
                                    guard.resume();
                                }
                            }

                            if is_reset {
                                break Ok(());
                            }

                            let _ = playback_state_sender.send(PlaybackState {
                                is_playing: true,
                                playback_speed,
                            });
                            let _ = app_handle.emit("playing", {});
                            #[cfg(target_os = "macos")]
                            mediakeys::set_playing();

                            let packet = if let Some(packet) = first_packet.take() {
                                packet
                            } else {
                                match reader.next_packet() {
                                    Ok(packet) => packet,
                                    Err(err) => break Err(err),
                                }
                            };

                            // If the packet does not belong to the selected track, skip over it.
                            if packet.track_id() != track_id {
                                continue;
                            }

                            // Loop region mode: If this packet is past the loop region,
                            // seek the reader back to the start point
                            if end_pos.is_some() && packet.ts > end_pos_frame_idx {
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
                                Ok(mut _decoded) => {
                                    last_sent_time = Instant::now();

                                    /*
                                    The transition is [`BUFFER_SIZE`] seconds long
                                    So decoding of the new track starts [`BUFFER_SIZE`] seconds before playback,
                                    and we can delay the song change in the UI by this time.
                                     */
                                    if is_transition && !started_transition {
                                        started_transition = true;
                                        transition_time = last_sent_time;
                                    } else if is_transition && started_transition {
                                        // Note: At this point we're already decoding the new track,
                                        // but we use the previous track's seek and duration info for this logic
                                        // Check if seek position is within transition zone
                                        // If so - make transition shorter by delta
                                        let duration =
                                            prev_song.clone().unwrap().file_info.duration.unwrap();
                                        let seeked_to = prev_seek;
                                        let mut delta = 0.0;

                                        if seeked_to > duration - BUFFER_SIZE
                                            && seeked_to < duration
                                        {
                                            delta = duration - seeked_to;
                                        }

                                        if transition_time.elapsed().as_secs_f64() * playback_speed
                                            >= (BUFFER_SIZE - delta + (resampler_delay))
                                        {
                                            info!(
                                                "transition complete after {:.2}s, with {:.2}s delay",
                                                transition_time.elapsed().as_secs_f64(),
                                                resampler_delay
                                            );
                                            if end_pos.is_some() {
                                                let _ =
                                                    sender_sample_offset.send(SampleOffsetEvent {
                                                        sample_offset: Some(
                                                            seek_ts * previous_channels as u64,
                                                        ),
                                                    });
                                            }

                                            if let Some(s) = &song {
                                                let _ = app_handle.emit("song_change", Some(s));

                                                let _ = reset_control_sender.send(true);
                                                let _ =
                                                    sender_sample_offset.send(SampleOffsetEvent {
                                                        sample_offset: Some(
                                                            seek_ts * previous_channels as u64,
                                                        ),
                                                    });
                                            } else {
                                                info!("ERROR getting song");
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
                                    if !is_reset {
                                        // Write the decoded audio samples to the audio output if the presentation timestamp
                                        // for the packet is >= the seeked position (0 if not seeking).
                                        if packet.ts() >= seek_ts {
                                            let mut ramp_up_smpls = 0;
                                            let mut ramp_down_smpls = 0;
                                            // Avoid clicks by ramping down and up quickly
                                            if !is_transition {
                                                if let Some(frames) = track.codec_params.n_frames {
                                                    if packet.ts >= frames - packet.dur {
                                                        ramp_down_smpls = packet.dur;
                                                    } else if packet.ts < packet.dur {
                                                        ramp_up_smpls = packet.dur;
                                                    }
                                                }
                                            }
                                            guard.write(_decoded, ramp_up_smpls, ramp_down_smpls);
                                        }
                                    }

                                    continue;
                                }
                                Err(symphonia::core::errors::Error::DecodeError(err)) => {
                                    info!("decode error: {}", err)
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
                                info!("End of stream!!");
                                let mut next_track = None;
                                while let Ok(value) =
                                    next_track_receiver.try_lock().unwrap().try_recv()
                                {
                                    next_track.replace(value);
                                }
                                if let Some(request) = next_track {
                                    if let Some(path) = request.path.clone() {
                                        is_transition = true;
                                        resampler_delay = guard.get_resampler_delay();
                                        info!("player: next track received! {:?}", request);
                                        path_str.replace(path);
                                        prev_seek = seek.unwrap_or(0.0);
                                        prev_song = song.clone();
                                        seek.replace(request.seek.unwrap());
                                        volume.replace(request.volume.unwrap());
                                        is_reset = false;
                                    } else {
                                        info!("player: nothing else in the queue");

                                        // Keep checking until all samples have been played (buffer is empty)
                                        while guard.has_remaining_samples() {
                                            info!("Buffer is not empty yet, waiting to pause...");
                                            thread::sleep(Duration::from_millis(500));
                                        }
                                        info!("Buffer is now empty. Pausing stream...");
                                        guard.pause();
                                        let _ = app_handle.emit("stopped", Some(0.0f64));
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
    info!("source {:?}", source);

    // Provide the file extension as a hint.
    info!("extension: {:?}", path.extension());
    if let Some(extension) = path.extension() {
        if let Some(extension_str) = extension.to_str() {
            hint.with_extension(extension_str);
        }
    }

    // Create the media source stream using the boxed media source from above.
    let mss = MediaSourceStream::new(source, Default::default());

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
    info!("probing {:?}", hint);
    info!("opts {:?}", format_opts);
    info!("meta {:?}", metadata_opts);

    let probe_result = get_probe().format(&hint, mss, &format_opts, &metadata_opts);
    info!("probe format {:?}", probe_result.is_ok());

    if probe_result.is_err() {
        return Err(probe_result.err().unwrap());
    }
    let mut reader = probe_result.unwrap().format;

    let track = reader.default_track().unwrap().clone();

    let track_id = track.id;

    info!("codec params: {:?}", &track.codec_params);

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions { verify: false })
        .unwrap();

    let new_spec = SignalSpec::new_with_layout(44100, Layout::Stereo);

    let expected_peaks_size =
        (track.codec_params.n_frames.unwrap() * new_spec.channels.count() as u64 / 4000) as usize;

    let mut window: Vec<f32> = Vec::with_capacity(4000);
    let mut peaks: Vec<f32> = Vec::new();

    let mut total_count = 0;
    let n_frames = 0;

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
                if cancel_token.is_cancelled() {
                    break Err(symphonia::core::errors::Error::LimitError("cancelled"));
                }
                // Create a raw sample buffer that matches the parameters of the decoded audio buffer.
                let mut sample_buf =
                    SampleBuffer::<f32>::new(_decoded.capacity() as u64, *_decoded.spec());

                // Copy the contents of the decoded audio buffer into the sample buffer whilst performing
                // any required conversions.
                sample_buf.copy_interleaved_ref(_decoded);
                sample_buf.samples().iter().for_each(|f| {
                    if window.len() < 4000 {
                        window.push(*f);
                    } else {
                        peaks.push(dsp::calculate_rms(&window));
                        window.clear();
                    }
                });

                total_count += 1;
                if total_count > 100 {
                    total_count = 0;
                    let len = expected_peaks_size.saturating_sub(peaks.len());
                    // info!("expected peaks size: {}, len: {}, n_adds: {}", expected_peaks_size, peaks.len(), n_adds);
                    let cln = [peaks.clone().as_slice(), vec![0f32; len].as_slice()].concat();
                    let _ = app_handle.emit("waveform", GetWaveformResponse { data: Some(cln) });
                }

                // Get waveform here
                continue;
            }
            Err(symphonia::core::errors::Error::DecodeError(err)) => {
                info!("decode error: {}", err)
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
            info!("End of stream!!");
            info!(
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AudioDevice {
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioDevices {
    devices: Vec<AudioDevice>,
    default: Option<AudioDevice>,
}

#[tauri::command]
pub fn get_devices(_app_handle: tauri::AppHandle) -> Option<AudioDevices> {
    // Get default host.
    let host = cpal::default_host();

    let cpal_devices: Vec<AudioDevice> = host
        .output_devices()
        .unwrap()
        .map(|device| AudioDevice {
            name: device.name().unwrap(),
        })
        .collect();

    let cpal_default = host.default_output_device();

    let default = if cpal_default.is_none() {
        None
    } else {
        Some(AudioDevice {
            name: cpal_default.unwrap().name().unwrap(),
        })
    };

    Some(AudioDevices {
        devices: cpal_devices,
        default,
    })
}
