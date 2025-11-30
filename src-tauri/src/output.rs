// Symphonia
// Copyright (c) 2019-2022 The Project Symphonia Developers.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Platform-dependant Audio Outputs

use std::result;
use std::sync::mpsc::{Receiver, Sender};

use ::cpal::traits::{DeviceTrait, HostTrait};
use ::cpal::{default_host, Device, SupportedStreamConfigRange};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Arc;

use symphonia::core::audio::{AudioBufferRef, SignalSpec};
use tokio::sync::Mutex;
use webrtc::data_channel::RTCDataChannel;

/// Small aliases to avoid repeating that long Arc<Mutex<...>> shape everywhere.
type LockedReceiver<T> = Arc<Mutex<Receiver<T>>>;
type LockedSender<T> = Arc<Mutex<Sender<T>>>;

/// Bundle all control channels into one argument.
#[derive(Clone)]
pub struct AudioControlHandles {
    pub volume_rx: LockedReceiver<crate::player::VolumeControlEvent>,
    pub sample_offset_rx: LockedReceiver<crate::SampleOffsetEvent>,
    pub playback_state_rx: LockedReceiver<PlaybackState>,
    pub timestamp_state_rx: LockedReceiver<TimestampState>,
    pub reset_control_rx: LockedReceiver<bool>,
    pub device_change_rx: LockedReceiver<String>,
    pub device_disconnected_tx: LockedSender<bool>,
    pub timestamp_tx: LockedSender<f64>,
    pub data_channel: Arc<tokio::sync::Mutex<Option<Arc<RTCDataChannel>>>>,
}

pub trait AudioOutput {
    fn write(&mut self, decoded: AudioBufferRef<'_>, ramp_up_samples: u64, ramp_down_samples: u64);
    fn flush(&mut self);
    #[allow(dead_code)]
    fn get_sample_rate(&self) -> u32;
    fn pause(&self);
    fn resume(&self);
    fn stop_stream(&mut self);
    fn update_resampler(
        &mut self,
        spec: SignalSpec,
        max_frames: u64,
        playback_speed: f64,
        is_reset: bool,
    ) -> bool;
    fn has_remaining_samples(&self) -> bool;
    fn get_resampler_delay(&self) -> f64;
    fn ramp_down(&mut self, buffer: AudioBufferRef, num_samples: usize);
    fn ramp_up(&mut self, buffer: AudioBufferRef, num_samples: usize);
}

pub struct PlaybackState {
    pub is_playing: bool,
    pub playback_speed: f64,
}

pub struct TimestampState {
    pub emit_to_client: usize, // 0 = not emit, 1 = emit
}

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum AudioOutputError {
    OpenStreamError,
    PlayStreamError,
    StreamClosedError,
}

pub type Result<T> = result::Result<T, AudioOutputError>;

mod cpal {
    use std::sync::mpsc::{Receiver, Sender};
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    use crate::constants::BUFFER_SIZE;
    use crate::output::{fft, get_device_by_name, ifft, AudioControlHandles, TimestampState};
    use crate::player::VolumeControlEvent;
    use crate::resampler::Resampler;
    use crate::SampleOffsetEvent;

    use super::{AudioOutput, AudioOutputError, PlaybackState, Result};

    use bytes::Bytes;
    use cpal::Sample;
    use symphonia::core::audio::{AudioBufferRef, Layout, RawSample, SampleBuffer, SignalSpec};
    use symphonia::core::conv::{ConvertibleSample, IntoSample};
    use symphonia::core::units::TimeBase;

    use cpal::traits::{DeviceTrait, StreamTrait};
    use rb::*;

    use log::{error, info};
    use tauri::{AppHandle, Emitter};
    use tokio::sync::Mutex;
    use webrtc::data_channel::RTCDataChannel;

    pub struct CpalAudioOutput {}

    trait AudioOutputSample:
        cpal::Sample
        + cpal::SizedSample
        + ConvertibleSample
        + IntoSample<f32>
        + RawSample
        + std::marker::Send
        + 'static
    {
    }

    impl AudioOutputSample for f32 {}
    impl AudioOutputSample for i16 {}
    impl AudioOutputSample for u16 {}

