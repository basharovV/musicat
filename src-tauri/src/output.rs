// Symphonia
// Copyright (c) 2019-2022 The Project Symphonia Developers.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Platform-dependant Audio Outputs

use std::f32::consts::PI;
use std::result;

use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Arc;

use symphonia::core::audio::{AudioBufferRef, SignalSpec};
use symphonia::core::units::Duration;
use webrtc::data_channel::RTCDataChannel;

pub trait AudioOutput {
    fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()>;
    fn flush(&mut self);
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

#[cfg(target_os = "linux")]
mod pulseaudio {
    use super::{AudioOutput, AudioOutputError, Result};

    use symphonia::core::audio::*;
    use symphonia::core::units::Duration;

    use libpulse_binding as pulse;
    use libpulse_simple_binding as psimple;

    use log::{error, warn};

    pub struct PulseAudioOutput {
        pa: psimple::Simple,
        sample_buf: RawSampleBuffer<f32>,
    }

    impl PulseAudioOutput {
        pub fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn AudioOutput>> {
            // An interleaved buffer is required to send data to PulseAudio. Use a SampleBuffer to
            // move data between Symphonia AudioBuffers and the byte buffers required by PulseAudio.
            let sample_buf = RawSampleBuffer::<f32>::new(duration, spec);

            // Create a PulseAudio stream specification.
            let pa_spec = pulse::sample::Spec {
                format: pulse::sample::Format::FLOAT32NE,
                channels: spec.channels.count() as u8,
                rate: spec.rate,
            };

            assert!(pa_spec.is_valid());

            let pa_ch_map = map_channels_to_pa_channelmap(spec.channels);

            // PulseAudio seems to not play very short audio buffers, use these custom buffer
            // attributes for very short audio streams.
            //
            // let pa_buf_attr = pulse::def::BufferAttr {
            //     maxlength: std::u32::MAX,
            //     tlength: 1024,
            //     prebuf: std::u32::MAX,
            //     minreq: std::u32::MAX,
            //     fragsize: std::u32::MAX,
            // };

            // Create a PulseAudio connection.
            let pa_result = psimple::Simple::new(
                None,                               // Use default server
                "Symphonia Player",                 // Application name
                pulse::stream::Direction::Playback, // Playback stream
                None,                               // Default playback device
                "Music",                            // Description of the stream
                &pa_spec,                           // Signal specification
                pa_ch_map.as_ref(),                 // Channel map
                None,                               // Custom buffering attributes
            );

            match pa_result {
                Ok(pa) => Ok(Box::new(PulseAudioOutput { pa, sample_buf })),
                Err(err) => {
                    error!("audio output stream open error: {}", err);

                    Err(AudioOutputError::OpenStreamError)
                }
            }
        }
    }

    impl AudioOutput for PulseAudioOutput {
        fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {
            // Do nothing if there are no audio frames.
            if decoded.frames() == 0 {
                return Ok(());
            }

            // Interleave samples from the audio buffer into the sample buffer.
            self.sample_buf.copy_interleaved_ref(decoded);

            // Write interleaved samples to PulseAudio.
            match self.pa.write(self.sample_buf.as_bytes()) {
                Err(err) => {
                    error!("audio output stream write error: {}", err);

                    Err(AudioOutputError::StreamClosedError)
                }
                _ => Ok(()),
            }
        }

        fn flush(&mut self) {
            // Flush is best-effort, ignore the returned result.
            let _ = self.pa.drain();
        }
    }

