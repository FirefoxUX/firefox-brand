use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("SVG rendering error: {0}")]
    Resvg(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Transformation error: {0}")]
    Transformation(String),

    #[error("Platform tool not available: {0}")]
    PlatformToolUnavailable(String),

    #[error("Platform tool failed: {tool} (exit code: {code})")]
    PlatformToolFailed { tool: String, code: i32 },

    #[error("Unsupported tool version: {tool} version {version} is not supported. {message}")]
    UnsupportedToolVersion {
        tool: String,
        version: String,
        message: String,
    },

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid file type: expected {expected}, got {actual}")]
    InvalidFileType { expected: String, actual: String },

    #[error("Unsupported transformation type: {0}")]
    UnsupportedTransformation(String),

    #[error("Missing brand config value for key: {0}")]
    MissingBrandConfigValue(String),
}

pub type Result<T> = std::result::Result<T, Error>;
