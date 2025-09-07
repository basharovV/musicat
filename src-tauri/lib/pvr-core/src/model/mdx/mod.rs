mod config;
mod preset;
mod stft;

use std::ops::{AddAssign, MulAssign};

use anyhow::Result;
use ndarray::{concatenate, prelude::*, StrideShape};
use ort::{session::Session, tensor::Shape, value::TensorRef};

use crate::utils::hann_window;
pub use config::{MdxConfig, MdxType};
pub use preset::MDX_PRESETS;
use stft::Stft;

pub struct MdxSeperator {
    n_fft: usize,
    segment_size: usize,
    stft: Stft,
    model: Session,
    compensate: f64,
}

impl MdxSeperator {
    pub fn demix(&mut self, mix: ArrayView2<f64>) -> Result<Array2<f64>> {
        tracing::info!("Start seperating...");

        let (_, length) = mix.dim();

        let trim = self.n_fft / 2;
        let chunk_size = 1024 * (self.segment_size - 1);
        let gen_size = chunk_size - 2 * trim;
        let pad = gen_size + trim - (length % gen_size);

        let mixture = concatenate(
            Axis(1),
            &[
                Array2::zeros((2, trim)).view(),
                mix.view(),
                Array2::zeros((2, pad)).view(),
            ],
        )?;

        let step = chunk_size - self.n_fft;

        let new_len = trim + length + pad;

        let mut result: Array3<f64> = Array3::zeros((1, 2, new_len));
        let mut divider: Array3<f64> = Array3::zeros((1, 2, new_len));

        let total_chunks = (new_len - 1) / step + 1;

        for i in (0..new_len).step_by(step) {
            let start = i;
            let end = (i + chunk_size).min(new_len);

            let actual_size = end - start;
            let cur_chunk = start / step + 1;

            tracing::info!(
                "{:.2}% Processing... ({cur_chunk}/{total_chunks})",
                cur_chunk as f64 * 100.0 / total_chunks as f64
            );

            let window = {
                let window = hann_window(actual_size, false);

                let mut res = Array3::zeros((1, 2, actual_size));
                res.slice_mut(s![.., 0, ..]).assign(&window);
                res.slice_mut(s![.., 1, ..]).assign(&window);
                res
            };

            let mut mix_part = mixture.slice(s![.., start..end]).to_owned();

            if end != i + chunk_size {
                let pad_size = i + chunk_size - end;
                mix_part.append(Axis(1), Array2::zeros((2, pad_size)).view())?;
            }

            let mut tar_waves = self.run_model(mix_part.insert_axis(Axis(0)).view())?;

            tar_waves
                .slice_mut(s![.., .., ..actual_size])
                .mul_assign(&window);

            divider
                .slice_mut(s![.., .., start..end])
                .add_assign(&window);

            result
                .slice_mut(s![.., .., start..end])
                .add_assign(&tar_waves.slice(s![.., .., ..actual_size]));
        }

        let tar_waves = result / divider;
        let tar_waves = tar_waves.remove_axis(Axis(0));

        let right = (new_len - trim).min(trim + length);
        let tar_waves = tar_waves.slice(s![.., trim..right]);

        Ok(tar_waves.mapv(|x| x * self.compensate))
    }

    fn run_model(&mut self, mix: ArrayView3<f64>) -> Result<Array3<f64>> {
        // Apply STFT
        let mut spek = self.stft.apply(mix)?;
        // Apply mask
        spek.slice_mut(s![.., .., ..3, ..]).fill(0.0);

        // Convert to f32
        let spek_f32 = spek.mapv(|x| x as f32).view().to_owned();

        // Option 1: Use from_array_view with a reference (recommended)
        let input_tensor = TensorRef::from_array_view(&spek_f32)?;

        let spec_pred = self.model.run(ort::inputs![input_tensor])?;
        let (shape, data) = spec_pred["output"].try_extract_tensor::<f32>()?;

        let spec_pred = MdxSeperator::to_array3_f64(shape, data)?;

        Ok(self.stft.inverse(spec_pred.view())?)
    }

    pub fn to_array3_f64(shape: &Shape, data: &[f32]) -> Result<Array4<f64>> {
        // Convert Shape to Vec<usize>
        let dims: Vec<usize> = shape.to_vec().iter().map(|d| *d as usize).collect();

        if dims.len() != 4 {
            anyhow::bail!("Expected 4D tensor, got {}D", dims.len());
        }

        let arr = Array4::from_shape_vec((dims[0], dims[1], dims[2], dims[3]), data.to_vec())?;

        let arr = arr.mapv(|x| x as f64);

        // Create Array3 (owned)
        Ok(arr)
    }
}