    /// Maps a set of Symphonia `Channels` to a PulseAudio channel map.
    fn map_channels_to_pa_channelmap(channels: Channels) -> Option<pulse::channelmap::Map> {
        let mut map: pulse::channelmap::Map = Default::default();
        map.init();
        map.set_len(channels.count() as u8);

        let is_mono = channels.count() == 1;

        for (i, channel) in channels.iter().enumerate() {
            map.get_mut()[i] = match channel {
                Channels::FRONT_LEFT if is_mono => pulse::channelmap::Position::Mono,
                Channels::FRONT_LEFT => pulse::channelmap::Position::FrontLeft,
                Channels::FRONT_RIGHT => pulse::channelmap::Position::FrontRight,
                Channels::FRONT_CENTRE => pulse::channelmap::Position::FrontCenter,
                Channels::REAR_LEFT => pulse::channelmap::Position::RearLeft,
                Channels::REAR_CENTRE => pulse::channelmap::Position::RearCenter,
                Channels::REAR_RIGHT => pulse::channelmap::Position::RearRight,
                Channels::LFE1 => pulse::channelmap::Position::Lfe,
                Channels::FRONT_LEFT_CENTRE => pulse::channelmap::Position::FrontLeftOfCenter,
                Channels::FRONT_RIGHT_CENTRE => pulse::channelmap::Position::FrontRightOfCenter,
                Channels::SIDE_LEFT => pulse::channelmap::Position::SideLeft,
                Channels::SIDE_RIGHT => pulse::channelmap::Position::SideRight,
                Channels::TOP_CENTRE => pulse::channelmap::Position::TopCenter,
                Channels::TOP_FRONT_LEFT => pulse::channelmap::Position::TopFrontLeft,
                Channels::TOP_FRONT_CENTRE => pulse::channelmap::Position::TopFrontCenter,
                Channels::TOP_FRONT_RIGHT => pulse::channelmap::Position::TopFrontRight,
                Channels::TOP_REAR_LEFT => pulse::channelmap::Position::TopRearLeft,
                Channels::TOP_REAR_CENTRE => pulse::channelmap::Position::TopRearCenter,
                Channels::TOP_REAR_RIGHT => pulse::channelmap::Position::TopRearRight,
                _ => {
                    // If a Symphonia channel cannot map to a PulseAudio position then return None
                    // because PulseAudio will not be able to open a stream with invalid channels.
                    warn!("failed to map channel {:?} to output", channel);
                    return None;
                }
            }
        }

        Some(map)
    }
}

#[cfg(not(target_os = "linux"))]
mod cpal {
    use std::io::Read;
    use std::ops::Mul;
    use std::sync::mpsc::Receiver;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    use crate::output::{fft, ifft};
    use crate::resampler::Resampler;
    use crate::{SampleOffsetEvent, VolumeControlEvent};

    use super::{AudioOutput, AudioOutputError, Result};

    use bytes::Bytes;
    use rayon::iter::IntoParallelRefIterator;
    use symphonia::core::audio::{AudioBufferRef, RawSample, SampleBuffer, SignalSpec};
    use symphonia::core::conv::{ConvertibleSample, IntoSample};
    use symphonia::core::units::TimeBase;

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use rb::*;

    use log::{error, info};
    use tauri::{AppHandle, Manager};
    use tokio::sync::Mutex;
    use webrtc::data_channel::RTCDataChannel;

