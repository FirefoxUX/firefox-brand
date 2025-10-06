pub mod check;
pub mod macos;

pub use check::PlatformCapabilities;

/// Detect if we're running on macOS
pub fn is_macos() -> bool {
    std::env::consts::OS == "macos"
}
