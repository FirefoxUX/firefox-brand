use crate::error::{Error, Result};
use image::DynamicImage;
use std::path::Path;

pub enum ImageSource {
    Svg(Vec<u8>),
    Raster(DynamicImage),
}

/// Load an image from a file, detecting whether it's SVG or raster
pub fn load(path: &Path) -> Result<ImageSource> {
    if !path.exists() {
        return Err(Error::FileNotFound(path.to_path_buf()));
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("svg") => {
            let svg_data = std::fs::read(path)?;
            Ok(ImageSource::Svg(svg_data))
        }
        Some("png") | Some("jpg") | Some("jpeg") | Some("bmp") | Some("gif") | Some("tiff") => {
            let img = image::open(path)?;
            Ok(ImageSource::Raster(img))
        }
        _ => Err(Error::InvalidFileType {
            expected: "svg, png, jpg, bmp, gif, or tiff".to_string(),
            actual: extension.unwrap_or_else(|| "unknown".to_string()),
        }),
    }
}