    pub struct CpalAudioOutput;

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
            spec: SignalSpec,
            duration: symphonia::core::units::Duration,
            volume_control_receiver: Arc<Mutex<Receiver<VolumeControlEvent>>>,
            sample_offset_receiver: Arc<Mutex<Receiver<SampleOffsetEvent>>>,
            playback_state_receiver: Arc<Mutex<Receiver<bool>>>,
            data_channel: Arc<tokio::sync::Mutex<Option<Arc<RTCDataChannel>>>>,
            app_handle: AppHandle,
        ) -> Result<Arc<Mutex<dyn AudioOutput>>> {
            // Get default host.
            let host = cpal::default_host();

            // Get the default audio output device.
            let device = match host.default_output_device() {
                Some(device) => device,
                _ => {
                    error!("failed to get default audio output device");
                    return Err(AudioOutputError::OpenStreamError);
                }
            };

            println!("Default audio device: {:?}", device.name());

            let config = match device.default_output_config() {
                Ok(config) => config,
                Err(err) => {
                    error!("failed to get default audio output device config: {}", err);
                    return Err(AudioOutputError::OpenStreamError);
                }
            };

            // Select proper playback routine based on sample format.
            match config.sample_format() {
                cpal::SampleFormat::F32 => CpalAudioOutputImpl::<f32>::try_open(
                    spec,
                    duration,
                    &device,
                    volume_control_receiver,
                    sample_offset_receiver,
                    playback_state_receiver,
                    data_channel,
                    |packet, volume| ((packet as f64) * volume) as f32,
                    |data| {
                        let fft_result = fft(&data);

                        let time_domain_signal = ifft(&fft_result);

                        Bytes::from(time_domain_signal)
                    },
                    app_handle,
                ),
                cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::try_open(
                    spec,
                    duration,
                    &device,
                    volume_control_receiver,
                    sample_offset_receiver,
                    playback_state_receiver,
                    data_channel,
                    |packet, volume| ((packet as f64) * volume) as i16,
                    |data| {
                        let length = data.len();
                        let mut byte_array = Vec::with_capacity(data.len());

                        let mut i = 0;
                        for d in &mut data.iter() {
                            byte_array.push(*d as u8);
                            i += 1;
                        }
                        Bytes::from(byte_array)
                    },
                    app_handle,
                ),
                cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::try_open(
                    spec,
                    duration,
                    &device,
                    volume_control_receiver,
                    sample_offset_receiver,
                    playback_state_receiver,
                    data_channel,
                    |packet, volume| ((packet as f64) * volume) as u16,
                    |data| {
                        let length = data.len();
                        let mut byte_array = Vec::with_capacity(data.len());

                        let mut i = 0;
                        for d in &mut data.iter() {
                            byte_array.push(*d as u8);
                            i += 1;
                        }
                        Bytes::from(byte_array)
                    },
                    app_handle,
                ),
                _ => CpalAudioOutputImpl::<f32>::try_open(
                    spec,
                    duration,
                    &device,
                    volume_control_receiver,
                    sample_offset_receiver,
                    playback_state_receiver,
                    data_channel,
                    |packet, volume| ((packet as f64) * volume) as f32,
                    |data| {
                        let fft_result = fft(&data);

                        let time_domain_signal = ifft(&fft_result);

                        Bytes::from(time_domain_signal)
                    },
                    app_handle,
                ),
            }
        }
    }

    struct CpalAudioOutputImpl<T: AudioOutputSample>
    where
        T: AudioOutputSample + Send + Sync,
    {
        ring_buf_producer: rb::Producer<T>,
        sample_buf: SampleBuffer<T>,
        stream: cpal::Stream,
        resampler: Option<Resampler<T>>,
    }

    fn change_volume<T>(data: &mut [T], volume: Option<f64>)
    where
        T: Mul<f64, Output = T> + Copy,
    {
        for d in data {
            *d = *d * volume.unwrap_or(0.5f64);
        }
    }

    impl<T: AudioOutputSample + Send + Sync> CpalAudioOutputImpl<T> {
        pub fn try_open(
            spec: SignalSpec,
            duration: symphonia::core::units::Duration,
            device: &cpal::Device,
            volume_control_receiver: Arc<Mutex<Receiver<VolumeControlEvent>>>,
            sample_offset_receiver: Arc<Mutex<Receiver<SampleOffsetEvent>>>,
            playback_state_receiver: Arc<Mutex<Receiver<bool>>>,
            data_channel: Arc<tokio::sync::Mutex<Option<Arc<RTCDataChannel>>>>,
            volume_change: fn(T, f64) -> T,
            get_viz_bytes: fn(Vec<T>) -> Bytes,
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

            // Create a ring buffer with a capacity for up-to 200ms of audio.
            let ring_len = ((200 * config.sample_rate.0 as usize) / 1000) * num_channels;

            let ring_buf = SpscRb::new(ring_len);
            let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());

            // States
            let volume_state = Arc::new(RwLock::new(0.5f64));
            let frame_idx_state = Arc::new(RwLock::new(0));
            let elapsed_time_state = Arc::new(RwLock::new(0));
            let playback_state = Arc::new(RwLock::new(true));
            let dc = Arc::new(data_channel);

            let mut rt = tokio::runtime::Runtime::new().unwrap();

            let stream_result = device.build_output_stream(
                &config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    // Get volume
                    let volume = volume_control_receiver.try_lock().unwrap().try_recv();
                    if let Ok(vol) = volume {
                        println!("Got volume: {:?}", vol);
                        let mut current_volume = volume_state.write().unwrap();
                        *current_volume = vol.volume.unwrap();
                    }

                    let current_volume = { *volume_state.read().unwrap() };
                    // println!("Current volume: {:?}", current_volume);

                    // Write out as many samples as possible from the ring buffer to the audio
                    // output.
                    let written = ring_buf_consumer.read(data).unwrap_or(0);

                    let sample_offset = sample_offset_receiver.try_lock().unwrap().try_recv();

                    if let Ok(offset) = sample_offset {
                        println!("Got sample offset: {:?}", offset);
                        let mut current_sample_offset = frame_idx_state.write().unwrap();
                        *current_sample_offset = offset.sample_offset.unwrap();
                    }

                    let mut i = 0;
                    for d in &mut *data {
                        *d = volume_change(*d, current_volume);
                        i += 1;
                    }

                    let length = data.len();
                    let mut viz_data = Vec::with_capacity(data.len());

                    let mut u = 0;
                    for d in &mut *data {
                        viz_data.push(*d);
                        u += 1;
                    }

                    let playing = playback_state_receiver.try_lock().unwrap().try_recv();
                    if let Ok(pl) = playing {
                        let mut current_playing = playback_state.write().unwrap();
                        *current_playing = pl;
                    }

                    // update duration if seconds changed
                    if (*playback_state.try_read().unwrap()) {
                        // new offset
                        let new_sample_offset = {
                            let mut sample_offset = frame_idx_state.write().unwrap();
                            *sample_offset += i;
                            *sample_offset
                        };
                        // new duration
                        let next_duration = time_base.calc_time(new_sample_offset as u64).seconds;
                        // println!("Next duration: {:?}", next_duration);

                        let prev_duration = { *elapsed_time_state.read().unwrap() };

                        if prev_duration != next_duration {
                            let new_duration = Duration::from_secs(next_duration);

                            app_handle.emit_all("timestamp", Some(new_duration.as_secs_f64()));

                            let mut duration = elapsed_time_state.write().unwrap();
                            *duration = new_duration.as_secs();
                        }

                        // Every x samples - send viz data to frontend
                        if let Ok(dc_guard) = dc.try_lock() {
                            if let Some(dc1) = dc_guard.as_ref().cloned() {
                                rt.spawn(async move {
                                    dc1.send(&get_viz_bytes(viz_data)).await;
                                });
                            }
                        }
                    }

                    // Mute any remaining samples.
                    data[written..].iter_mut().for_each(|s| *s = T::MID);
                },
                move |err| error!("audio output error: {}", err),
                None,
            );

            if let Err(err) = stream_result {
                error!("audio output stream open error: {}", err);

                return Err(AudioOutputError::OpenStreamError);
            }

            let stream = stream_result.unwrap();

            // Start the output stream.
            if let Err(err) = stream.play() {
                error!("audio output stream play error: {}", err);

                return Err(AudioOutputError::PlayStreamError);
            }

            let sample_buf = SampleBuffer::<T>::new(duration, spec);

            let resampler = if spec.rate != config.sample_rate.0 {
                info!("resampling {} Hz to {} Hz", spec.rate, config.sample_rate.0);
                Some(Resampler::new(
                    spec,
                    config.sample_rate.0 as usize,
                    duration,
                ))
            } else {
                None
            };

            Ok(Arc::new(Mutex::new(CpalAudioOutputImpl {
                ring_buf_producer,
                sample_buf,
                stream,
                resampler,
            })))
        }
    }

    impl<T: AudioOutputSample + Send + Sync> AudioOutput for CpalAudioOutputImpl<T> {
        fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {
            // Do nothing if there are no audio frames.
            if decoded.frames() == 0 {
                return Ok(());
            }

            let mut samples = if let Some(resampler) = &mut self.resampler {
                // Resampling is required. The resampler will return interleaved samples in the
                // correct sample format.
                match resampler.resample(decoded) {
                    Some(resampled) => resampled,
                    None => return Ok(()),
                }
            } else {
                // Resampling is not required. Interleave the sample for cpal using a sample buffer.
                self.sample_buf.copy_interleaved_ref(decoded);

                self.sample_buf.samples()
            };

            // Write all samples to the ring buffer.
            while let Some(written) = self.ring_buf_producer.write_blocking(samples) {
                samples = &samples[written..];
            }

            Ok(())
        }

        fn flush(&mut self) {
            // If there is a resampler, then it may need to be flushed
            // depending on the number of samples it has.
            if let Some(resampler) = &mut self.resampler {
                let mut remaining_samples = resampler.flush().unwrap_or_default();

                while let Some(written) = self.ring_buf_producer.write_blocking(remaining_samples) {
                    remaining_samples = &remaining_samples[written..];
                }
            }

            // Flush is best-effort, ignore the returned result.
            let _ = self.stream.pause();
        }
    }
}

