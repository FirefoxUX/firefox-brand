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
    validate_only: bool,
) -> Result<()> {
    // Detect platform capabilities
    let capabilities = PlatformCapabilities::detect();

    // In validate mode, ignore the host's platform tooling — we want to
    // validate Mac-only transformations even on Linux CI.
    if !validate_only {
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
    }

    // Merge brand name into filter options so `only` fields in config.json are respected.
    // In validate mode, force MacMode::All so Mac-specific transformations are still inspected.
    let effective_filter = FilterOptions {
        only_types: filter_options.only_types.clone(),
        mac_mode: if validate_only {
            crate::generator::filter::MacMode::All
        } else {
            filter_options.mac_mode
        },
        brand_name: brand_config.env.get("name").cloned(),
    };

    // Filter transformations
    let filtered = filter_transformations(&config.transformations, &effective_filter, &capabilities);

    // Create transformation context
    let ctx = TransformationContext {
        source_dir: paths.source_dir,
        static_dir: paths.static_dir,
        output_dir: paths.output_dir,
        brand_config,
        capabilities: &capabilities,
    };

    // Execute (or validate) each transformation
    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    let action_verb = if validate_only { "Validating" } else { "Processing" };

    for (transformation, should_warn) in filtered {
        let t_type = transformation.transformation_type();
        let output = transformation.output_path();

        // Only honour missing-tool warnings when actually executing.
        if should_warn && !validate_only {
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
            action_verb.dimmed(),
            t_type.bold(),
            "->".dimmed(),
            output
        );

        let result = if validate_only {
            transformations::validate(&transformation, &ctx)
        } else {
            transformations::execute(&transformation, &ctx)
        };

        match result {
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

    let success_label = if validate_only { "Valid:   " } else { "Success: " };

    println!();
    println!("{}", "Summary:".bold().underline());
    println!("  {}{}", success_label, success_count.to_string());
    println!("  Skipped: {}", skip_count.to_string());
    println!("  Errors:  {}", error_count.to_string());

    if error_count > 0 {
        let msg = if validate_only {
            format!("{} transformation(s) failed validation", error_count)
        } else {
            format!("{} transformation(s) failed", error_count)
        };
        Err(crate::error::Error::Transformation(msg))
    } else {
        Ok(())
    }
}
