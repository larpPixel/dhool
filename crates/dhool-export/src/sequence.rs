use std::path::{Path, PathBuf};

use anyhow::Result;
use dhool_config::SequenceFormat;

use crate::write_png;

pub fn write_sequence_frame(
    sequence_dir: impl AsRef<Path>,
    frame_index: u32,
    format: SequenceFormat,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<PathBuf> {
    anyhow::ensure!(
        format == SequenceFormat::Png,
        "only PNG sequence export is implemented"
    );
    let path = sequence_dir
        .as_ref()
        .join(format!("frame_{frame_index:06}.png"));
    write_png(&path, width, height, rgba)?;
    Ok(path)
}
