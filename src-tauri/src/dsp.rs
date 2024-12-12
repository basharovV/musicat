pub fn calculate_rms(samples: &[f32]) -> f32 {
    let sum_of_squares: f32 = samples.iter().map(|&sample| sample * sample).sum();
    let mean_of_squares = sum_of_squares / samples.len() as f32;
    mean_of_squares.sqrt()
}