    impl CpalAudioOutput {
        pub fn try_open(
            device_name: &String,
            spec: SignalSpec,
            sample_buf_size: u64,
            controls: AudioControlHandles,
            vol: Option<f64>,
            app_handle: AppHandle,
        ) -> Result<Arc<Mutex<dyn AudioOutput>>> {
            let device = get_device_by_name(Some(device_name.clone())).unwrap();

            info!("Default audio device: {:?}", device.name());

            let config = match device.default_output_config() {
                Ok(config) => config,
                Err(err) => {
                    error!("failed to get default audio output device config: {}", err);
                    return Err(AudioOutputError::OpenStreamError);
                }
            };

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

            info!(
                "output: supports sample rate ({}) ? {}",
                spec.rate, supports_sample_rate
            );

            let rate = if supports_sample_rate {
                spec.rate
            } else {
                config.sample_rate().0
            };

            let device_spec = SignalSpec::new_with_layout(
                rate,
                match spec.channels.count() {
                    1 => Layout::Mono,
                    2 => Layout::Stereo,
                    3 => Layout::TwoPointOne,
                    5 => Layout::FivePointOne,
                    _ => Layout::Stereo,
                },
            );

            // Prepare the sample buffer size based on the maximum number of frames per packet
            let duration = sample_buf_size;
            info!("sample buffer size: {:?}", duration);

            // Select proper playback routine based on sample format.
            match config.sample_format() {
                cpal::SampleFormat::F32 => CpalAudioOutputImpl::<f32>::try_open(
                    device_spec,
                    duration,
                    &device,
                    controls,
                    |packet, volume| ((packet as f64) * volume) as f32,
                    |data| {
                        let fft_result = fft(&data);

                        let time_domain_signal = ifft(&fft_result);

                        Bytes::from(time_domain_signal)
                    },
                    vol,
                    app_handle,
                ),
                cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::try_open(
                    device_spec,
                    duration,
                    &device,
                    controls,
                    |packet: i16, volume: f64| {
                        ((packet as f64) * 10f64.powf(volume * 2.0 - 2.0))
                            .clamp(i16::MIN as f64, i16::MAX as f64) as i16
                    },
                    |data| {
                        let mut byte_array = Vec::with_capacity(data.len());

                        for d in &mut data.iter() {
                            byte_array.push(*d as u8);
                        }
                        Bytes::from(byte_array)
                    },
                    vol,
                    app_handle,
                ),
                cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::try_open(
                    device_spec,
                    duration,
                    &device,
                    controls,
                    |packet: u16, volume: f64| {
                        ((packet as f64) * 10f64.powf(volume * 2.0 - 2.0))
                            .clamp(0.0, u16::MAX as f64) as u16
                    },
                    |data| {
                        let mut byte_array = Vec::with_capacity(data.len());

                        for d in &mut data.iter() {
                            byte_array.push(*d as u8);
                        }
                        Bytes::from(byte_array)
                    },
                    vol,
                    app_handle,
                ),
                _ => CpalAudioOutputImpl::<f32>::try_open(
                    device_spec,
                    duration,
                    &device,
                    controls,
                    |packet, volume| ((packet as f64) * 10f64.powf(volume * 2.0 - 2.0)) as f32,
                    |data| {
                        let fft_result = fft(&data);

                        let time_domain_signal = ifft(&fft_result);

                        Bytes::from(time_domain_signal)
                    },
                    vol,
                    app_handle,
                ),
            }
        }
    }

    struct CpalAudioOutputImpl<T: AudioOutputSample>
    where
        T: AudioOutputSample + Send + Sync,
    {
        ring_buf: SpscRb<T>,
        ring_buf_producer: rb::Producer<T>,
        sample_buf: SampleBuffer<T>,
        stream: Option<cpal::Stream>,
        resampler: Option<Resampler<T>>,
        sample_rate: u32,
        name: String,
        time_base: TimeBase,
    }

