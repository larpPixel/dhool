use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use dhool_config::{BackendKind, ExportMode, Quality, SequenceFormat};

#[derive(Debug, Parser)]
#[command(name = "dhool", about = "Rust/CUDA cinematic background renderer")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    DeviceInfo,
    Validate(RenderArgs),
    Still(RenderArgs),
    Preview(RenderArgs),
    Final(RenderArgs),
    Benchmark(RenderArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RenderArgs {
    pub(crate) config: PathBuf,
    #[arg(long, value_enum)]
    pub(crate) backend: Option<BackendKind>,
    #[arg(long)]
    pub(crate) output_dir: Option<String>,
    #[arg(long, value_enum)]
    pub(crate) export: Option<ExportMode>,
    #[arg(long, value_enum)]
    pub(crate) sequence_format: Option<SequenceFormat>,
    #[arg(long)]
    pub(crate) frames: Option<u32>,
    #[arg(long)]
    pub(crate) duration: Option<f32>,
    #[arg(long)]
    pub(crate) fps: Option<f32>,
    #[arg(long)]
    pub(crate) width: Option<u32>,
    #[arg(long)]
    pub(crate) height: Option<u32>,
    #[arg(long)]
    pub(crate) seed: Option<u64>,
    #[arg(long, value_enum)]
    pub(crate) quality: Option<Quality>,
}
