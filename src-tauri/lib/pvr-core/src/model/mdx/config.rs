use std::{env, fmt, path::PathBuf};

use anyhow::{Context, Result};
use ort::{session::builder::GraphOptimizationLevel, session::Session};

use super::{MdxSeperator, Stft};

pub enum MdxType {
    Vocals,
    Instrumental,
    Reverb,
}

impl fmt::Display for MdxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MdxType::Vocals => write!(f, "Vocals"),
            MdxType::Instrumental => write!(f, "Instrumental"),
            MdxType::Reverb => write!(f, "Reverb"),
        }
    }
}

impl MdxType {
    pub fn get_primary_stem(&self) -> &'static str {
        match self {
            MdxType::Vocals => "vocal",
            MdxType::Instrumental => "inst",
            MdxType::Reverb => "reverb",
        }
    }

    pub fn get_secondary_stem(&self) -> &'static str {
        match self {
            MdxType::Vocals => "inst",
            MdxType::Instrumental => "vocal",
            MdxType::Reverb => "no_reverb",
        }
    }
}

pub struct MdxConfig {
    pub name: &'static str,
    pub filename: &'static str,
    pub model_type: MdxType,
    n_fft: usize,
    dim_t: u8,
    dim_f: usize,
    compensate: f64,
}

impl MdxConfig {
    pub const fn new(
        name: &'static str,
        filename: &'static str,
        model_type: MdxType,
        n_fft: usize,
        dim_t: u8,
        dim_f: usize,
        compensate: f64,
    ) -> Self {
        Self {
            name,
            filename,
            model_type,
            n_fft,
            dim_t,
            dim_f,
            compensate,
        }
    }

    fn model_path(&self, models_dir: &Option<PathBuf>) -> PathBuf {
        if let Some(model_path) = models_dir {
            return model_path.join(self.filename);
        }
        env::var("PVR_MODELS")
            .map(|s| PathBuf::from(s))
            .unwrap_or_else(|_| {
                env::current_exe()
                    .expect("Failed to get exe path")
                    .parent()
                    .expect("Failed to get the parent path of exe")
                    .join("models")
            })
            .join(self.filename)
    }

    pub fn exists(&self, model_path: &Option<PathBuf>) -> bool {
        self.model_path(model_path).exists()
    }

    pub fn build(&self, model_dir: &Option<PathBuf>) -> Result<MdxSeperator> {
        tracing::info!(
          name = self.name,
          r#type = %self.model_type,
          "Building model..."
        );

        let model = Session::builder()
            .context("Failed to get ort session builder")?
            .with_intra_threads(4)
            .context("Failed to set intra threads")?
            .with_inter_threads(4)
            .context("Failed to set inter threads")?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .context("Failed to optimize ort session")?
            .commit_from_file(self.model_path(model_dir))
            .context("Failed to load onnx model")?;

        let stft = Stft::new(self.n_fft, 1024, self.dim_f);

        Ok(MdxSeperator {
            n_fft: self.n_fft,
            segment_size: 1 << self.dim_t, // TODO: support other segment size
            stft,
            model,
            compensate: self.compensate,
        })
    }
}