    impl<T: AudioOutputSample + Send + Sync> CpalAudioOutputImpl<T> {
        pub fn try_open(
            spec: SignalSpec,
            duration: symphonia::core::units::Duration,
            device: &cpal::Device,
            controls: AudioControlHandles,
            volume_change: fn(T, f64) -> T,
            get_viz_bytes: fn(Vec<T>) -> Bytes,
            vol: Option<f64>,
            app_handle: AppHandle,
        ) -> Result<Arc<Mutex<dyn AudioOutput>>> {
            let num_channels = spec.channels.count();
            // Output audio stream config.
            let config = if cfg!(not(target_os = "windows")) {
                cpal::StreamConfig {
                    channels: num_channels as cpal::ChannelCount,
                    sample_rate: cpal::SampleRate(spec.rate),
                    buffer_size: cpal::BufferSize::Default,
                }
            } else {
                // Use the default config for Windows.
                device
                    .default_output_config()
                    .expect("Failed to get the default output config.")
                    .config()
            };

            let time_base = TimeBase {
                numer: 1,
                denom: config.sample_rate.0 * config.channels as u32,
            };

            // Create a ring buffer with a capacity
            let ring_len = (((BUFFER_SIZE as usize * 1000) * config.sample_rate.0 as usize) / 1000)
                * num_channels;

            let ring_buf = SpscRb::new(ring_len);
            let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());
            info!("Ring buffer capacity: {:?}", ring_buf.capacity());
            // States
            let volume_state = Arc::new(RwLock::new(vol.unwrap()));
            let frame_idx_state = Arc::new(RwLock::new(0.0f64));
            let elapsed_time_state = Arc::new(RwLock::new(0));
            let elapsed_frac_time_state = Arc::new(RwLock::new(0.0));
            let playback_state: Arc<RwLock<PlaybackState>> = Arc::new(RwLock::new(PlaybackState {
                is_playing: true,
                playback_speed: 1.0f64,
            }));
            let timestamp_state: Arc<RwLock<TimestampState>> =
                Arc::new(RwLock::new(TimestampState { emit_to_client: 1 }));

            let device_state = Arc::new(RwLock::new(
                device.name().unwrap_or(String::from("Unknown")),
            ));
            let device_name_state = Arc::new(RwLock::new(
                device.name().unwrap_or(String::from("Unknown")),
            ));
            let dc = Arc::new(controls.data_channel);

            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut viz_data = Vec::with_capacity(1024);

