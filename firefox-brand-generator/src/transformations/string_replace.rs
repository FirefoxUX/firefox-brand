use crate::config::BrandConfig;
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;

pub fn execute(
    input_path: &Path,
    output_path: &Path,
    search_value: &str,
    replace_key: &str,
    brand_config: &BrandConfig,
) -> Result<()> {
    // Read the input file
    let contents = fs::read_to_string(input_path)?;

    // Get the replacement value from brand config
    let replace_value = brand_config
        .strings
        .get(replace_key)
        .ok_or_else(|| Error::MissingBrandConfigValue(replace_key.to_string()))?;

    // Perform the replacement
    let replaced = contents.replace(search_value, replace_value);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the output file
    fs::write(output_path, replaced)?;

    Ok(())
}