#[cfg(target_os = "linux")]
pub fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn AudioOutput>> {
    pulseaudio::PulseAudioOutput::try_open(spec, duration)
}

#[cfg(not(target_os = "linux"))]
pub fn try_open(
    spec: SignalSpec,
    duration: Duration,
    volume_control_receiver: Arc<
        tokio::sync::Mutex<std::sync::mpsc::Receiver<crate::VolumeControlEvent>>,
    >,
    sample_offset_receiver: Arc<
        tokio::sync::Mutex<std::sync::mpsc::Receiver<crate::SampleOffsetEvent>>,
    >,
    playback_state_receiver: Arc<tokio::sync::Mutex<std::sync::mpsc::Receiver<bool>>>,
    data_channel: Arc<tokio::sync::Mutex<Option<Arc<RTCDataChannel>>>>,
    app_handle: tauri::AppHandle,
) -> Result<Arc<tokio::sync::Mutex<dyn AudioOutput>>> {
    cpal::CpalAudioOutput::try_open(
        spec,
        duration,
        volume_control_receiver,
        sample_offset_receiver,
        playback_state_receiver,
        data_channel,
        app_handle,
    )
}

fn hanning_window(n: usize, N: usize) -> f32 {
    0.5 * (1.0 - ((2.0 * PI * n as f32) / (N as f32 - 1.0)).cos())
}

fn hamming_window(n: usize, N: usize) -> f32 {
    0.54 - 0.46 * ((2.0 * std::f32::consts::PI * n as f32) / (N as f32 - 1.0)).cos()
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
    let mut complex_input: Vec<Complex<f32>> = input
        .iter()
        .map(|&x| Complex::new(x, 0.0))
        .collect();

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

    let mut interleaved_bytes: Vec<u8> = Vec::new();

    // Iterate through each complex number in the FFT result
    for i in 0..time_domain_signal.len() {
        // Every 2nd sample, sum the last two together and divide by two
        if i % 2 == 0 && i + 1 < time_domain_signal.len() {
            let freq1 = time_domain_signal[i];
            let freq2 = time_domain_signal[i + 1];

            // Calculate magnitude of the complex number
            // let magnitude1 = (freq1.re.powi(2) + freq1.im.powi(2)).sqrt();
            // let magnitude2 = (freq2.re.powi(2) + freq2.im.powi(2)).sqrt();
            let summed = (((freq1 - freq2) * 0.8 / 2.0) + 128.0) as u8;
            // println!("L: {}, R: {}, summed: {}", freq1, freq2, summed);

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
