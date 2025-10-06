use clap::{Parser, ValueEnum};
use firefox_brand_generator::{FilterOptions, MacMode, is_macos, run};
use owo_colors::OwoColorize;
use std::path::PathBuf;
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
    /// Path to the configuration JSON file
    #[arg(value_name = "CONFIG")]
    config: PathBuf,

    /// Path to the brand-specific source folder
    #[arg(value_name = "SOURCE")]
    source: PathBuf,

    /// Path to the static assets folder
    #[arg(value_name = "STATIC_DIR")]
    static_dir: PathBuf,

    /// Path to the output/destination folder
    #[arg(short, long, value_name = "DIR", default_value = "dist")]
    output: PathBuf,

    #[arg(
        long,
        value_name = "TYPES",
        value_delimiter = ',',
        help = "Comma-separated list of transformation types to run. If specified, only these types will be run and --mac is ignored.\nAvailable types: raster, ico, icns, assets-car, copy, copy-preprocess, ds-store"
    )]
    only: Option<Vec<String>>,

    /// Mac-specific transformation mode
    #[arg(
        long,
        value_enum,
        help = "Control macOS-specific transformations. Ignored if --only is used. Options:\n  - none (skip ds-store, icns, assets-car)\n  - simple (run icns, assets-car only)\n  - all (run all)."
    )]
    mac: Option<MacModeArg>,
}

fn main() {
    let cli = Cli::parse();

    // Validate paths
    if !cli.config.exists() {
        eprintln!(
            "{} Config file not found: {}",
            "Error:".red().bold(),
            cli.config.display().to_string().yellow()
        );
        process::exit(1);
    }

    if !cli.source.exists() {
        eprintln!(
            "{} Source directory not found: {}",
            "Error:".red().bold(),
            cli.source.display().to_string().yellow()
        );
        process::exit(1);
    }

    if !cli.static_dir.exists() {
        eprintln!(
            "{} Static directory not found: {}",
            "Error:".red().bold(),
            cli.static_dir.display().to_string().yellow()
        );
        process::exit(1);
    }

    // Build filter options
    let mut filter_options = if let Some(types) = cli.only {
        FilterOptions::new().with_types(types)
    } else {
        FilterOptions::new()
    };

    // Apply Mac mode filtering
    let mac_mode = if let Some(mac_mode) = cli.mac {
        // Use explicitly specified mode
        mac_mode.into()
    } else {
        if filter_options.only_types.is_some() {
            MacMode::All
        } else if is_macos() {
            println!(
                "{} Auto-detected macOS: enabling {} (icns + assets-car transformations)",
                "[Info]".on_blue().bold(),
                "simple Mac mode".bold()
            );
            println!("       To run all Mac-specific transformations, use the --mac all option.");
            MacMode::Simple
        } else {
            println!(
                "{} Non-macOS platform detected: disabling Mac-specific transformations (ds-store, icns, assets-car)",
                "[Info]".on_blue().bold()
            );
            MacMode::None
        }
    };
    filter_options = filter_options.with_mac_mode(mac_mode);

    // Run the generator
    match run(
        &cli.config,
        &cli.source,
        &cli.static_dir,
        &cli.output,
        filter_options,
    ) {
        Ok(_) => {
            println!(
                "\n{} {}",
                "✓".green().bold(),
                "Brand asset generation completed successfully!".green()
            );
        }
        Err(e) => {
            eprintln!(
                "\n{} {}: {}",
                "✗".red().bold(),
                "Generation failed".red().bold(),
                e.to_string().red()
            );
            process::exit(1);
        }
    }
}
