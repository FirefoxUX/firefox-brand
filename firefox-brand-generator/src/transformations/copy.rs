use crate::error::Result;
use std::fs;
use std::path::Path;

pub fn execute(input_path: &Path, output_path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(input_path, output_path)?;

    Ok(())
}
