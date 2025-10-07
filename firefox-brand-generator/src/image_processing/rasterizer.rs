use crate::config::FitStrategy;
use crate::error::{Error, Result};
use image::{DynamicImage, RgbaImage};
use resvg::usvg;

/// Rasterize an SVG to a specific size using a FitStrategy
pub fn rasterize_svg(
    svg_data: &[u8],
    target_width: u32,
    target_height: u32,
    fit: &FitStrategy,
) -> Result<DynamicImage> {
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &opts).map_err(|e| Error::Resvg(e.to_string()))?;

    let svg_size = tree.size();
    let svg_width = svg_size.width();
    let svg_height = svg_size.height();

    // Calculate the rendering dimensions and scale based on fit strategy
    let (render_width, render_height, scale_x, scale_y) = match fit {
        FitStrategy::Fill => {
            // Stretch to fill exact dimensions
            let scale_x = target_width as f32 / svg_width;
            let scale_y = target_height as f32 / svg_height;
            (target_width, target_height, scale_x, scale_y)
        }
        FitStrategy::Contain => {
            // Maintain aspect ratio, fit within bounds
            let scale_x = target_width as f32 / svg_width;
            let scale_y = target_height as f32 / svg_height;
            let scale = scale_x.min(scale_y);

            let render_width = (svg_width * scale).ceil() as u32;
            let render_height = (svg_height * scale).ceil() as u32;
            (render_width, render_height, scale, scale)
        }
        FitStrategy::ScaleDown => {
            // Only scale down if larger than target, otherwise keep original size
            if svg_width <= target_width as f32 && svg_height <= target_height as f32 {
                // SVG is smaller than target, keep original size
                (svg_width.ceil() as u32, svg_height.ceil() as u32, 1.0, 1.0)
            } else {
                // SVG is larger, apply contain strategy
                let scale_x = target_width as f32 / svg_width;
                let scale_y = target_height as f32 / svg_height;
                let scale = scale_x.min(scale_y);

                let render_width = (svg_width * scale).ceil() as u32;
                let render_height = (svg_height * scale).ceil() as u32;
                (render_width, render_height, scale, scale)
            }
        }
        FitStrategy::Cover => {
            // Maintain aspect ratio, fill entire bounds (may crop)
            let scale_x = target_width as f32 / svg_width;
            let scale_y = target_height as f32 / svg_height;
            let scale = scale_x.max(scale_y);

            let render_width = (svg_width * scale).ceil() as u32;
            let render_height = (svg_height * scale).ceil() as u32;
            (render_width, render_height, scale, scale)
        }
    };

    // Create pixmap for rendering
    let mut pixmap = tiny_skia::Pixmap::new(render_width, render_height)
        .ok_or_else(|| Error::Resvg("Failed to create pixmap".to_string()))?;

    // For non-uniform scaling (Fill), we need to apply different transforms
    let transform = if matches!(fit, FitStrategy::Fill) {
        tiny_skia::Transform::from_scale(scale_x, scale_y)
    } else {
        tiny_skia::Transform::from_scale(scale_x, scale_y)
    };

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
    let img = RgbaImage::from_raw(render_width, render_height, data)
        .ok_or_else(|| Error::Resvg("Failed to convert pixmap to image".to_string()))?;

    let mut result = DynamicImage::ImageRgba8(img);

    // For Cover strategy, we may need to crop the result to target dimensions
    if matches!(fit, FitStrategy::Cover)
        && (render_width > target_width || render_height > target_height)
    {
        let x_offset = (render_width - target_width) / 2;
        let y_offset = (render_height - target_height) / 2;
        result = result.crop_imm(x_offset, y_offset, target_width, target_height);
    }

    Ok(result)
}

/// Convenience function to rasterize SVG with Contain fit strategy (maintains aspect ratio)
pub fn rasterize_svg_contain(svg_data: &[u8], width: u32, height: u32) -> Result<DynamicImage> {
    rasterize_svg(svg_data, width, height, &FitStrategy::Contain)
}
