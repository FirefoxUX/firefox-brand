use crate::error::Result;
use crate::image_processing::{self, ImageSource};
use crate::platform::macos;
use crate::temp::TempDir;
use std::path::Path;

pub fn execute(input_path: &Path, output_path: &Path, sizes: &[u32]) -> Result<()> {
    // Create temporary directory for iconset
    let temp_dir = TempDir::new("firefox-brand-icns")?;
    let iconset_name = "icon.iconset";
    let iconset_path = temp_dir.create_dir(iconset_name)?;

    // Load the source image
    let img_source = image_processing::load(input_path)?;

    // Generate images for each size
    for &size in sizes {
        let img = match &img_source {
            ImageSource::Svg(svg_data) => image_processing::rasterize_svg(svg_data, size, size)?,
            ImageSource::Raster(img) => image_processing::resize(img, size, size)?,
        };

        // Save as icon_{size}x{size}.png
        let filename = format!("icon_{}x{}.png", size, size);
        let output = iconset_path.join(&filename);
        image_processing::save_png(&img, &output)?;

        // Also generate @2x version if this is a standard size
        if is_standard_retina_size(size) {
            let retina_size = size / 2;
            let retina_filename = format!("icon_{}x{}@2x.png", retina_size, retina_size);
            let retina_output = iconset_path.join(&retina_filename);
            image_processing::save_png(&img, &retina_output)?;
        }
    }

    // Run iconutil
    macos::run_iconutil(&iconset_path, output_path)?;

    Ok(())
}

fn is_standard_retina_size(size: u32) -> bool {
    matches!(size, 32 | 64 | 256 | 512 | 1024)
}
