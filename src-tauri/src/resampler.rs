// Symphonia
// Copyright (c) 2019-2022 The Project Symphonia Developers.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::info;
use rubato::{calculate_cutoff, SincInterpolationParameters};
use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Signal, SignalSpec};
use symphonia::core::conv::{FromSample, IntoSample};
use symphonia::core::sample::Sample;

pub struct Resampler<T> {
    resampler: rubato::FftFixedIn<f32>,
    input: Vec<Vec<f32>>,
    output: Vec<Vec<f32>>,
    interleaved: Vec<T>,
    duration: usize,
}

impl<T> Resampler<T>
where
    T: Sample + FromSample<f32> + IntoSample<f32>,
{
    fn resample_inner(&mut self) -> &[T] {
        let mut output: Vec<Vec<f32>> = Default::default();

        {
            let mut input: arrayvec::ArrayVec<&[f32], 32> = Default::default();

            for channel in self.input.iter() {
                input.push(&channel[..self.duration]);
            }

            // Resample.
            output = rubato::Resampler::process(&mut self.resampler, &input, None).unwrap();
        }

        // Remove consumed samples from the input buffer.
        for channel in self.input.iter_mut() {
            channel.drain(0..self.duration);
        }

        // Interleave the planar samples from Rubato.
        let num_channels = output.len();

        self.interleaved
            .resize(num_channels * output[0].len(), T::MID);

        for (i, frame) in self.interleaved.chunks_exact_mut(num_channels).enumerate() {
            for (ch, s) in frame.iter_mut().enumerate() {
                *s = output[ch][i].into_sample();
            }
        }

        &self.interleaved
    }
}

impl<T> Resampler<T>
where
    T: Sample + FromSample<f32> + IntoSample<f32>,
{
    pub fn new(spec: SignalSpec, to_sample_rate: usize, duration: u64) -> Self {
        let duration = duration as usize;
        let num_channels = spec.channels.count();

        let resampler = rubato::FftFixedIn::<f32>::new(
            spec.rate as usize,
            to_sample_rate,
            duration,
            1,
            num_channels,
        )
        .unwrap();

        // let sinc_len = duration;
        // let oversampling_factor = 128;
        // let interpolation = rubato::SincInterpolationType::Linear;
        // let window = rubato::WindowFunction::Blackman2;

        // let f_cutoff = 0.95;
        // let params = SincInterpolationParameters {
        //     sinc_len,
        //     f_cutoff,
        //     interpolation,
        //     oversampling_factor,
        //     window,
        // };
        // let ratio = spec.rate as f64 / to_sample_rate as f64;
        // let resampler =
        //     rubato::FastFixedIn::<f32>::new(ratio, 1.0f64, rubato::PolynomialDegree::Cubic, 1, num_channels).unwrap();

        let output = rubato::Resampler::output_buffer_allocate(&resampler, true);

        let input = vec![Vec::with_capacity(duration); num_channels];

        Self {
            resampler,
            input,
            output,
            duration,
            interleaved: Default::default(),
        }
    }

    /// Resamples a planar/non-interleaved input.
    ///
    /// Returns the resampled samples in an interleaved format.
    pub fn resample(&mut self, input: AudioBufferRef<'_>) -> Option<&[T]> {
        // Copy and convert samples into input buffer.
        convert_samples_any(&input, &mut self.input);

        // Check if more samples are required.
        if self.input[0].len() < self.duration {
            return None;
        }

        Some(self.resample_inner())
    }

    /// Resample any remaining samples in the resample buffer.
    pub fn flush(&mut self) -> Option<&[T]> {
        let len = self.input[0].len();

        info!("Flushing resampler, len: {}", len);
        if len == 0 {
            return None;
        }

        for channel in self.input.iter_mut() {
            channel.drain(0..len);
            // channel.resize(len + (self.duration - partial_len), f32::MID);
        }

        None
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