            let stream_result = device.build_output_stream(
                &config,
                move |data: &mut [T], _cb: &cpal::OutputCallbackInfo| {
                    // If the device changed, ignore callback
                    if let Ok(device_change) = controls.device_change_rx.try_lock() {
                        if let Ok(result) = device_change.try_recv() {
                            info!("Got device change: {:?}", result);
                            let mut dvc_state = device_state.write().unwrap();
                            *dvc_state = result;
                        }
                    }

                    if let Ok(dvc_state) = device_state.try_read() {
                        if *dvc_state != *device_name_state.try_read().unwrap() {
                            data.iter_mut().for_each(|s| *s = T::MID);
                            info!("Ignoring this device");
                            return;
                        }
                    }

                    let timestamp_control = controls.timestamp_state_rx.try_lock();
                    if let Ok(timestamp_state_lock) = timestamp_control {
                        let mut state = timestamp_state.write().unwrap();
                        if let Ok(control) = timestamp_state_lock.try_recv() {
                            info!("Got timestamp control: {:?}", control.emit_to_client);
                            *state = control;
                        }
                    }

                    // info!("playing back {:?}", data.len());
                    // If file changed, reset
                    let reset = controls.reset_control_rx.try_lock();
                    if let Ok(reset_lock) = reset {
                        if let Ok(rst) = reset_lock.try_recv() {
                            if rst {
                                info!("Got rst: {:?}", rst);
                                let mut frame_idx = frame_idx_state.write().unwrap();
                                *frame_idx = 0.0;
                                let mut elapsed_time = elapsed_time_state.write().unwrap();
                                *elapsed_time = 0;

                                let ts_state = timestamp_state.write().unwrap();
                                if ts_state.emit_to_client == 1 {
                                    let _ = app_handle.emit("timestamp", Some(0f64));
                                }
                            }
                        }
                    }

                    // Get volume
                    let volume = controls.volume_rx.try_lock();
                    if let Ok(volume_lock) = volume {
                        if let Ok(vol) = volume_lock.try_recv() {
                            info!("Got volume: {:?}", vol);
                            let mut current_volume = volume_state.write().unwrap();
                            *current_volume = vol.volume.unwrap();
                        }
                    }

                    let current_volume = { *volume_state.read().unwrap() };
                    // info!("Current volume: {:?}", current_volume);

                    let playing = controls.playback_state_rx.try_lock();
                    if let Ok(play_lock) = playing {
                        if let Ok(pl) = play_lock.try_recv() {
                            let mut current_playing = playback_state.write().unwrap();
                            if current_playing.playback_speed != pl.playback_speed {
                                // Update time base for correct timestamp calculation
                                // time_base = TimeBase {
                                //     numer: 1,
                                //     denom: (config.sample_rate.0 as f64
                                //         * config.channels as f64
                                //         * pl.playback_speed)
                                //         as u32,
                                // };
                                info!("Updated time base: {:?}", time_base);
                            }
                            *current_playing = pl;
                        }
                    }

                    // update duration if seconds changed
                    if let Ok(pl_state) = playback_state.try_read() {
                        if pl_state.is_playing {
                            // Write out as many samples as possible from the ring buffer to the audio
                            // output.
                            let written = ring_buf_consumer.read(data).unwrap_or(0);

                            let sample_offset = controls.sample_offset_rx.try_lock();
                            if let Ok(offset_lock) = sample_offset {
                                if let Ok(offset) = offset_lock.try_recv() {
                                    info!("Got sample offset: {:?}", offset);
                                    let mut current_sample_offset =
                                        frame_idx_state.write().unwrap();
                                    *current_sample_offset = offset.sample_offset.unwrap() as f64;
                                }
                            }

                            let mut i = 0;
                            for d in &mut *data {
                                *d = volume_change(*d, current_volume);
                                i += 1;
                            }

                            let length = data.len();

                            let mut should_send = false;
                            for d in &mut *data {
                                if viz_data.len() < length {
                                    viz_data.push(*d);
                                } else {
                                    should_send = true;
                                }
                            }

                            // new offset
                            let new_sample_offset = {
                                let mut sample_offset = frame_idx_state.write().unwrap();
                                *sample_offset += i as f64 * pl_state.playback_speed;
                                *sample_offset
                            };
                            // new duration
                            let next_duration =
                                time_base.calc_time(new_sample_offset as u64).seconds;
                            // info!("Next duration: {:?}", next_duration);

                            let prev_duration = { *elapsed_time_state.read().unwrap() };

                            // Second changed
                            if prev_duration != next_duration {
                                let new_duration =
                                    Duration::from_secs((next_duration as f64) as u64);

                                let ts_state = timestamp_state.write().unwrap();
                                if ts_state.emit_to_client == 1 {
                                    let _ = app_handle
                                        .emit("timestamp", Some(new_duration.as_secs_f64()));
                                }

                                // Also emit back to the decoding thread
                                let _ = controls
                                    .timestamp_tx
                                    .try_lock()
                                    .unwrap()
                                    .send(new_duration.as_secs_f64());

                                let mut duration = elapsed_time_state.write().unwrap();
                                *duration = new_duration.as_secs();
                            } else {
                                // Also emit events multiple times per second

                                // new duration
                                let current_frac =
                                    time_base.calc_time(new_sample_offset as u64).frac;
                                // info!("Next duration: {:?}", next_duration);

                                let prev_frac_emit = { *elapsed_frac_time_state.read().unwrap() };
                                // Every 0.2 seconds
                                if (current_frac - prev_frac_emit).abs() > 0.2 {
                                    let new_duration = Duration::from_secs_f64(
                                        next_duration as f64 + current_frac,
                                    );

                                    let ts_state = timestamp_state.write().unwrap();
                                    if ts_state.emit_to_client == 1 {
                                        let _ = app_handle
                                            .emit("timestamp", Some(new_duration.as_secs_f64()));
                                    }

                                    let mut duration = elapsed_frac_time_state.write().unwrap();
                                    *duration = current_frac;
                                }
                            }
                            let viz = viz_data.clone();
                            // Every x samples - send viz data to frontend
                            if should_send {
                                viz_data.clear();
                                if let Ok(dc_guard) = dc.try_lock() {
                                    if let Some(dc1) = dc_guard.as_ref().cloned() {
                                        rt.spawn(async move {
                                            let _ = dc1.send(&get_viz_bytes(viz)).await;
                                        });
                                    }
                                }
                            }
                            // Mute any remaining samples.
                            data[written..].iter_mut().for_each(|s| *s = T::MID);
                        } else {
                            data.iter_mut().for_each(|s| *s = T::MID);
                        }
                    } else {
                        data.iter_mut().for_each(|s| *s = T::MID);
                    }
                },
                move |err| {
                    match err {
                        cpal::StreamError::DeviceNotAvailable => {
                            error!("audio output stream device not available: {}", err);
                            if let Ok(guard) = controls.device_disconnected_tx.try_lock() {
                                info!("Sending device disconnected");
                                guard.send(true).unwrap();
                            }
                        }
                        _ => {
                            error!("audio output stream error: {}", err);
                            // TODO: Handle?
                        }
                    }
                },
                None,
            );

