use crate::config::types::{BrandConfig, Config};
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;

pub fn load_config(config_path: &Path) -> Result<Config> {
    if !config_path.exists() {
        return Err(Error::FileNotFound(config_path.to_path_buf()));
    }

    let contents = fs::read_to_string(config_path)?;
    let config: Config = serde_json::from_str(&contents)?;

    Ok(config)
}

pub fn load_brand_config(brand_config_path: &Path) -> Result<BrandConfig> {
    if !brand_config_path.exists() {
        // Brand config is optional
        return Ok(BrandConfig::default());
    }

    let contents = fs::read_to_string(brand_config_path)?;
    let brand_config: BrandConfig = serde_json::from_str(&contents)?;

    Ok(brand_config)
}
