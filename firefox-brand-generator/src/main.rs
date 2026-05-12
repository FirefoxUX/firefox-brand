use clap::{Args, Parser, Subcommand, ValueEnum};
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
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build one or all brands using the standard repo layout (auto-detects paths)
    Build(BuildArgs),
    /// Generate brand assets with full, explicit path control
    Custom(CustomArgs),
}

#[derive(Args)]
struct BuildArgs {
    /// Brand to build (e.g. official, nightly, aurora, unofficial).
    /// Omit to build all brands found under <ROOT>/brands/
    brand: Option<String>,

    /// Repo root directory. Auto-detected by walking up from the current directory
    #[arg(long, value_name = "DIR")]
    root: Option<PathBuf>,

    /// Comma-separated list of transformation types to run. When specified, --mac is ignored.
    /// Available types: raster, ico, icns, assets-car, copy, copy-preprocess, ds-store
    #[arg(long, value_name = "TYPES", value_delimiter = ',')]
    only: Option<Vec<String>>,

    /// Control macOS-specific transformations. Ignored if --only is used.
    /// Defaults to simple on macOS (icns + assets-car) and none elsewhere
    #[arg(long, value_enum, value_name = "MODE")]
    mac: Option<MacModeArg>,
}

#[derive(Args)]
struct CustomArgs {
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

    /// Comma-separated list of transformation types to run. When specified, --mac is ignored.
    /// Available types: raster, ico, icns, assets-car, copy, copy-preprocess, ds-store
    #[arg(long, value_name = "TYPES", value_delimiter = ',')]
    only: Option<Vec<String>>,

    /// Control macOS-specific transformations. Ignored if --only is used.
    /// Defaults to simple on macOS (icns + assets-car) and none elsewhere
    #[arg(long, value_enum, value_name = "MODE")]
    mac: Option<MacModeArg>,
}

fn make_filter_options(only: Option<Vec<String>>, mac: Option<MacModeArg>) -> FilterOptions {
    let mut filter_options = if let Some(types) = only {
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
    };

    filter_options = filter_options.with_mac_mode(mac_mode);
    filter_options
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

fn run_build(args: BuildArgs) {
    // Resolve repo root
    let root = if let Some(r) = args.root {
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
                eprintln!("       Run from within the repo or use {} to specify the root.", "--root <DIR>".cyan());
                process::exit(1);
            }
        }
    };

    let brands_dir = root.join("brands");
    let available_brands = discover_brands(&brands_dir);

    // Determine which brands to build
    let brands_to_build: Vec<String> = if let Some(brand) = args.brand {
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
    let multiple = brands_to_build.len() > 1;
    let filter_options = make_filter_options(args.only, args.mac);

    let mut errors: Vec<String> = Vec::new();

    for brand in &brands_to_build {
        if multiple {
            println!("\n{}", format!("=== Building {} ===", brand).bold());
        }

        let source = brands_dir.join(brand);
        let output = root.join(format!("dist-{}", brand));

        match run(&config_path, &source, &static_dir, &output, filter_options.clone()) {
            Ok(_) => {
                println!(
                    "\n{} {}",
                    "✓".green().bold(),
                    format!("Brand asset generation completed successfully!").green()
                );
            }
            Err(e) => {
                eprintln!(
                    "\n{} {}: {}",
                    "✗".red().bold(),
                    format!("Generation failed for '{}'", brand).red().bold(),
                    e.to_string().red()
                );
                errors.push(brand.clone());
            }
        }
    }

    if multiple && !errors.is_empty() {
        eprintln!(
            "\n{} {} brand(s) failed: {}",
            "✗".red().bold(),
            errors.len(),
            errors.join(", ").yellow()
        );
        process::exit(1);
    } else if !errors.is_empty() {
        process::exit(1);
    }
}

fn run_custom(args: CustomArgs) {
    // Validate paths
    if !args.config.exists() {
        eprintln!(
            "{} Config file not found: {}",
            "Error:".red().bold(),
            args.config.display().to_string().yellow()
        );
        process::exit(1);
    }
    if !args.source.exists() {
        eprintln!(
            "{} Source directory not found: {}",
            "Error:".red().bold(),
            args.source.display().to_string().yellow()
        );
        process::exit(1);
    }
    if !args.static_dir.exists() {
        eprintln!(
            "{} Static directory not found: {}",
            "Error:".red().bold(),
            args.static_dir.display().to_string().yellow()
        );
        process::exit(1);
    }

    let filter_options = make_filter_options(args.only, args.mac);

    match run(&args.config, &args.source, &args.static_dir, &args.output, filter_options) {
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => run_build(args),
        Commands::Custom(args) => run_custom(args),
    }
}