            if let Err(err) = stream_result {
                error!("audio output stream open error: {}", err);

                return Err(AudioOutputError::OpenStreamError);
            }

            let stream = Some(stream_result.unwrap());

            // Start the output stream.
            if let Err(err) = stream.as_ref().unwrap().play() {
                error!("audio output stream play error: {}", err);

                return Err(AudioOutputError::PlayStreamError);
            }

            let sample_buf = SampleBuffer::<T>::new(duration, spec);

            Ok(Arc::new(Mutex::new(CpalAudioOutputImpl {
                ring_buf,
                ring_buf_producer,
                sample_buf,
                stream,
                resampler: None,
                sample_rate: config.sample_rate.0,
                name: device.name().unwrap_or(String::from("Unknown")),
                time_base: time_base.clone(),
            })))
        }
    }

    impl<T: AudioOutputSample + Send + Sync> Drop for CpalAudioOutputImpl<T> {
        fn drop(&mut self) {
            info!("Audio output dropped: {}", self.name);
            self.stop_stream();
        }
    }

    impl<T: AudioOutputSample + Send + Sync> AudioOutput for CpalAudioOutputImpl<T> {
        fn write(
            &mut self,
            decoded: AudioBufferRef<'_>,
            ramp_up_samples: u64,
            ramp_down_samples: u64,
        ) -> () {
            // Do nothing if there are no audio frames.
            if decoded.frames() == 0 {
                info!("No more samples.");
                return;
            }

            // Print buffer size
            // {
            //     info!("decoded samples: {}", decoded.frames());
            //     // Current Buffer size
            //     info!("buffer samples: {}", self.sample_buf.samples().len());
            //     info!("ring buffer size: {}", self.ring_buf.count());
            // }

            let mut samples = if let Some(resampler) = &mut self.resampler {
                // Resampling is required. The resampler will return interleaved samples in the
                // correct sample format.
                match resampler.resample(decoded) {
                    Some(resampled) => resampled,
                    None => return,
                }
            } else {
                // info!(
                //     "sample_buf capacity {:?} | decoded size {:?}",
                //     self.sample_buf.capacity(),
                //     decoded.spec().channels.count() * decoded.frames()
                // );
                if self.sample_buf.capacity() >= decoded.spec().channels.count() * decoded.frames()
                {
                    // Resampling is not required. Interleave the sample for cpal using a sample buffer.
                    if ramp_up_samples > 0 {
                        info!("Ramping up first {:?}", ramp_up_samples);
                        self.ramp_up(decoded, ramp_up_samples as usize);
                        self.sample_buf.samples()
                    } else if ramp_down_samples > 0 {
                        info!("Ramping down last {:?}", ramp_down_samples);
                        self.ramp_down(decoded, ramp_down_samples as usize);
                        self.sample_buf.samples()
                    } else {
                        self.sample_buf.copy_interleaved_ref(decoded);
                        self.sample_buf.samples()
                    }
                } else {
                    // The sample buffer is not big enough to process all the samples.
                    // TODO Error?
                    return;
                }
            };

            // Write all samples to the ring buffer.
            // info!("Writing samples: {}", samples.len());
            while let Ok(Some(written)) = self
                .ring_buf_producer
                .write_blocking_timeout(samples, Duration::from_secs_f64(0.5))
            {
                samples = &samples[written..];
                // Print written
                // info!("written: {}", written);
            }
        }

        fn flush(&mut self) {
            // If there is a resampler, then it may need to be flushed
            // depending on the number of samples it has.
            if let Some(resampler) = &mut self.resampler {
                while let Some(remaining_samples) = resampler.flush() {
                    info!("Flushed samples {:?}", remaining_samples.len());
                }
            }

            // Flush is best-effort, ignore the returned result.

            self.sample_buf.clear();
            self.ring_buf.clear();

            // Check what's left now
            info!(
                "Sample buf empty: {}, Ring buf empty:{} ",
                self.sample_buf.is_empty(),
                self.ring_buf.is_empty()
            );
        }

