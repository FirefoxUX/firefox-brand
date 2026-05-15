use crate::error::Result;
use crate::platform::macos;
use std::fs;
use std::path::Path;

/// Execute CopyImageMac transformation: copy a file and optionally set DPI using sips
pub fn execute(input_path: &Path, output_path: &Path, dpi: Option<f64>) -> Result<()> {
    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy the file first
    fs::copy(input_path, output_path)?;

    // If DPI is specified, apply it using sips
    if let Some(dpi_value) = dpi {
        macos::run_sips_set_dpi(output_path, dpi_value)?;
    }

    Ok(())
}