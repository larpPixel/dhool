use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

use anyhow::{Context, Result};
use dhool_config::ExportConfig;

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
