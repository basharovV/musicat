use crate::dsp::PeakMethod;

/// Buffer size in seconds
pub const BUFFER_SIZE: f64 = 1.0;
pub const WAVEFORM_WINDOW_SIZE: usize = 8192;
pub const WAVEFORM_PEAK_METHOD: PeakMethod = PeakMethod::Rms;
