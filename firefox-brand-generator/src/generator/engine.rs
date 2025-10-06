use crate::config::{BrandConfig, Config};
use crate::error::Result;
use crate::generator::filter::{FilterOptions, filter_transformations};
use crate::platform::PlatformCapabilities;
use crate::transformations::{self, TransformationContext};
use owo_colors::OwoColorize;
use std::path::Path;

pub struct GeneratorPaths<'a> {
    pub source_dir: &'a Path,
    pub static_dir: &'a Path,
    pub output_dir: &'a Path,
}

pub fn generate(
    config: &Config,
    brand_config: &BrandConfig,
    paths: &GeneratorPaths,
    filter_options: &FilterOptions,
) -> Result<()> {
    // Detect platform capabilities
    let capabilities = PlatformCapabilities::detect();

    // Warn about missing tools
    if !capabilities.has_iconutil {
        eprintln!(
            "{} {} {}",
            "Warning:".yellow().bold(),
            "iconutil".cyan(),
            "not found. ICNS generation will be skipped.".dimmed()
        );
    }
    if !capabilities.has_actool {
        eprintln!(
            "{} {} {}",
            "Warning:".yellow().bold(),
            "actool".cyan(),
            "not found. Assets.car generation will be skipped.".dimmed()
        );
    }

    // Filter transformations
    let filtered = filter_transformations(&config.transformations, filter_options, &capabilities);

    // Create transformation context
    let ctx = TransformationContext {
        source_dir: paths.source_dir,
        static_dir: paths.static_dir,
        output_dir: paths.output_dir,
        brand_config,
        capabilities: &capabilities,
    };

    // Execute each transformation
    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for (transformation, should_warn) in filtered {
        let t_type = transformation.transformation_type();
        let output = transformation.output_path();

        if should_warn {
            eprintln!(
                "{} {} transformation for '{}': {}",
                "Skipping".yellow(),
                t_type.cyan().bold(),
                output.yellow(),
                "required tool not available".dimmed()
            );
            skip_count += 1;
            continue;
        }

        // Check if we should skip based on filter
        if let Some(ref only_types) = filter_options.only_types {
            if !only_types.contains(t_type) {
                skip_count += 1;
                continue;
            }
        }

        print!(
            "{} {} {} {}... ",
            "Processing".dimmed(),
            t_type.bold(),
            "->".dimmed(),
            output
        );

        match transformations::execute(&transformation, &ctx) {
            Ok(_) => {
                println!("{}", "✓".green().bold());
                success_count += 1;
            }
            Err(e) => {
                println!("{}", "✗".red().bold());
                eprintln!("  {}: {}", "Error".red().bold(), e.to_string());
                error_count += 1;
            }
        }
    }

    println!();
    println!("{}", "Summary:".bold().underline());
    println!("  Success: {}", success_count.to_string());
    println!("  Skipped: {}", skip_count.to_string());
    println!("  Errors:  {}", error_count.to_string());

    if error_count > 0 {
        Err(crate::error::Error::Transformation(format!(
            "{} transformation(s) failed",
            error_count
        )))
    } else {
        Ok(())
    }
}
