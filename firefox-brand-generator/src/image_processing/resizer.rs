use crate::config::FitStrategy;
use crate::error::Result;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

/// Resize an image to target dimensions with optional padding
pub fn resize_with_padding(
    img: &DynamicImage,
    target_width: u32,
    target_height: u32,
    padding_width: Option<u32>,
    padding_height: Option<u32>,
    fit: &FitStrategy,
) -> Result<DynamicImage> {
    let pad_w = padding_width.unwrap_or(0);
    let pad_h = padding_height.unwrap_or(0);

    // Calculate the content area (target minus padding)
    let content_width = target_width.saturating_sub(pad_w * 2);
    let content_height = target_height.saturating_sub(pad_h * 2);

    // Resize the image to fit in the content area
    let resized = resize_to_fit(img, content_width, content_height, fit)?;

    // If no padding, return the resized image
    if pad_w == 0 && pad_h == 0 {
        return Ok(resized);
    }

    // Create a new image with the target dimensions and transparent background
    let mut canvas = RgbaImage::from_pixel(target_width, target_height, Rgba([0, 0, 0, 0]));

    // Calculate position to center the resized image
    let x_offset = (target_width - resized.width()) / 2;
    let y_offset = (target_height - resized.height()) / 2;

    // Overlay the resized image onto the canvas
    image::imageops::overlay(
        &mut canvas,
        &resized.to_rgba8(),
        x_offset.into(),
        y_offset.into(),
    );

    Ok(DynamicImage::ImageRgba8(canvas))
}

fn resize_to_fit(
    img: &DynamicImage,
    width: u32,
    height: u32,
    fit: &FitStrategy,
) -> Result<DynamicImage> {
    let (img_w, img_h) = img.dimensions();

    let resized = match fit {
        FitStrategy::Fill => {
            // Stretch to exact dimensions
            img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
        }
        FitStrategy::Contain => {
            // Fit within bounds, maintaining aspect ratio
            img.resize(width, height, image::imageops::FilterType::Lanczos3)
        }
        FitStrategy::Cover => {
            // Fill bounds, maintaining aspect ratio, crop if necessary
            let scale_x = width as f64 / img_w as f64;
            let scale_y = height as f64 / img_h as f64;
            let scale = scale_x.max(scale_y);

            let new_width = (img_w as f64 * scale) as u32;
            let new_height = (img_h as f64 * scale) as u32;

            let resized =
                img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

            // Crop to exact dimensions
            let x_offset = (new_width - width) / 2;
            let y_offset = (new_height - height) / 2;

            resized.crop_imm(x_offset, y_offset, width, height)
        }
        FitStrategy::ScaleDown => {
            // Only scale down if larger than target, maintain aspect ratio
            if img_w <= width && img_h <= height {
                img.clone()
            } else {
                img.resize(width, height, image::imageops::FilterType::Lanczos3)
            }
        }
    };

    Ok(resized)
}

/// Simple resize without padding (used by ico, icns, assets_car)
pub fn resize(img: &DynamicImage, width: u32, height: u32) -> Result<DynamicImage> {
    Ok(img.resize_exact(width, height, image::imageops::FilterType::Lanczos3))
}