        fn get_sample_rate(&self) -> u32 {
            return self.sample_rate;
        }

        fn pause(&self) {
            let pause_result = self.stream.as_ref().unwrap().pause();
            info!("cpal: Stream pause result: {:?}", pause_result);
        }

        fn resume(&self) {
            let resume_result = self.stream.as_ref().unwrap().play();
            info!("cpal: Stream resume result: {:?}", resume_result);
        }

        // Explicitly drop the stream
        fn stop_stream(&mut self) {
            if let Some(stream) = self.stream.take() {
                // Dropping the stream explicitly
                std::mem::drop(stream);
            }
            info!("Audio output stopped: {}", self.name);
        }

        fn update_resampler(
            &mut self,
            spec: SignalSpec,
            max_frames: u64,
            playback_speed: f64,
            is_reset: bool,
        ) -> bool {
            // When resampling is required eg. 48khz -> 44.1khz, calculate the target speed ratio.
            // Optional playback rate adjustment is added on top
            let adjusted_speed = if spec.rate != self.sample_rate {
                info!("resampling {} Hz to {} Hz", spec.rate, self.sample_rate);
                spec.rate as f64 / self.sample_rate as f64 * playback_speed
            } else {
                playback_speed
            };

            info!(
                "requested speed: {}, adjusted speed: {}",
                playback_speed, adjusted_speed
            );

            // Custom speed (slowed down / sped up)
            if adjusted_speed != 1.0f64 {
                info!("Resampling for {:.2}x playback speed", adjusted_speed);
                // spec.rate = (spec.rate as f32 * playback_speed) as u32;
                if let Some(resampler) = &mut self.resampler {
                    resampler.set_playback_rate(adjusted_speed as f64);
                    if is_reset {
                        resampler.set_playback_pos(0.0);
                        resampler.flush();
                    }
                } else {
                    self.resampler.replace(Resampler::with_playback_rate(
                        spec,
                        max_frames,
                        adjusted_speed,
                    ));
                }
                return true;
            } else {
                // Original speed - 1x
                if let Some(resampler) = &mut self.resampler {
                    if resampler.playback_rate != adjusted_speed {
                        // Back to original speed - ramp back to 1.0
                        // and keep resampling instead of abruptly switching
                        resampler.set_playback_rate(adjusted_speed);
                        return true;
                    }
                    // When switching tracks, we can remove the resampler if not required
                    if is_reset {
                        if self.sample_rate != spec.rate {
                            self.resampler.replace(Resampler::new(spec, max_frames));
                            return true;
                        } else {
                            self.resampler.take();
                            return false;
                        }
                    }
                }
                return false;
            }

            // If we have a default audio device (we always should, but just in case)
            // we check if the track spec differs from the output device
            // if it does - resample the decoded audio using Symphonia.
        }

        /// Checks if there are any samples left in the buffer that have not been played yet.
        fn has_remaining_samples(&self) -> bool {
            !self.ring_buf.is_empty()
        }

        fn get_resampler_delay(&self) -> f64 {
            if let Some(resampler) = &self.resampler {
                let remaining_samples = resampler.get_remaining_samples();
                let time = self.time_base.calc_time(remaining_samples);
                info!("remaining_samples: {}, time: {:?}", remaining_samples, time);
                time.seconds as f64 + time.frac
            } else {
                0.0
            }
        }

        fn ramp_down(&mut self, buffer: AudioBufferRef, num_samples: usize) {
            self.sample_buf.copy_interleaved_ref(buffer);
            let ramp_len = num_samples.min(self.sample_buf.len());

            for (i, sample) in self.sample_buf.samples_mut()[..ramp_len]
                .iter_mut()
                .enumerate()
            {
                let factor = 1.0 - (i as f32 / ramp_len as f32);
                sample.mul_amp(factor.to_sample());
            }
        }

        fn ramp_up(&mut self, buffer: AudioBufferRef, num_samples: usize) {
            self.sample_buf.copy_interleaved_ref(buffer);
            let ramp_len = num_samples.min(self.sample_buf.len());

            for (i, sample) in self.sample_buf.samples_mut()[..ramp_len]
                .iter_mut()
                .enumerate()
            {
                let factor = i as f32 / ramp_len as f32;
                sample.mul_amp(factor.to_sample());
            }
        }
    }
}

