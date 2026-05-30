use thiserror::Error;

#[derive(Debug, Error)]
pub enum CudaBackendError {
    #[error("CUDA backend is required; cpu-reference is not a production render backend")]
    CpuReferenceRequested,
    #[error("CUDA failed: {0}")]
    Cuda(String),
}
