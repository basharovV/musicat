// src/lib.rs

mod cli;
mod setup;
mod util; // existing // existing // existing

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use ndarray::{Array2, ArrayView2};
pub use pvr_core::config::Backend;

pub use pvr_core::mdx::MDX_PRESETS;
pub use tracing::Level;
use tracing_log::LogTracer;

pub use util::AudioFormat;

/// Options for a separation call
#[derive(Clone, Debug)]
pub struct SeparateOptions {
    /// Path to models directory.
    pub models_dir: Option<PathBuf>,
    /// Model name in `MDX_PRESETS`. If `None`, error with available models.
    pub model: Option<String>,
    /// Preferred ONNXRuntime execution providers, in priority order.
    pub backends: Vec<Backend>,
    /// Path to ONNX Runtime lib (libonnxruntime.so, libonnxruntime.dylib, dll).
    pub onnx_runtime_path: Option<PathBuf>,
    /// Output container when writing to files.
    pub output_format: AudioFormat,
}

impl Default for SeparateOptions {
    fn default() -> Self {
        Self {
            model: None,
            backends: vec![Backend::CPU],
            output_format: AudioFormat::Flac,
            models_dir: None,
            onnx_runtime_path: None,
        }
    }
}

/// Initialize tracing (idempotent). Useful for host apps that want logs.
pub fn init_tracing(level: Level) {
    let _ = LogTracer::init();
}

/// Configure ONNX Runtime execution providers for pvr_core.
pub fn init_backends(backends: &[Backend], onnxruntime_path: &Option<PathBuf>) -> Result<()> {
    tracing::info!("Initializing ONNX Runtime with backends: {backends:?}");
    if backends.is_empty() {
        tracing::warn!("No backend specified, defaulting to CPU");
        pvr_core::config::setup_backends(vec![Backend::CPU], onnxruntime_path)
            .map_err(|e| anyhow!("Init ORT failed: {e}"))
    } else {
        pvr_core::config::setup_backends(backends.to_vec(), onnxruntime_path)
            .map_err(|e| anyhow!("Init ORT failed: {e}"))
    }
}

/// List available models (only those whose weights are present).
pub fn available_models(models_dir: &Option<PathBuf>) -> Vec<(usize, String)> {
    MDX_PRESETS
        .iter()
        .enumerate()
        .filter(|(_, p)| p.exists(models_dir))
        .map(|(i, p)| (i, format!("{} ({})", p.name, p.model_type)))
        .collect()
}

/// High-level: file → files. Returns the written primary/secondary file paths.
pub fn separate_to_files(
    input: &Path,
    output_dir: &Path,
    opts: &SeparateOptions,
) -> Result<(PathBuf, PathBuf)> {
    tracing::info!("Options: ");
    tracing::info!(
        "• models dir: {}",
        opts.models_dir.as_ref().unwrap().display()
    );
    tracing::info!("• model: {}", opts.model.as_ref().unwrap());
    tracing::info!(
        "• backends: {:?}",
        opts.backends
            .iter()
            .map(|b| format!("{:?}", b))
            .collect::<Vec<_>>()
            .join(", ")
    );
    tracing::info!(
        "• output format: {}",
        opts.output_format.extension().to_string()
    );
    tracing::info!("• input file: {}", input.display());
    tracing::info!("• output dir: {}", output_dir.display());

    // tracing + backends (idempotent if host called earlier)
    init_backends(&opts.backends, &opts.onnx_runtime_path)?;

    // Resolve model
    let model_name = opts.model.clone().ok_or_else(|| {
        anyhow!("preset is None; call available_models() or set SeparateOptions.preset")
    })?;
    let preset = MDX_PRESETS
        .iter()
        .find(|p| p.filename == model_name)
        .ok_or_else(|| anyhow!("invalid model name: {model_name}"))?;

    let mut mdx = preset
        .build(&opts.models_dir)
        .map_err(|e| anyhow!("build model failed: {e}"))?;

    // IO
    if !input.is_file() {
        return Err(anyhow!("input path is not a regular file"));
    }

    let mix = util::read_audio(input)?; // (channels, frames) f64 @ 44.1k
    let res = mdx
        .demix(mix.view())
        .map_err(|e| anyhow!("inference failed: {e}"))?; // target

    tracing::info!("Separation complete");
    // filenames
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("invalid file stem"))?;
    let primary_name = format!(
        "{}_{}.{}",
        stem,
        preset.model_type.get_primary_stem(),
        opts.output_format.extension()
    );
    let secondary_name = format!(
        "{}_{}.{}",
        stem,
        preset.model_type.get_secondary_stem(),
        opts.output_format.extension()
    );

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).unwrap();
    }

    // write
    let primary_path = output_dir.join(primary_name);
    util::write_audio(primary_path.as_path(), res.view(), &opts.output_format)?;

    let secondary = &mix - &res; // accompaniment (or the other stem)
    let secondary_path = output_dir.join(secondary_name);
    util::write_audio(
        secondary_path.as_path(),
        secondary.view(),
        &opts.output_format,
    )?;

    Ok((primary_path, secondary_path))
}
