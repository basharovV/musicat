use symphonia::core::{audio::SignalSpec, conv::IntoSample};

use crate::{constants::BUFFER_SIZE, output::cpal::AudioOutputSample};

// Add this inside the 'mod cpal' block or just above it
pub struct BiquadFilter {
    // Coefficients
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    // Delay lines for each channel (assuming Stereo max for simplicity)
    z1: [f32; 2],
    z2: [f32; 2],
}

impl BiquadFilter {
    fn new() -> Self {
        Self {
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            z1: [0.0; 2],
            z2: [0.0; 2],
        }
    }

    // Standard Cookbook formulas for Peaking EQ
    fn update_peaking(&mut self, sample_rate: f32, freq: f32, gain_db: f32, q: f32) {
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let alpha = omega.sin() / (2.0 * q);
        let cos_w = omega.cos();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha / a;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    #[inline(always)]
    fn process(&mut self, sample: f32, ch: usize) -> f32 {
        let out = self.b0 * sample + self.z1[ch];
        self.z1[ch] = self.b1 * sample - self.a1 * out + self.z2[ch];
        self.z2[ch] = self.b2 * sample - self.a2 * out;
        out
    }
}

pub struct Equalizer<T: AudioOutputSample> {
    pub filters: Vec<BiquadFilter>,
    num_channels: usize,
    sample_rate: f32,
    // Pre-allocated buffer to avoid allocations in the hot path
    internal_buf: Vec<T>,
}

impl<T> Equalizer<T>
where
    T: AudioOutputSample + cpal::FromSample<T> + IntoSample<f32>,
{
    pub fn new(spec: SignalSpec, bands: &[(f32, f32, f32)]) -> Self {
        let num_channels = spec.channels.count();
        let sample_rate = spec.rate as f32;

        let mut filters = Vec::new();
        for &(freq, gain, q) in bands {
            let mut filter = BiquadFilter::new();
            filter.update_peaking(sample_rate, freq, gain, q);
            filters.push(filter);
        }

        Self {
            filters,
            num_channels,
            sample_rate,
            internal_buf: Vec::with_capacity(BUFFER_SIZE as usize),
        }
    }

    pub fn update_band(&mut self, index: usize, freq: f32, gain: f32, q: f32) {
        if let Some(filter) = self.filters.get_mut(index) {
            filter.update_peaking(self.sample_rate, freq, gain, q);
        }
    }

    /// Processes samples in-place to minimize memory movement
    pub fn process(&mut self, samples: &mut [T]) {
        for (i, sample) in samples.iter_mut().enumerate() {
            let ch = i % self.num_channels;

            // We only process first 2 channels for standard EQ
            if ch < 2 {
                let mut val_f32: f32 = (*sample).into_sample();

                for filter in &mut self.filters {
                    val_f32 = filter.process(val_f32, ch);
                }

                *sample = <T as symphonia::core::conv::FromSample<f32>>::from_sample(val_f32);
            }
        }
    }
}
