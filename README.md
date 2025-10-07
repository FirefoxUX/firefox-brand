# Firefox Brand

A tool for generating Firefox brand folder content in all its variations from a smaller set of source assets. This generator processes brand configurations to produce complete branding packages for different Firefox distributions.

## Usage

```bash
# List of all available commands
cargo run -- --help
# Example
cargo run -- ../config.json ../brands/official ../static -o ../dist-official
```

## Command Line Options

### Syntax

`firefox-brand-generator [OPTIONS] <CONFIG> <SOURCE> <STATIC_DIR>`

### Arguments

- **`CONFIG`** - Path to the configuration JSON file
- **`SOURCE`** - Path to the brand-specific source folder
- **`STATIC_DIR`** - Path to the static assets folder

### Options

- **`-o, --output <DIR>`** - Path to the output/destination folder (default: `dist`)
- **`--mac <MODE>`** - Control macOS-specific transformations
  - `none` - Skip all macOS-specific operations
  - `simple` - Run `icns`, `assets-car` only
  - `all` - Run `icns`, `assets-car`, `ds-store`
  - If not specified, defaults to `simple` on macOS and `none` on other platforms
- **`--only <TYPES>`** - Comma-separated list of transformation types to run
  - Available types: `raster`, `ico`, `icns`, `assets-car`, `copy`, `copy-preprocess`, `ds-store`
  - When specified, only these types will be run and `--mac` is ignored
- **`-h, --help`** - Print help information
- **`-V, --version`** - Print version information

## Configuration Format

The main configuration file defines a list of transformations that specify how source assets are processed into output files. Each transformation has a `type` field and specific arguments based on the transformation type.

Source files can come from two locations: a shared "static" folder containing assets used across all brands, or brand-specific "source" folders containing assets unique to each Firefox distribution.

### Transformation Types

- **`raster`** - Converts vector graphics or images to raster formats
  - `fileType`: Source asset location ("source" or "static")
  - `inputPath`: Source file path
  - `outputPath`: Output file path
  - `outputFileType`: Target format ("png", "jpg", "bmp", "tiff", "gif")
  - `width`: Output width in pixels
  - `height`: Output height in pixels
  - `paddingPixelsWidth`: Optional horizontal padding in pixels
  - `paddingPixelsHeight`: Optional vertical padding in pixels
  - `offsetX`: Optional horizontal offset in pixels
  - `offsetY`: Optional vertical offset in pixels
  - `fit`: Scaling strategy ("fill", "cover", "contain", "scale-down") - defaults to "contain"

- **`ico`** - Creates Windows ICO files with multiple sizes
  - `fileType`: Source asset location ("source" or "static")
  - `inputPath`: Source file path
  - `outputPath`: Output ICO file path
  - `sizes`: Array of icon sizes to include (e.g., [256, 48, 32, 16])

- **`icns`** - Creates macOS ICNS files with multiple sizes
  - `fileType`: Source asset location ("source" or "static")
  - `inputPath`: Source file path
  - `outputPath`: Output ICNS file path
  - `sizes`: Array of icon sizes to include (e.g., [1024, 512, 256, 128, 32, 16])

- **`copy`** - Direct file copy without modification
  - `fileType`: Source asset location ("source" or "static")
  - `inputPath`: Source file path
  - `outputPath`: Destination file path

- **`copy-preprocess`** - File copy with template variable substitution
  - `fileType`: Source asset location ("source" or "static")
  - `inputPath`: Source file path
  - `outputPath`: Destination file path

- **`assets-car`** - Creates macOS Assets.car bundle
  - `liquidGlassIconFileType`: Source asset location for liquid glass icon ("source" or "static")
  - `liquidGlassIconPath`: Liquid glass icon path
  - `outputPath`: Assets.car output path
  - `appIconInput`: Application icon source path
  - `appIconFileType`: Source asset location for app icon ("source" or "static")
  - `iconInput`: Generic icon source path
  - `iconFileType`: Source asset location for icon ("source" or "static")

- **`ds-store`** - Generates macOS .DS_Store files for disk images
  - `outputPath`: .DS_Store output path
  - `appName`*: Application name for the volume
  - `volumeName`*: Disk image volume name
  - `backgroundImage`: DMG background image path
  - `backgroundImageFileType`: Source asset location for background ("source" or "static")
  - `volumeIcon`: Volume icon file path
  - `volumeIconFileType`: Source asset location for volume icon ("source" or "static")
  - `windowPosition`*: DMG window position as "x y" (e.g., "200 120")
  - `windowSize`*: DMG window size as "width height" (e.g., "680 400")
  - `appIconPosition`*: App icon position as "x y" (e.g., "209 220")
  - `appDropLinkPosition`*: Applications link position as "x y" (e.g., "472 220")

**Note**: Fields marked with an asterisk (*) support string substitution using template variables from the brand configuration.

## Brand Configuration

Brand configurations are stored in `brand-config.json` files within each brand folder. They contain:

- **`strings`** - Key-value pairs for template substitution (e.g., product names, version strings)
- **`env`** - Environment variables for build configuration

## Template Processing

The `copy-preprocess` transformation supports template variable substitution and conditional logic:

### String Substitution
- **`{{#str key}}`** - Replaces with value from `strings` in brand config
- Example: `{{#str brandName}}` → `Firefox`

### Conditional Blocks
- **`{{#if condition}}`** - Start conditional block
- **`{{#elseif condition}}`** - Alternative condition
- **`{{#else}}`** - Fallback block
- **`{{#endif}}`** - End conditional block

#### Supported Conditions
- **Equality**: `env == value` or `env != value`
- **Logical AND**: `condition1 && condition2`
- **Logical OR**: `condition1 || condition2`  
- **Parentheses**: `(condition1 || condition2) && (condition3)`

#### Examples
```

{{#if name != official && name != aurora}}
Official or aurora build content here
{{#elseif name == nightly}}
Nightly build content
{{#else}}
Other build content
{{#endif}}
```

## macOS-Specific Operations

These transformations require macOS-specific tools and only work on macOS systems:

- **ICNS Generation**: Creates multi-resolution icon bundles using Apple's `iconutil` command-line tool. 
  
  The process generates a temporary `.iconset` directory with PNG files at various sizes (including @2x retina variants for standard sizes like 32, 64, 256, 512, 1024), then uses `iconutil -c icns` to compile them into a single `.icns` file.

- **Assets.car Creation**: Generates compiled asset catalogs using Apple's `actool` (Asset Catalog Tool).
  
  The process creates a temporary `.xcassets` bundle containing `AppIcon.appiconset` (with sizes 16-512px at 1x and 2x scales) and `Icon.iconset` (256px variants), along with a liquid glass icon template directory. 
  
  `actool` then compiles these into an optimized `.car` file. This file is specifically used on newer macOS versions for the App icon, particularly for rendering the liquid glass icon effect.

- **DS_Store Generation**: Creates properly formatted `.DS_Store` files for disk images by generating a temporary DMG using the `create-dmg` script, then extracting the `.DS_Store` file from the mounted volume.
  
  The process includes setting background images with `sips` (to adjust DPI), creating volume icons via ICNS generation, and positioning UI elements. 
