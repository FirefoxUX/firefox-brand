use crate::config::{FitStrategy, OutputFileType};
use crate::error::Result;
use crate::image_processing::{self, ImageSource};
use image::GenericImageView;
use std::path::Path;

pub fn execute(
    input_path: &Path,
    output_path: &Path,
    output_file_type: &OutputFileType,
    width: u32,
    height: u32,
    padding_width: Option<u32>,
    padding_height: Option<u32>,
    offset_x: Option<i32>,
    offset_y: Option<i32>,
    fit: &FitStrategy,
) -> Result<()> {
    // Calculate the content area dimensions based on padding
    let pad_w = padding_width.unwrap_or(0);
    let pad_h = padding_height.unwrap_or(0);

    // Calculate the content dimensions (target minus padding)
    let content_width = width.saturating_sub(pad_w * 2);
    let content_height = height.saturating_sub(pad_h * 2);

    // Load the image (SVG or raster)
    let img_source = image_processing::load(input_path)?;

    // Convert to raster if needed, using the content dimensions
    let img = match img_source {
        ImageSource::Svg(svg_data) => {
            // For SVG, we rasterize to the content size first
            image_processing::rasterize_svg(&svg_data, content_width, content_height)?
        }
        ImageSource::Raster(img) => {
            // For raster images, we need to handle different fit strategies
            match fit {
                FitStrategy::Fill => {
                    // Stretch to exact dimensions
                    image_processing::resize(&img, content_width, content_height)?
                }
                FitStrategy::Contain | FitStrategy::ScaleDown => {
                    // Maintain aspect ratio
                    let (img_w, img_h) = img.dimensions();

                    // Only scale down if larger than target (for ScaleDown)
                    if matches!(fit, FitStrategy::ScaleDown)
                        && img_w <= content_width
                        && img_h <= content_height
                    {
                        img.clone()
                    } else {
                        // Calculate scaling factors to fit within content area
                        let scale_x = content_width as f64 / img_w as f64;
                        let scale_y = content_height as f64 / img_h as f64;
                        let scale = scale_x.min(scale_y);

                        let new_width = (img_w as f64 * scale) as u32;
                        let new_height = (img_h as f64 * scale) as u32;

                        img.resize_exact(
                            new_width,
                            new_height,
                            image::imageops::FilterType::Lanczos3,
                        )
                    }
                }
                FitStrategy::Cover => {
                    // Fill bounds, maintaining aspect ratio
                    let (img_w, img_h) = img.dimensions();
                    let scale_x = content_width as f64 / img_w as f64;
                    let scale_y = content_height as f64 / img_h as f64;
                    let scale = scale_x.max(scale_y);

                    let new_width = (img_w as f64 * scale) as u32;
                    let new_height = (img_h as f64 * scale) as u32;

                    let resized = img.resize_exact(
                        new_width,
                        new_height,
                        image::imageops::FilterType::Lanczos3,
                    );

                    // Crop to exact content dimensions
                    let x_offset = (new_width - content_width) / 2;
                    let y_offset = (new_height - content_height) / 2;

                    resized.crop_imm(x_offset, y_offset, content_width, content_height)
                }
            }
        }
    };

    // Create the final image with padding
    let processed = if pad_w == 0 && pad_h == 0 {
        // No padding needed, use the image as is
        img
    } else {
        // Create an image with padding
        let mut canvas = image::RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 0]));

        // Calculate position to center the image
        let mut x_offset = (width - img.width()) / 2;
        let mut y_offset = (height - img.height()) / 2;

        // Apply user-defined offsets if provided
        if let Some(offset_x_val) = offset_x {
            // Convert to i64 to handle potential negative values
            let x_pos = x_offset as i64 + offset_x_val as i64;
            x_offset = x_pos.clamp(-(img.width() as i64), width as i64) as u32;
        }

        if let Some(offset_y_val) = offset_y {
            // Convert to i64 to handle potential negative values
            let y_pos = y_offset as i64 + offset_y_val as i64;
            y_offset = y_pos.clamp(-(img.height() as i64), height as i64) as u32;
        }

        // Overlay the image onto the canvas
        image::imageops::overlay(
            &mut canvas,
            &img.to_rgba8(),
            x_offset.into(),
            y_offset.into(),
        );

        image::DynamicImage::ImageRgba8(canvas)
    };

    // Save the result
    image_processing::save(&processed, output_path, output_file_type)?;

    Ok(())
}
