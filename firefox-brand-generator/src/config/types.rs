use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    Source,
    Static,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFileType {
    Png,
    Jpg,
    Bmp,
    Tiff,
    Gif,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FitStrategy {
    Fill,
    Cover,
    Contain,
    ScaleDown,
}

impl Default for FitStrategy {
    fn default() -> Self {
        Self::Contain
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Transformation {
    Raster {
        #[serde(rename = "fileType")]
        file_type: FileType,
        #[serde(rename = "inputPath")]
        input_path: String,
        #[serde(rename = "outputPath")]
        output_path: String,
        #[serde(rename = "outputFileType")]
        output_file_type: OutputFileType,
        width: u32,
        height: u32,
        #[serde(rename = "paddingPixelsWidth")]
        padding_pixels_width: Option<u32>,
        #[serde(rename = "paddingPixelsHeight")]
        padding_pixels_height: Option<u32>,
        #[serde(rename = "offsetX")]
        offset_x: Option<i32>,
        #[serde(rename = "offsetY")]
        offset_y: Option<i32>,
        #[serde(default)]
        fit: FitStrategy,
    },
    Ico {
        #[serde(rename = "fileType")]
        file_type: FileType,
        #[serde(rename = "inputPath")]
        input_path: String,
        #[serde(rename = "outputPath")]
        output_path: String,
        sizes: Vec<u32>,
    },
    Icns {
        #[serde(rename = "fileType")]
        file_type: FileType,
        #[serde(rename = "inputPath")]
        input_path: String,
        #[serde(rename = "outputPath")]
        output_path: String,
        sizes: Vec<u32>,
    },
    AssetsCar {
        #[serde(rename = "liquidGlassIconFileType")]
        liquid_glass_icon_file_type: FileType,
        #[serde(rename = "liquidGlassIconPath")]
        liquid_glass_icon_path: String,
        #[serde(rename = "outputPath")]
        output_path: String,
        #[serde(rename = "appIconInput")]
        app_icon_input: String,
        #[serde(rename = "appIconFileType")]
        app_icon_file_type: FileType,
        #[serde(rename = "iconInput")]
        icon_input: String,
        #[serde(rename = "iconFileType")]
        icon_file_type: FileType,
    },
    Copy {
        #[serde(rename = "fileType")]
        file_type: FileType,
        #[serde(rename = "inputPath")]
        input_path: String,
        #[serde(rename = "outputPath")]
        output_path: String,
    },
    CopyPreprocess {
        #[serde(rename = "fileType")]
        file_type: FileType,
        #[serde(rename = "inputPath")]
        input_path: String,
        #[serde(rename = "outputPath")]
        output_path: String,
    },
    DsStore {
        #[serde(rename = "outputPath")]
        output_path: String,
        #[serde(rename = "appName")]
        app_name: String,
        #[serde(rename = "volumeName")]
        volume_name: String,
        #[serde(rename = "backgroundImage")]
        background_image: String,
        #[serde(rename = "backgroundImageFileType")]
        background_image_file_type: FileType,
        #[serde(rename = "volumeIcon")]
        volume_icon: String,
        #[serde(rename = "volumeIconFileType")]
        volume_icon_file_type: FileType,
        #[serde(rename = "windowPosition")]
        window_position: String,
        #[serde(rename = "windowSize")]
        window_size: String,
        #[serde(rename = "appIconPosition")]
        app_icon_position: String,
        #[serde(rename = "appDropLinkPosition")]
        app_drop_link_position: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "brandConfigPath")]
    pub brand_config_path: String,
    pub transformations: Vec<Transformation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BrandConfig {
    #[serde(default)]
    pub strings: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl Transformation {
    pub fn output_path(&self) -> &str {
        match self {
            Transformation::Raster { output_path, .. } => output_path,
            Transformation::Ico { output_path, .. } => output_path,
            Transformation::Icns { output_path, .. } => output_path,
            Transformation::AssetsCar { output_path, .. } => output_path,
            Transformation::Copy { output_path, .. } => output_path,
            Transformation::CopyPreprocess { output_path, .. } => output_path,
            Transformation::DsStore { output_path, .. } => output_path,
        }
    }

    pub fn transformation_type(&self) -> &str {
        match self {
            Transformation::Raster { .. } => "raster",
            Transformation::Ico { .. } => "ico",
            Transformation::Icns { .. } => "icns",
            Transformation::AssetsCar { .. } => "assets-car",
            Transformation::Copy { .. } => "copy",
            Transformation::CopyPreprocess { .. } => "copy-preprocess",
            Transformation::DsStore { .. } => "ds-store",
        }
    }
}
