use crate::error::{Error, Result};
use crate::platform::macos;
use crate::temp::TempDir;
use crate::transformations::icns;
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute(
    output_path: &Path,
    app_name: &str,
    volume_name: &str,
    background_image_path: &Path,
    volume_icon_path: &Path,
    window_position: &str,
    window_size: &str,
    app_icon_position: &str,
    app_drop_link_position: &str,
) -> Result<()> {
    // Split the space-separated values (string substitution is done in mod.rs)
    let window_pos_parts: Vec<&str> = window_position.split_whitespace().collect();
    let window_size_parts: Vec<&str> = window_size.split_whitespace().collect();
    let app_icon_pos_parts: Vec<&str> = app_icon_position.split_whitespace().collect();
    let app_drop_link_pos_parts: Vec<&str> = app_drop_link_position.split_whitespace().collect();

    // Validate that we have the expected number of parts
    if window_pos_parts.len() != 2 {
        return Err(Error::Transformation(format!(
            "Invalid window position format: '{}'. Expected 'x y'",
            window_position
        )));
    }
    if window_size_parts.len() != 2 {
        return Err(Error::Transformation(format!(
            "Invalid window size format: '{}'. Expected 'width height'",
            window_size
        )));
    }
    if app_icon_pos_parts.len() != 2 {
        return Err(Error::Transformation(format!(
            "Invalid app icon position format: '{}'. Expected 'x y'",
            app_icon_position
        )));
    }
    if app_drop_link_pos_parts.len() != 2 {
        return Err(Error::Transformation(format!(
            "Invalid app drop link position format: '{}'. Expected 'x y'",
            app_drop_link_position
        )));
    }

    // Create temporary directory structure
    let temp_dir = TempDir::new("firefox-brand-dsstore")?;
    let src_dir = temp_dir.create_dir("src")?;
    let app_dir = src_dir.join(app_name);
    fs::create_dir_all(&app_dir)?;

    // Copy background image to temp directory and set DPI
    let background_dest = temp_dir.path().join("background.png");
    fs::copy(background_image_path, &background_dest)?;
    macos::run_sips_set_dpi(&background_dest, 144.0)?;

    // Convert volume icon from PNG to ICNS
    let volume_icon_icns = temp_dir.path().join("disk.icns");

    // Use standard icon sizes for disk icons
    let icon_sizes = vec![16, 32, 128, 256, 512];
    icns::execute(volume_icon_path, &volume_icon_icns, &icon_sizes)?;

    // Get the path to create-dmg script
    let create_dmg_script = Path::new("external/create-dmg/create-dmg");
    if !create_dmg_script.exists() {
        return Err(Error::FileNotFound(create_dmg_script.to_path_buf()));
    }

    // Change to temp directory to run create-dmg
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Build create-dmg command
    let dmg_name = "firefox-installer.dmg";
    let status = Command::new(original_dir.join(create_dmg_script))
        .arg("--volname")
        .arg(volume_name)
        .arg("--volicon")
        .arg("disk.icns")
        .arg("--background")
        .arg("background.png")
        .arg("--window-pos")
        .arg(window_pos_parts[0])
        .arg(window_pos_parts[1])
        .arg("--window-size")
        .arg(window_size_parts[0])
        .arg(window_size_parts[1])
        .arg("--icon-size")
        .arg("128")
        .arg("--text-size")
        .arg("12")
        .arg("--icon")
        .arg(app_name)
        .arg(app_icon_pos_parts[0])
        .arg(app_icon_pos_parts[1])
        .arg("--app-drop-link")
        .arg(app_drop_link_pos_parts[0])
        .arg(app_drop_link_pos_parts[1])
        .arg("--app-drop-link-name")
        .arg(" ")
        .arg("--hide-extension")
        .arg(app_name)
        .arg("--no-internet-enable")
        .arg(dmg_name)
        .arg("src/")
        .status();

    // Restore original directory
    std::env::set_current_dir(&original_dir)?;

    let status = status?;
    if !status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "create-dmg".to_string(),
            code: status.code().unwrap_or(-1),
        });
    }

    // Mount the DMG
    let dmg_path = temp_dir.path().join(dmg_name);
    let mount_point = macos::mount_dmg(&dmg_path)?;

    // Extract .DS_Store file
    let ds_store_source = mount_point.join(".DS_Store");
    if !ds_store_source.exists() {
        macos::unmount_volume(&mount_point)?;
        return Err(Error::Transformation(
            ".DS_Store file not found in generated DMG".to_string(),
        ));
    }

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy .DS_Store to destination
    fs::copy(&ds_store_source, output_path)?;

    // Make the copied file visible (remove the hidden flag)
    let status = Command::new("chflags")
        .arg("nohidden")
        .arg(output_path)
        .status()?;

    if !status.success() {
        println!(
            "{} Failed to remove hidden flag from .DS_Store file at {}",
            "[Warning]".black().on_yellow(),
            output_path.display()
        );
    }

    // Verify the file exists and is accessible
    if !output_path.exists() {
        return Err(Error::Transformation(
            "Failed to copy .DS_Store file to output path".to_string(),
        ));
    }

    // Unmount the DMG
    macos::unmount_volume(&mount_point)?;

    Ok(())
}
