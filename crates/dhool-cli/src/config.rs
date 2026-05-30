use anyhow::Result;
use dhool_config::{BackendKind, Config, Overrides};
use dhool_cuda::device_info;

use crate::args::RenderArgs;

pub(crate) fn load_effective_config(args: &RenderArgs) -> Result<Config> {
    let mut config = Config::load(&args.config)?;
    config.apply_overrides(&Overrides {
        backend: args.backend,
        output_dir: args.output_dir.clone(),
        export: args.export,
        sequence_format: args.sequence_format,
        frames: args.frames,
        duration: args.duration,
        fps: args.fps,
        width: args.width,
        height: args.height,
        seed: args.seed,
        quality: args.quality,
    });
    config.validate()?;
    Ok(config)
}

pub(crate) fn ensure_cuda_backend(config: &Config, command: &str) -> Result<()> {
    anyhow::ensure!(
        config.backend.preferred == BackendKind::Cuda,
        "{command} requires --backend cuda; cpu-reference is only for setup validation tests"
    );
    let info = device_info();
    anyhow::ensure!(
        info.available,
        "CUDA is not available: {}",
        info.error
            .unwrap_or_else(|| "unknown CUDA initialization failure".to_string())
    );
    Ok(())
}
