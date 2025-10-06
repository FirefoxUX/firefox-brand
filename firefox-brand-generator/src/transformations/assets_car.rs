use crate::error::Result;
use crate::image_processing::{self, ImageSource};
use crate::platform::{check, macos};
use crate::temp::TempDir;
use serde_json::json;
use std::fs;
use std::path::Path;

pub fn execute(
    icon_path: &Path,
    output_path: &Path,
    app_icon_input: &Path,
    icon_input: &Path,
) -> Result<()> {
    // Check actool version compatibility for .icon files
    let capabilities = check::PlatformCapabilities::detect();
    capabilities.validate_actool_for_icon_support()?;

    // Create temporary directory structure
    let temp_dir = TempDir::new("firefox-brand-assets")?;
    let xcassets_path = temp_dir.create_dir("Assets.xcassets")?;

    // Copy icon directory to temp directory and rename to AppIcon.icon
    let temp_icon_path = temp_dir.path().join("AppIcon.icon");
    copy_dir_all(icon_path, &temp_icon_path)?;

    // Generate AppIcon.appiconset
    generate_app_icon_set(&xcassets_path, app_icon_input)?;

    // Generate Icon.iconset
    generate_icon_set(&xcassets_path, icon_input)?;

    // Generate root Contents.json
    generate_root_contents_json(&xcassets_path)?;

    // Run actool
    let output_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(output_dir)?;

    // Generate Assets.car in output_dir and get the path to it
    let generated_car = macos::run_actool(&xcassets_path, &temp_icon_path, output_dir)?;

    // Move the generated Assets.car to the exact output location if needed
    if generated_car != output_path {
        fs::rename(generated_car, output_path)?;
    }

    Ok(())
}

fn generate_app_icon_set(xcassets_path: &Path, input: &Path) -> Result<()> {
    let appiconset_path = xcassets_path.join("AppIcon.appiconset");
    fs::create_dir_all(&appiconset_path)?;

    // Load source image
    let img_source = image_processing::load(input)?;

    // Sizes required for AppIcon
    let sizes = [
        (16, 1),
        (16, 2),
        (32, 1),
        (32, 2),
        (128, 1),
        (128, 2),
        (256, 1),
        (256, 2),
        (512, 1),
        (512, 2),
    ];

    let mut images_json = Vec::new();

    for (size, scale) in sizes {
        let actual_size = size * scale;

        let img = match &img_source {
            ImageSource::Svg(svg_data) => {
                image_processing::rasterize_svg(svg_data, actual_size, actual_size)?
            }
            ImageSource::Raster(img) => image_processing::resize(img, actual_size, actual_size)?,
        };

        let filename = if scale == 1 {
            format!("icon_{}x{}.png", size, size)
        } else {
            format!("icon_{}x{}@{}x.png", size, size, scale)
        };

        let output = appiconset_path.join(&filename);
        image_processing::save_png(&img, &output)?;

        images_json.push(json!({
            "filename": filename,
            "idiom": "mac",
            "scale": format!("{}x", scale),
            "size": format!("{}x{}", size, size)
        }));
    }

    // Generate Contents.json
    let contents = json!({
        "images": images_json,
        "info": {
            "author": "xcode",
            "version": 1
        }
    });

    let contents_path = appiconset_path.join("Contents.json");
    fs::write(contents_path, serde_json::to_string_pretty(&contents)?)?;

    Ok(())
}

fn generate_icon_set(xcassets_path: &Path, input: &Path) -> Result<()> {
    let iconset_path = xcassets_path.join("Icon.iconset");
    fs::create_dir_all(&iconset_path)?;

    // Load source image
    let img_source = image_processing::load(input)?;

    // Sizes for Icon.iconset
    let sizes = [(256, 1), (256, 2)];

    for (size, scale) in sizes {
        let actual_size = size * scale;

        let img = match &img_source {
            ImageSource::Svg(svg_data) => {
                image_processing::rasterize_svg(svg_data, actual_size, actual_size)?
            }
            ImageSource::Raster(img) => image_processing::resize(img, actual_size, actual_size)?,
        };

        let filename = if scale == 1 {
            format!("icon_{}x{}.png", size, size)
        } else {
            format!("icon_{}x{}@{}x.png", size, size, scale)
        };

        let output = iconset_path.join(&filename);
        image_processing::save_png(&img, &output)?;
    }

    Ok(())
}

fn generate_root_contents_json(xcassets_path: &Path) -> Result<()> {
    let contents = json!({
        "info": {
            "author": "xcode",
            "version": 1
        }
    });

    let contents_path = xcassets_path.join("Contents.json");
    fs::write(contents_path, serde_json::to_string_pretty(&contents)?)?;

    Ok(())
}

/// Recursively copy a directory and all its contents
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
