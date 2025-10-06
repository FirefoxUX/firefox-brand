use crate::config::OutputFileType;
use crate::error::Result;
use image::{DynamicImage, ImageFormat};
use std::path::Path;

/// Save an image to a file with the specified format
pub fn save(img: &DynamicImage, path: &Path, format: &OutputFileType) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let image_format = match format {
        OutputFileType::Png => ImageFormat::Png,
        OutputFileType::Jpg => ImageFormat::Jpeg,
        OutputFileType::Bmp => ImageFormat::Bmp,
        OutputFileType::Tiff => ImageFormat::Tiff,
        OutputFileType::Gif => ImageFormat::Gif,
    };

    img.save_with_format(path, image_format)?;

    Ok(())
}

/// Save as PNG (convenience function for ico, icns, assets_car)
pub fn save_png(img: &DynamicImage, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    img.save_with_format(path, ImageFormat::Png)?;

    Ok(())
}
