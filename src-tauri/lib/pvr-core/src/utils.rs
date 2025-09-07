use ndarray::Array1;

pub fn hann_window(window_length: usize, periodic: bool) -> Array1<f64> {
  if periodic {
    return hann_window_periodic(window_length);
  }

  if window_length == 0 {
    return Array1::zeros(0);
  }

  if window_length == 1 {
    return Array1::ones(1);
  }

  let half_length = (window_length + 1) / 2;
  let scaling = (std::f64::consts::PI * 2.0) / (window_length - 1) as f64;

  let mut res = Array1::zeros(window_length);

  for i in 0..half_length {
    let cur = 0.5 - 0.5 * (scaling * i as f64).cos();
    res[i] = cur;
    res[window_length - i - 1] = cur;
  }

  res
}

fn hann_window_periodic(window_length: usize) -> Array1<f64> {
  if window_length == 0 {
    return Array1::zeros(0);
  }

  if window_length == 1 {
    return Array1::ones(1);
  }

  let half_length = window_length / 2 + 1;
  let scaling = (std::f64::consts::PI * 2.0) / window_length as f64;

  let mut res = Array1::zeros(window_length);

  for i in 1..half_length {
    let cur = 0.5 - 0.5 * (scaling * i as f64).cos();
    res[i] = cur;
    res[window_length - i] = cur;
  }

  res
}