pub fn try_open(
    device_name: &String,
    spec: SignalSpec,
    sample_buf_size: u64,
    controls: AudioControlHandles,
    vol: Option<f64>,
    app_handle: tauri::AppHandle,
) -> Result<Arc<tokio::sync::Mutex<dyn AudioOutput>>> {
    cpal::CpalAudioOutput::try_open(
        device_name,
        spec,
        sample_buf_size,
        controls,
        vol,
        app_handle,
    )
}

pub fn get_device_by_name(name: Option<String>) -> Option<Device> {
    let host = default_host();
    if name.is_none() {
        return host.default_output_device();
    }
    let name = name.unwrap();
    return host
        .devices()
        .unwrap()
        .find(|device| {
            device.name().unwrap() == name
                && device.supported_output_configs().is_ok_and(|configs| {
                    let mut has = false;
                    for _config in configs {
                        has = true;
                    }
                    has
                })
        })
        .or(host.default_output_device());
}

pub fn default_device() -> Option<Device> {
    let host = default_host();
    return host.default_output_device();
}

pub struct DeviceWithConfig {
    pub device: Device,
    pub config: Vec<SupportedStreamConfigRange>,
}

pub fn enumerate_devices() -> Vec<DeviceWithConfig> {
    let host = default_host();
    host.devices()
        .unwrap()
        // Map to tuple of device and supported output configs
        // Filter out devices that don't support output
        .filter_map(|device| {
            device
                .supported_output_configs()
                .ok()
                .map(|configs| DeviceWithConfig {
                    device: device,
                    config: configs.collect(),
                })
                .filter(|d| d.config.len() > 0)
        })
        .collect()
}

fn fft(input: &[f32]) -> Vec<Complex<f32>> {
    let len = input.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(len);

    // Apply Hanning window
    // let windowed_input: Vec<f32> = input
    //     .iter()
    //     .enumerate()
    //     .map(|(i, &x)| x * hamming_window(i, len))
    //     .collect();

    // Convert input into complex numbers
    let mut complex_input: Vec<Complex<f32>> =
        input.iter().map(|&x| Complex::new(x, 0.0)).collect();

    // Perform FFT
    fft.process(&mut complex_input);

    complex_input
}

fn ifft(input: &[Complex<f32>]) -> Vec<u8> {
    let len = input.len();
    let mut planner = FftPlanner::new();
    let ifft = planner.plan_fft_inverse(len);

    let mut output: Vec<Complex<f32>> = input.to_vec();

    // Perform inverse FFT
    ifft.process(&mut output);

    // Extract real parts and scale
    let mut time_domain_signal = output.iter().map(|&freq| freq.re).collect::<Vec<f32>>();

    // Remove any residual imaginary part due to precision issues
    for val in time_domain_signal.iter_mut() {
        if val.abs() < 1e-6 {
            *val = 0.0;
        }
    }

    let interpolated_signal: Vec<f32> = time_domain_signal
        .windows(2)
        .map(|pair| {
            let (x0, x1) = (pair[0], pair[1]);
            smoothing(x0, x1, 0.2f32)
        })
        .collect();

    let mut interleaved_bytes: Vec<u8> = Vec::new();

    // Iterate through each complex number in the FFT result
    for i in 0..interpolated_signal.len() {
        // Every 2nd sample, sum the last two together and divide by two
        if i % 2 == 0 && i + 1 < interpolated_signal.len() {
            let freq1 = interpolated_signal[i];
            let freq2 = interpolated_signal[i + 1];

            // Calculate magnitude of the complex number
            // let magnitude1 = (freq1.re.powi(2) + freq1.im.powi(2)).sqrt();
            // let magnitude2 = (freq2.re.powi(2) + freq2.im.powi(2)).sqrt();
            let summed = (((freq1 - freq2) * 0.8 / 2.0) + 128.0) as u8;
            // info!("L: {}, R: {}, summed: {}", freq1, freq2, summed);

            // Split the f32 into its individual bytes
            // let bytes = summed.to_ne_bytes();
            interleaved_bytes.push(summed);

            // Interleave the bytes
            // for &byte in bytes.iter() {
            //     interleaved_bytes.push(byte);
            // }
        }
    }

    interleaved_bytes
}

fn smoothing(x0: f32, x1: f32, factor: f32) -> f32 {
    x1 + ((x0 - x1) * factor)
}
