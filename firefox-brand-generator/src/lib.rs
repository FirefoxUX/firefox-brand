pub mod config;
pub mod error;
pub mod generator;
pub mod image_processing;
pub mod platform;
pub mod temp;
pub mod transformations;
pub mod utils;

pub use config::{BrandConfig, Config, load_brand_config, load_config};
pub use error::{Error, Result};
pub use generator::{FilterOptions, GeneratorPaths, MacMode, generate};
pub use platform::is_macos;

use std::path::Path;

/// Main entry point for the library
pub fn run(
    config_path: &Path,
    source_dir: &Path,
    static_dir: &Path,
    output_dir: &Path,
    filter_options: FilterOptions,
) -> Result<()> {
    // Load configuration files
    let config = load_config(config_path)?;
    let brand_config = load_brand_config(source_dir)?;

    // Set up paths
    let paths = GeneratorPaths {
        source_dir,
        static_dir,
        output_dir,
    };

    // Run the generator
    generate(&config, &brand_config, &paths, &filter_options)?;

    Ok(())
}
