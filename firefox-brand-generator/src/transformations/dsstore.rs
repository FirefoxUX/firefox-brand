use crate::config::types::BrandConfig;
use crate::error::{Error, Result};
use crate::platform::macos;
use crate::temp::TempDir;
use crate::transformations::icns;
use crate::utils::string_processing;
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
    brand_config: &BrandConfig,
) -> Result<()> {
    // Process string replacements in app_name and volume_name
    let processed_app_name =
        string_processing::process_string_replacements(app_name, brand_config)?;
    let processed_volume_name =
        string_processing::process_string_replacements(volume_name, brand_config)?;

    // Create temporary directory structure
    let temp_dir = TempDir::new("firefox-brand-dsstore")?;
    let src_dir = temp_dir.create_dir("src")?;
    let app_dir = src_dir.join(&processed_app_name);
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
        .arg(&processed_volume_name)
        .arg("--volicon")
        .arg("disk.icns")
        .arg("--background")
        .arg("background.png")
        .arg("--window-pos")
        .arg("200")
        .arg("120")
        .arg("--window-size")
        .arg("680")
        .arg("400")
        .arg("--icon-size")
        .arg("128")
        .arg("--text-size")
        .arg("12")
        .arg("--icon")
        .arg(&processed_app_name)
        .arg("209")
        .arg("220")
        .arg("--app-drop-link")
        .arg("472")
        .arg("220")
        .arg("--app-drop-link-name")
        .arg(" ")
        .arg("--hide-extension")
        .arg(&processed_app_name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_dsstore_string_processing() {
        // Set up brand config with test values
        let mut strings = HashMap::new();
        strings.insert("appName".to_string(), "Firefox".to_string());
        strings.insert("shortAppName".to_string(), "FF".to_string());

        let brand_config = BrandConfig {
            strings,
            env: HashMap::new(),
        };

        // Test app name with extension
        let app_name = "{{#str appName}}.app";
        let expected_app_name = "Firefox.app";

        let result =
            string_processing::process_string_replacements(app_name, &brand_config).unwrap();
        assert_eq!(result, expected_app_name);

        // Test volume name
        let volume_name = "{{#str shortAppName}}";
        let expected_volume_name = "FF";

        let result =
            string_processing::process_string_replacements(volume_name, &brand_config).unwrap();
        assert_eq!(result, expected_volume_name);

        // Test complex app name
        let complex_app_name = "{{#str appName}} Installer.app";
        let expected_complex_app_name = "Firefox Installer.app";

        let result =
            string_processing::process_string_replacements(complex_app_name, &brand_config)
                .unwrap();
        assert_eq!(result, expected_complex_app_name);
    }
}
