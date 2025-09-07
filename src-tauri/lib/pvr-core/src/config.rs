use std::path::PathBuf;

use anyhow::Result;
use ort::execution_providers::{
    CPUExecutionProvider, CUDAExecutionProvider, CoreMLExecutionProvider,
    DirectMLExecutionProvider, ExecutionProviderDispatch, TensorRTExecutionProvider,
};

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    DirectML,
    CoreML,
    CUDA,
    TensorRT,
    CPU,
}

impl Backend {
    pub fn to_ep(&self) -> ExecutionProviderDispatch {
        match self {
            Self::DirectML => DirectMLExecutionProvider::default().build(),
            Self::CoreML => CoreMLExecutionProvider::default().build(),
            Self::CUDA => CUDAExecutionProvider::default().build(),
            Self::TensorRT => TensorRTExecutionProvider::default().build(),
            Self::CPU => CPUExecutionProvider::default().build(),
        }
    }
}

pub fn setup_backends(
    backends: impl AsRef<[Backend]>,
    onnx_runtime_path: &Option<PathBuf>,
) -> Result<()> {
    let backends: Vec<_> = backends.as_ref().iter().map(|b| b.to_ep()).collect();
    if let Some(path) = onnx_runtime_path {
        ort::init_from(path.display())
            .with_execution_providers(backends)
            .commit();
    } else {
        ort::init().with_execution_providers(backends).commit();
    }
    Ok(())
}
