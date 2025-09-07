mod cli; // existing
mod setup;

use std::path::{Path, PathBuf};

use crate::cli::Cli;
use clap::Parser;
use pvr; // the lib
use pvr::AudioFormat;
use setup::setup_tracing;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() {
    let args = Cli::parse();

    setup_tracing();

    let mut backends = Vec::new();
    #[cfg(target_os = "windows")]
    if args.directml_backend {
        backends.push(pvr_core::config::Backend::DirectML);
    }
    #[cfg(target_os = "macos")]
    if args.coreml_backend {
        backends.push(pvr_core::config::Backend::CoreML);
    }
    if args.cuda_backend {
        backends.push(pvr_core::config::Backend::CUDA);
    }
    if args.tensorrt_backend {
        backends.push(pvr_core::config::Backend::TensorRT);
    }

    backends.push(pvr_core::config::Backend::CPU);

    let output_format = match args.format.as_str() {
        "wav" => AudioFormat::Wav,
        "flac" => AudioFormat::Flac,
        other => {
            tracing::error!(format = other, "Unknown output audio format");
            return;
        }
    };

    let executable_path = std::env::current_exe().unwrap();
    let executable_dir = executable_path.parent().unwrap();

    let models_dir = args.models_dir.unwrap_or(executable_dir.join("models"));
    if !models_dir.exists() {
        tracing::error!("Failed to find models directory: {}. If your models are elsewhere, specify the directory with --models-dir", models_dir.display());
        return;
    }

    // Mandatory (dynamic linking)
    let onnx_runtime_path = args.onnx_runtime_path;
    if !onnx_runtime_path.exists() {
        tracing::error!(
            "Failed to find onnxruntime: {}",
            onnx_runtime_path.display()
        );
        return;
    }

    let opts = pvr::SeparateOptions {
        model: args.model.clone(),
        backends,
        output_format,
        models_dir: Some(models_dir),
        onnx_runtime_path: Some(onnx_runtime_path),
    };

    // Show models if none selected (behavior preserved)
    if args.model.is_none() {
        tracing::info!("Please specify the model you wish to use");
        tracing::info!("All available models:");
        for (id, name) in pvr::available_models(&opts.models_dir) {
            tracing::info!("{id}. {name}");
        }
        return;
    }

    let input: PathBuf = args.input_path;
    let outdir: PathBuf = args.output_path;

    match pvr::separate_to_files(&input, &outdir, &opts) {
        Ok((p1, p2)) => {
            tracing::info!(primary = %p1.display(), secondary = %p2.display(), "Separation complete");
        }
        Err(err) => {
            tracing::error!(%err, "Separation failed");
        }
    }
}
