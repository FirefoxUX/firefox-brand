use crate::error::{Error, Result};
use image::{DynamicImage, RgbaImage};
use resvg::usvg;

/// Rasterize an SVG to a specific size
pub fn rasterize_svg(svg_data: &[u8], width: u32, height: u32) -> Result<DynamicImage> {
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &opts).map_err(|e| Error::Resvg(e.to_string()))?;

    let size = tree.size();
    let scale_x = width as f32 / size.width();
    let scale_y = height as f32 / size.height();
    let scale = scale_x.min(scale_y);

    let scaled_width = (size.width() * scale).ceil() as u32;
    let scaled_height = (size.height() * scale).ceil() as u32;

    let mut pixmap = tiny_skia::Pixmap::new(scaled_width, scaled_height)
        .ok_or_else(|| Error::Resvg("Failed to create pixmap".to_string()))?;

    let transform = tiny_skia::Transform::from_scale(scale, scale);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert from premultiplied to straight alpha
    let mut data = pixmap.data().to_vec();
    for pixel in data.chunks_exact_mut(4) {
        let a = pixel[3] as f32 / 255.0;
        if a > 0.0 {
            pixel[0] = (pixel[0] as f32 / a).min(255.0) as u8;
            pixel[1] = (pixel[1] as f32 / a).min(255.0) as u8;
            pixel[2] = (pixel[2] as f32 / a).min(255.0) as u8;
        }
    }

    // Convert pixmap to image
    let img = RgbaImage::from_raw(scaled_width, scaled_height, data)
        .ok_or_else(|| Error::Resvg("Failed to convert pixmap to image".to_string()))?;

    Ok(DynamicImage::ImageRgba8(img))
}
