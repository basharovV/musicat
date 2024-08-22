pub fn calculate_db(rms: f32) -> f32 {
    // Ensure the RMS is greater than a very small value to avoid log(0)
    if rms <= 0.0 {
        f32::NEG_INFINITY // Return negative infinity for non-positive RMS
    } else {
        20.0 * rms.log10()
    }
}

pub fn calculate_rms(samples: &[f32]) -> f32 {
    let sum_of_squares: f32 = samples.iter().map(|&sample| sample * sample).sum();
    let mean_of_squares = sum_of_squares / samples.len() as f32;
    mean_of_squares.sqrt()
}
