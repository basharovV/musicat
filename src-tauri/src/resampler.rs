// Symphonia
// Copyright (c) 2019-2022 The Project Symphonia Developers.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Signal, SignalSpec};
use symphonia::core::conv::{FromSample, IntoSample};
use symphonia::core::sample::Sample;

pub struct Resampler<T> {
    pub playback_rate: f64, // Playback rate multiplier (e.g., 1.0 = normal, 0.5 = half speed)
    pub playback_pos: f64,  // Fractional position in the input buffer
    input: Vec<Vec<f32>>,   // Input audio data (planar)
    interleaved: Vec<T>,    // Interleaved output samples
    duration: usize,        // Input buffer size in frames,
}

impl<T> Resampler<T>
where
    T: Sample + FromSample<f32> + IntoSample<f32>,
{
    fn resample_inner(&mut self) -> &[T] {
        let num_channels = self.input.len();
        let output_len = self.duration;
        // info!(
        //     "playback_pos: {}, input len: {}",
        //     self.playback_pos,
        //     self.input[0].len()
        // );
        let sinc_window_size = 8; // Adjust for quality vs performance

        self.interleaved.resize(output_len * num_channels, T::MID);

        for (i, frame) in self.interleaved.chunks_exact_mut(num_channels).enumerate() {
            if self.playback_pos >= self.input[0].len() as f64 {
                // Exit early if past the end of the input buffer
                self.interleaved.truncate(i * num_channels);
                break;
            }

            for ch in 0..num_channels {
                let int_pos = self.playback_pos as usize;
                let frac = self.playback_pos - int_pos as f64;

                // Sinc interpolation
                let mut sample = 0.0;
                for offset in -(sinc_window_size as isize)..=(sinc_window_size as isize) {
                    let sinc_value = windowed_sinc((frac - offset as f64) as f64, sinc_window_size);
                    let pos = int_pos as isize + offset;
                    if pos >= 0 && (pos as usize) < self.input[ch].len() {
                        sample += self.input[ch][pos as usize] * sinc_value as f32;
                    }
                }

                frame[ch] = sample.into_sample();
            }

            // Advance playback position by playback rate
            self.playback_pos += self.playback_rate;
        }

        &self.interleaved
    }

    pub fn new(spec: SignalSpec, duration: u64) -> Self {
        let duration = duration as usize;
        let num_channels = spec.channels.count();

        let input = vec![Vec::with_capacity(duration); num_channels];

        Self {
            playback_rate: 1.0, // Default playback rate is normal speed
            playback_pos: 0.0,  // Start at the beginning
            input,
            interleaved: Default::default(),
            duration,
        }
    }

    pub fn with_playback_rate(spec: SignalSpec, duration: u64, playback_rate: f64) -> Self {
        let duration = duration as usize;
        let num_channels = spec.channels.count();

        let input = vec![Vec::with_capacity(duration); num_channels];

        Self {
            playback_rate,     // Default playback rate is normal speed
            playback_pos: 0.0, // Start at the beginning
            input,
            interleaved: Default::default(),
            duration,
        }
    }

    pub fn resample(&mut self, input: AudioBufferRef<'_>) -> Option<&[T]> {
        // Copy and convert samples into input buffer.
        convert_samples_any(&input, &mut self.input);

        // Check if there is enough data to produce output
        if self.input[0].len() < self.duration {
            return None;
        }

        Some(self.resample_inner())
    }

    pub fn flush(&mut self) -> Option<&[T]> {
        let len = self.input[0].len();
        if len == 0 {
            return None;
        }

        // Adjust buffer for remaining samples
        for channel in self.input.iter_mut() {
            channel.drain(0..len);
        }

        None
    }

    pub fn set_playback_rate(&mut self, rate: f64) {
        self.playback_rate = rate;
    }

    pub fn set_playback_pos(&mut self, pos: f64) {
        self.playback_pos = pos;
    }

    pub fn get_remaining_samples(&self) -> u64 {
        (self.input[0].len() as u64).saturating_sub(self.playback_pos as u64)
    }
}

fn convert_samples_any(input: &AudioBufferRef<'_>, output: &mut [Vec<f32>]) {
    match input {
        AudioBufferRef::U8(input) => convert_samples(input, output),
        AudioBufferRef::U16(input) => convert_samples(input, output),
        AudioBufferRef::U24(input) => convert_samples(input, output),
        AudioBufferRef::U32(input) => convert_samples(input, output),
        AudioBufferRef::S8(input) => convert_samples(input, output),
        AudioBufferRef::S16(input) => convert_samples(input, output),
        AudioBufferRef::S24(input) => convert_samples(input, output),
        AudioBufferRef::S32(input) => convert_samples(input, output),
        AudioBufferRef::F32(input) => convert_samples(input, output),
        AudioBufferRef::F64(input) => convert_samples(input, output),
    }
}

fn convert_samples<S>(input: &AudioBuffer<S>, output: &mut [Vec<f32>])
where
    S: Sample + IntoSample<f32>,
{
    for (c, dst) in output.iter_mut().enumerate() {
        let src = input.chan(c);
        dst.extend(src.iter().map(|&s| s.into_sample()));
    }
}

fn cubic_interpolation(y0: f32, y1: f32, y2: f32, y3: f32, frac: f64) -> f32 {
    let a = y3 - y2 - y0 + y1;
    let b = y0 - y1 - a;
    let c = y2 - y0;
    let d = y1;

    let frac = frac as f32; // Convert fractional position to f32
    a * frac.powi(3) + b * frac.powi(2) + c * frac + d
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        (x * std::f64::consts::PI).sin() / (x * std::f64::consts::PI)
    }
}

fn windowed_sinc(x: f64, window_size: usize) -> f64 {
    let window_factor = 0.5 * (1.0 + (std::f64::consts::PI * x / window_size as f64).cos());
    sinc(x) * window_factor
}
