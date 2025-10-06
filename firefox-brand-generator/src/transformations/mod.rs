pub mod assets_car;
pub mod copy;
pub mod copy_preprocess;
pub mod dsstore;
pub mod icns;
pub mod ico;
pub mod raster;

use crate::config::{BrandConfig, FileType, Transformation};
use crate::error::{Error, Result};
use crate::platform::PlatformCapabilities;
use std::path::{Path, PathBuf};

pub struct TransformationContext<'a> {
    pub source_dir: &'a Path,
    pub static_dir: &'a Path,
    pub output_dir: &'a Path,
    pub brand_config: &'a BrandConfig,
    pub capabilities: &'a PlatformCapabilities,
}

pub fn execute(transformation: &Transformation, ctx: &TransformationContext) -> Result<()> {
    // Execute the appropriate transformation, handling input/output paths individually
    match transformation {
        Transformation::Raster {
            file_type,
            input_path,
            output_path,
            output_file_type,
            width,
            height,
            padding_pixels_width,
            padding_pixels_height,
            offset_x,
            offset_y,
            fit,
        } => {
            let resolved_input_path =
                resolve_input_path(file_type, input_path, ctx.source_dir, ctx.static_dir)?;
            let resolved_output_path = ctx.output_dir.join(output_path);

            raster::execute(
                &resolved_input_path,
                &resolved_output_path,
                output_file_type,
                *width,
                *height,
                *padding_pixels_width,
                *padding_pixels_height,
                *offset_x,
                *offset_y,
                fit,
            )
        }

        Transformation::Ico {
            file_type,
            input_path,
            output_path,
            sizes,
        } => {
            let resolved_input_path =
                resolve_input_path(file_type, input_path, ctx.source_dir, ctx.static_dir)?;
            let resolved_output_path = ctx.output_dir.join(output_path);

            ico::execute(&resolved_input_path, &resolved_output_path, sizes)
        }

        Transformation::Icns {
            file_type,
            input_path,
            output_path,
            sizes,
        } => {
            if !ctx.capabilities.has_iconutil {
                return Err(Error::PlatformToolUnavailable(
                    "iconutil (required for icns generation)".to_string(),
                ));
            }

            let resolved_input_path =
                resolve_input_path(file_type, input_path, ctx.source_dir, ctx.static_dir)?;
            let resolved_output_path = ctx.output_dir.join(output_path);

            icns::execute(&resolved_input_path, &resolved_output_path, sizes)
        }

        Transformation::AssetsCar {
            input_path,
            file_type,
            output_path,
            app_icon_input,
            app_icon_file_type,
            icon_input,
            icon_file_type,
        } => {
            if !ctx.capabilities.has_actool {
                return Err(Error::PlatformToolUnavailable(
                    "actool (required for Assets.car generation)".to_string(),
                ));
            }

            let resolved_input_path =
                resolve_input_path(file_type, input_path, ctx.source_dir, ctx.static_dir)?;
            let resolved_output_path = ctx.output_dir.join(output_path);

            let app_icon_path = resolve_input_path(
                app_icon_file_type,
                app_icon_input,
                ctx.source_dir,
                ctx.static_dir,
            )?;

            let icon_path_input =
                resolve_input_path(icon_file_type, icon_input, ctx.source_dir, ctx.static_dir)?;

            assets_car::execute(
                &resolved_input_path,
                &resolved_output_path,
                &app_icon_path,
                &icon_path_input,
            )
        }

        Transformation::Copy {
            file_type,
            input_path,
            output_path,
        } => {
            let resolved_input_path =
                resolve_input_path(file_type, input_path, ctx.source_dir, ctx.static_dir)?;
            let resolved_output_path = ctx.output_dir.join(output_path);

            copy::execute(&resolved_input_path, &resolved_output_path)
        }

        Transformation::CopyPreprocess {
            file_type,
            input_path,
            output_path,
        } => {
            let resolved_input_path =
                resolve_input_path(file_type, input_path, ctx.source_dir, ctx.static_dir)?;
            let resolved_output_path = ctx.output_dir.join(output_path);

            copy_preprocess::execute(
                &resolved_input_path,
                &resolved_output_path,
                ctx.brand_config,
            )
        }

        Transformation::DsStore {
            output_path,
            app_name,
            volume_name,
            background_image,
            background_image_file_type,
            volume_icon,
            volume_icon_file_type,
        } => {
            // Check required platform tools
            if !ctx.capabilities.has_sips {
                return Err(Error::PlatformToolUnavailable(
                    "sips (required for .DS_Store generation)".to_string(),
                ));
            }
            if !ctx.capabilities.has_hdiutil {
                return Err(Error::PlatformToolUnavailable(
                    "hdiutil (required for .DS_Store generation)".to_string(),
                ));
            }
            if !ctx.capabilities.has_iconutil {
                return Err(Error::PlatformToolUnavailable(
                    "iconutil (required for .DS_Store generation)".to_string(),
                ));
            }
            let resolved_output_path = ctx.output_dir.join(output_path);

            let background_image_path = resolve_input_path(
                background_image_file_type,
                background_image,
                ctx.source_dir,
                ctx.static_dir,
            )?;

            let volume_icon_path = resolve_input_path(
                volume_icon_file_type,
                volume_icon,
                ctx.source_dir,
                ctx.static_dir,
            )?;

            dsstore::execute(
                &resolved_output_path,
                app_name,
                volume_name,
                &background_image_path,
                &volume_icon_path,
                ctx.brand_config,
            )
        }
    }
}

fn resolve_input_path(
    file_type: &FileType,
    input_path: &str,
    source_dir: &Path,
    static_dir: &Path,
) -> Result<PathBuf> {
    let base_dir = match file_type {
        FileType::Source => source_dir,
        FileType::Static => static_dir,
    };

    let full_path = base_dir.join(input_path);

    if !full_path.exists() {
        return Err(Error::FileNotFound(full_path));
    }

    Ok(full_path)
}
