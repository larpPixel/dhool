use std::{fs, io::BufWriter, path::Path};

use anyhow::{Context, Result};

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
