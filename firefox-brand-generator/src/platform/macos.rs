use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run iconutil to convert an iconset to icns
/// iconset_path should end with .iconset
pub fn run_iconutil(iconset_path: &Path, output_path: &Path) -> Result<()> {
    let status = Command::new("iconutil")
        .arg("-c")
        .arg("icns")
        .arg(iconset_path)
        .arg("-o")
        .arg(output_path)
        .status()?;

    if !status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "iconutil".to_string(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

/// Run actool to compile Assets.xcassets
/// Returns the path to the generated Assets.car file
pub fn run_actool(xcassets_path: &Path, icon_path: &Path, output_dir: &Path) -> Result<PathBuf> {
    use crate::temp::TempDir;
    use std::fs;

    // Create a temporary directory for actool output
    let temp_dir = TempDir::new("actool-output")?;
    let temp_output_dir = temp_dir.path();

    let status = Command::new("actool")
        .arg(xcassets_path)
        .arg(icon_path)
        .arg("--compile")
        .arg(temp_output_dir)
        .arg("--target-device")
        .arg("mac")
        .arg("--platform")
        .arg("macosx")
        .arg("--minimum-deployment-target")
        .arg("26.0")
        .arg("--enable-on-demand-resources")
        .arg("NO")
        .arg("--app-icon")
        .arg("AppIcon")
        .arg("--output-partial-info-plist")
        .arg(temp_output_dir.join("partial-info.plist"))
        .status()?;

    if !status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "actool".to_string(),
            code: status.code().unwrap_or(-1),
        });
    }

    // Check if Assets.car was generated
    let assets_car_path = temp_output_dir.join("Assets.car");
    if !assets_car_path.exists() {
        return Err(Error::Transformation(
            "actool did not generate Assets.car file".to_string(),
        ));
    }

    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Copy the Assets.car file to the requested output directory
    let output_assets_car = output_dir.join("Assets.car");
    fs::copy(&assets_car_path, &output_assets_car)?;

    Ok(output_assets_car)
}

/// Run sips to set DPI on an image
pub fn run_sips_set_dpi(image_path: &Path, dpi: f64) -> Result<()> {
    let status = Command::new("sips")
        .arg("-s")
        .arg("dpiHeight")
        .arg(dpi.to_string())
        .arg("-s")
        .arg("dpiWidth")
        .arg(dpi.to_string())
        .arg(image_path)
        .status()?;

    if !status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "sips".to_string(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

/// Mount a DMG file and return the mount point
pub fn mount_dmg(dmg_path: &Path) -> Result<PathBuf> {
    let output = Command::new("hdiutil")
        .arg("mount")
        .arg(dmg_path)
        .arg("-readonly")
        .output()?;

    if !output.status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "hdiutil mount".to_string(),
            code: output.status.code().unwrap_or(-1),
        });
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Parse the mount point from hdiutil output
    // Output format is typically: /dev/disk2s1        Apple_HFS   /Volumes/Firefox
    // or with spaces: /dev/disk2s1        Apple_HFS   /Volumes/Firefox Developer Edition
    for line in output_str.lines() {
        // Find the position of "/Volumes/" in the line
        if let Some(volumes_start) = line.find("/Volumes/") {
            // Extract everything from "/Volumes/" to the end of the line
            let mount_point = line[volumes_start..].trim();
            return Ok(PathBuf::from(mount_point));
        }
    }

    Err(Error::Transformation(
        "Could not parse mount point from hdiutil output".to_string(),
    ))
}

/// Unmount a volume
pub fn unmount_volume(mount_point: &Path) -> Result<()> {
    let status = Command::new("hdiutil")
        .arg("unmount")
        .arg(mount_point)
        .status()?;

    if !status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "hdiutil unmount".to_string(),
            code: status.code().unwrap_or(-1),
        });
    }

    Ok(())
}
