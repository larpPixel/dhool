use eframe::egui::{self, Vec2};

pub(super) fn scaled_dimension(base: u32, scale: f32) -> u32 {
    ((base as f32 * scale).round() as u32).clamp(16, 8192)
}

pub(super) fn fit_size(content: Vec2, bounds: Vec2) -> Vec2 {
    if content.x <= 0.0 || content.y <= 0.0 || bounds.x <= 0.0 || bounds.y <= 0.0 {
        return Vec2::ZERO;
    }
    let scale = (bounds.x / content.x).min(bounds.y / content.y);
    content * scale
}

pub(super) fn color_image_from_opaque_rgba(size: [usize; 2], rgba: Vec<u8>) -> egui::ColorImage {
    let expected_bytes = size[0] * size[1] * 4;
    assert_eq!(expected_bytes, rgba.len());

    if std::mem::size_of::<egui::Color32>() != 4
        || std::mem::align_of::<egui::Color32>() != 1
        || rgba.capacity() % 4 != 0
    {
        return egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    }

    let mut rgba = std::mem::ManuallyDrop::new(rgba);
    let pixel_count = expected_bytes / 4;
    let pixel_capacity = rgba.capacity() / 4;
    // CUDA writes opaque RGBA8, which is already valid `Color32` storage.
    let pixels = unsafe {
        Vec::from_raw_parts(
            rgba.as_mut_ptr().cast::<egui::Color32>(),
            pixel_count,
            pixel_capacity,
        )
    };
    egui::ColorImage { size, pixels }
}
