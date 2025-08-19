#[derive(Debug, Clone, Copy)]
pub enum PeakMethod {
    Rms,
    Max,
    MinMax,
    Peak,
}
pub fn calculate_rms(samples: &[f32]) -> f32 {
    let sum_of_squares: f32 = samples.iter().map(|&sample| sample * sample).sum();
    let mean_of_squares = sum_of_squares / samples.len() as f32;
    mean_of_squares.sqrt()
}

pub fn calculate_max(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
}

pub fn calculate_peak(samples: &[f32]) -> f32 {
    let max_val = samples.iter().map(|&x| x.abs()).fold(0.0, f32::max);

    // Apply slight smoothing to reduce noise while preserving dynamics
    max_val * 0.9 + calculate_rms(samples) * 0.1
}

pub fn calculate_min_max(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let (min_val, max_val) = samples
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), &sample| {
            (min.min(sample), max.max(sample))
        });

    // Return the range (max - min) or just max absolute
    (max_val - min_val).max(max_val.abs()).max(min_val.abs())
}

pub fn calculate_peak_value(samples: &[f32], method: PeakMethod) -> f32 {
    match method {
        PeakMethod::Rms => calculate_rms(samples),
        PeakMethod::Max => calculate_max(samples),
        PeakMethod::MinMax => calculate_min_max(samples),
        PeakMethod::Peak => calculate_peak(samples),
    }
}

pub fn stereo_to_mono(samples: &[f32]) -> Vec<f32> {
    samples
        .chunks_exact(2)
        .map(|chunk| (chunk[0] + chunk[1]) * 0.5)
        .collect()
}

pub fn process_samples(samples: &[f32], channels: usize, force_mono: bool) -> Vec<f32> {
    match (channels, force_mono) {
        (1, _) => samples.to_vec(),
        (2, true) => stereo_to_mono(samples),
        (2, false) => samples.to_vec(),
        (n, _) => {
            eprintln!("Warning: {} channels detected, treating as stereo", n);
            samples.to_vec()
        }
    }
}
