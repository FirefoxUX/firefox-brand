use clap::{Parser, ValueEnum};
use firefox_brand_generator::{FilterOptions, MacMode, is_macos, run};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Debug, Clone, ValueEnum)]
enum MacModeArg {
    None,
    Simple,
    All,
}

impl From<MacModeArg> for MacMode {
    fn from(mode: MacModeArg) -> Self {
        match mode {
            MacModeArg::None => MacMode::None,
            MacModeArg::Simple => MacMode::Simple,
            MacModeArg::All => MacMode::All,
        }
    }
}

#[derive(Parser)]
#[command(
    name = "firefox-brand-generator",
    about = "Generate Firefox brand assets from source files",
    version
)]
struct Cli {
    /// Brand to build (e.g. official, nightly, aurora, unofficial).
    /// Omit to build every brand found under <ROOT>/brands/
    brand: Option<String>,

    /// Repo root. Auto-detected by walking up from the current directory
    /// to find a folder containing config.json and brands/
    #[arg(long, value_name = "DIR")]
    root: Option<PathBuf>,

    /// Output parent directory. Each brand is written to <OUTPUT>/<BRAND>/.
    /// Defaults to <ROOT>/dist
    #[arg(short, long, value_name = "DIR")]
    output: Option<PathBuf>,

    /// Comma-separated list of transformation types to run. When specified, --mac is ignored.
    /// Available types: raster, ico, icns, assets-car, copy, copy-preprocess, copy-image-mac, ds-store
    #[arg(long, value_name = "TYPES", value_delimiter = ',')]
    only: Option<Vec<String>>,

    /// Control macOS-specific transformations. Ignored if --only is used.
    /// Options: none (skip ds-store, icns, assets-car, copy-image-mac),
    /// simple (run icns, assets-car, copy-image-mac only), all (run all).
    /// Defaults to simple on macOS and none elsewhere.
    #[arg(long, value_enum, value_name = "MODE")]
    mac: Option<MacModeArg>,

    /// Validate config and brand assets without producing any output.
    /// Skips platform tool checks and all filesystem writes — safe to run on Linux CI.
    #[arg(long, conflicts_with_all = ["only", "mac", "output"])]
    validate: bool,
}

fn make_filter_options(only: Option<Vec<String>>, mac: Option<MacModeArg>) -> FilterOptions {
    let filter_options = if let Some(types) = only {
        FilterOptions::new().with_types(types)
    } else {
        FilterOptions::new()
    };

    let mac_mode = if let Some(mac_mode) = mac {
        mac_mode.into()
    } else if filter_options.only_types.is_some() {
        MacMode::All
    } else if is_macos() {
        println!(
            "{} Auto-detected macOS: enabling {} (icns + assets-car + copy-image-mac transformations)",
            "[Info]".on_blue().bold(),
            "simple Mac mode".bold()
        );
        println!("       To run all Mac-specific transformations, use the --mac all option.");
        MacMode::Simple
    } else {
        println!(
            "{} Non-macOS platform detected: disabling Mac-specific transformations (ds-store, icns, assets-car, copy-image-mac)",
            "[Info]".on_blue().bold()
        );
        MacMode::None
    };

    filter_options.with_mac_mode(mac_mode)
}

fn find_repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("config.json").exists() && dir.join("brands").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn discover_brands(brands_dir: &Path) -> Vec<String> {
    let mut brands: Vec<String> = std::fs::read_dir(brands_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                entry.file_name().into_string().ok()
            } else {
                None
            }
        })
        .collect();
    brands.sort();
    brands
}

fn main() {
    let cli = Cli::parse();

    let root = if let Some(r) = cli.root {
        if !r.join("config.json").exists() || !r.join("brands").is_dir() {
            eprintln!(
                "{} '{}' is not a valid repo root (missing config.json or brands/)",
                "Error:".red().bold(),
                r.display().to_string().yellow()
            );
            process::exit(1);
        }
        r
    } else {
        match find_repo_root() {
            Some(r) => r,
            None => {
                eprintln!(
                    "{} Could not find repo root (no config.json + brands/ directory found in current directory or any parent).",
                    "Error:".red().bold()
                );
                eprintln!(
                    "       Run from within the repo or use {} to specify the root.",
                    "--root <DIR>".cyan()
                );
                process::exit(1);
            }
        }
    };

    let brands_dir = root.join("brands");
    let available_brands = discover_brands(&brands_dir);

    let brands_to_build: Vec<String> = if let Some(brand) = cli.brand {
        if !available_brands.contains(&brand) {
            eprintln!(
                "{} Brand '{}' not found under {}",
                "Error:".red().bold(),
                brand.yellow(),
                brands_dir.display().to_string().yellow()
            );
            eprintln!(
                "       Available brands: {}",
                available_brands.join(", ").cyan()
            );
            process::exit(1);
        }
        vec![brand]
    } else {
        if available_brands.is_empty() {
            eprintln!(
                "{} No brands found under {}",
                "Error:".red().bold(),
                brands_dir.display().to_string().yellow()
            );
            process::exit(1);
        }
        available_brands.clone()
    };

    let config_path = root.join("config.json");
    let static_dir = root.join("static");
    let output_parent = cli.output.unwrap_or_else(|| root.join("dist"));
    let multiple = brands_to_build.len() > 1;
    let validate_only = cli.validate;
    let filter_options = if validate_only {
        // In validate mode --only/--mac are forbidden by clap; build a default
        // FilterOptions without the platform-aware logging make_filter_options does.
        FilterOptions::new().with_mac_mode(MacMode::All)
    } else {
        make_filter_options(cli.only, cli.mac)
    };

    let mut errors: Vec<String> = Vec::new();
    let (action_heading, success_msg, failure_msg) = if validate_only {
        ("Validating", "Brand validation passed!", "Validation failed for")
    } else {
        ("Building", "Brand asset generation completed successfully!", "Generation failed for")
    };

    for brand in &brands_to_build {
        if multiple {
            println!("\n{}", format!("=== {} {} ===", action_heading, brand).bold());
        }

        let source = brands_dir.join(brand);
        let output = output_parent.join(brand);

        match run(
            &config_path,
            &source,
            &static_dir,
            &output,
            filter_options.clone(),
            validate_only,
        ) {
            Ok(_) => {
                println!("\n{} {}", "✓".green().bold(), success_msg.green());
            }
            Err(e) => {
                eprintln!(
                    "\n{} {}: {}",
                    "✗".red().bold(),
                    format!("{} '{}'", failure_msg, brand).red().bold(),
                    e.to_string().red()
                );
                errors.push(brand.clone());
            }
        }
    }

    if !errors.is_empty() {
        if multiple {
            eprintln!(
                "\n{} {} brand(s) failed: {}",
                "✗".red().bold(),
                errors.len(),
                errors.join(", ").yellow()
            );
        }
        process::exit(1);
    }
}
