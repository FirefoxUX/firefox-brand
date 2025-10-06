use crate::error::Result;
use crate::image_processing::{self, ImageSource};
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn execute(input_path: &Path, output_path: &Path, sizes: &[u32]) -> Result<()> {
    // Load the source image
    let img_source = image_processing::load(input_path)?;

    let mut icon_dir = IconDir::new(ResourceType::Icon);

    // Generate an image for each size
    for &size in sizes {
        let img = match &img_source {
            ImageSource::Svg(svg_data) => image_processing::rasterize_svg(svg_data, size, size)?,
            ImageSource::Raster(img) => image_processing::resize(img, size, size)?,
        };

        // Convert to RGBA8
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Create IconImage
        let icon_image = IconImage::from_rgba_data(width, height, rgba.into_raw());

        // Add to icon directory
        let entry = IconDirEntry::encode(&icon_image)?;
        icon_dir.add_entry(entry);
    }

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write the ICO file
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    icon_dir.write(&mut writer)?;

    Ok(())
}
