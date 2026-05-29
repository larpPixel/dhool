use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

use anyhow::{Context, Result};
use dhool_config::{ExportConfig, ExportMode, SequenceFormat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedOutputs {
    pub output_dir: PathBuf,
    pub sequence_dir: Option<PathBuf>,
    pub video_path: Option<PathBuf>,
}

pub fn plan_outputs(export: &ExportConfig, stem: &str, include_sequence: bool) -> PlannedOutputs {
    let output_dir = PathBuf::from(&export.output_dir);
    let writes_sequence =
        include_sequence && matches!(export.mode, ExportMode::Sequence | ExportMode::Both);
    let writes_video = matches!(export.mode, ExportMode::Pipe | ExportMode::Both);
    PlannedOutputs {
        output_dir: output_dir.clone(),
        sequence_dir: writes_sequence.then(|| output_dir.join("frames")),
        video_path: writes_video.then(|| output_dir.join(format!("{stem}.mp4"))),
    }
}

pub fn write_png(path: impl AsRef<Path>, width: u32, height: u32, rgba: &[u8]) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let expected = width as usize * height as usize * 4;
    anyhow::ensure!(
        rgba.len() == expected,
        "RGBA buffer length mismatch: got {}, expected {}",
        rgba.len(),
        expected
    );
    let file =
        fs::File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    let writer = BufWriter::new(file);
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        writer,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
    );
    image::ImageEncoder::write_image(
        encoder,
        rgba,
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )
    .with_context(|| format!("failed to encode PNG {}", path.display()))
}

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

pub struct FfmpegPipe {
    child: Child,
    stdin: Option<ChildStdin>,
    path: PathBuf,
}

impl FfmpegPipe {
    pub fn start(
        path: impl AsRef<Path>,
        width: u32,
        height: u32,
        fps: f32,
        export: &ExportConfig,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let size = format!("{width}x{height}");
        let fps_string = format!("{fps:.6}");
        let crf = export.crf.to_string();
        let mut child = Command::new("ffmpeg")
            .arg("-y")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("rgba")
            .arg("-s")
            .arg(&size)
            .arg("-r")
            .arg(&fps_string)
            .arg("-i")
            .arg("-")
            .arg("-an")
            .arg("-c:v")
            .arg(&export.video_codec)
            .arg("-preset")
            .arg(&export.preset)
            .arg("-crf")
            .arg(&crf)
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg(&path)
            .stdin(Stdio::piped())
            .spawn()
            .with_context(|| "failed to start ffmpeg")?;
        let stdin = child.stdin.take().context("ffmpeg stdin was not piped")?;
        Ok(Self {
            child,
            stdin: Some(stdin),
            path,
        })
    }

    pub fn write_frame(&mut self, rgba: &[u8]) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .context("ffmpeg stdin is already closed")?;
        stdin.write_all(rgba).with_context(|| {
            format!(
                "failed to write raw frame to ffmpeg for {}",
                self.path.display()
            )
        })
    }

    pub fn finish(mut self) -> Result<PathBuf> {
        drop(self.stdin.take());
        let status = self.child.wait().context("failed to wait for ffmpeg")?;
        anyhow::ensure!(status.success(), "ffmpeg exited with status {status}");
        Ok(self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_sequence_and_video_paths() {
        let export = ExportConfig {
            mode: ExportMode::Both,
            sequence_format: SequenceFormat::Png,
            video_codec: "libx264".to_string(),
            crf: 18,
            preset: "medium".to_string(),
            output_dir: "outputs/test".to_string(),
        };
        let plan = plan_outputs(&export, "preview", true);
        assert_eq!(
            plan.sequence_dir.unwrap(),
            PathBuf::from("outputs/test/frames")
        );
        assert_eq!(
            plan.video_path.unwrap(),
            PathBuf::from("outputs/test/preview.mp4")
        );
    }
}
