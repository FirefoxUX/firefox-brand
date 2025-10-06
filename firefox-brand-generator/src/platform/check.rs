use crate::error::{Error, Result};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct PlatformCapabilities {
    pub has_iconutil: bool,
    pub has_actool: bool,
    pub actool_version: Option<String>,
    pub has_sips: bool,
    pub has_hdiutil: bool,
}

impl PlatformCapabilities {
    pub fn detect() -> Self {
        let has_actool = check_command_available("actool");
        let actool_version = if has_actool {
            get_actool_version().ok()
        } else {
            None
        };

        Self {
            has_iconutil: check_command_available("iconutil"),
            has_actool,
            actool_version,
            has_sips: check_command_available("sips"),
            has_hdiutil: check_command_available("hdiutil"),
        }
    }

    /// Validate actool availability and warn about version requirements
    pub fn validate_actool_for_icon_support(&self) -> Result<()> {
        if !self.has_actool {
            return Err(Error::PlatformToolUnavailable("actool".to_string()));
        }

        // Check macOS version (requires macOS 15.0+ / Darwin 25.0+)
        if let Ok(darwin_version) = get_darwin_version() {
            if let Some(major_version) = parse_darwin_major_version(&darwin_version) {
                if major_version < 25 {
                    return Err(Error::UnsupportedToolVersion {
                        tool: "macOS".to_string(),
                        version: format!("Darwin {}", darwin_version),
                        message: "actool .icon support requires macOS 15 (Darwin 25.0) or higher"
                            .to_string(),
                    });
                }
            }
        }

        // Check actool version
        match &self.actool_version {
            Some(version) => {
                // Parse major version number (e.g., "16.0.0" -> 16)
                let major_version = version
                    .split('.')
                    .next()
                    .and_then(|v| v.parse::<u32>().ok());

                if let Some(major_version) = major_version {
                    if major_version < 16 {
                        return Err(Error::UnsupportedToolVersion {
                            tool: "actool".to_string(),
                            version: version.clone(),
                            message: format!(
                                "Unsupported actool version. Must be on actool 16.0.0 or higher but found {}. Install XCode 16 or higher to get a supported version of actool.",
                                version
                            ),
                        });
                    }
                } else {
                    return Err(Error::UnsupportedToolVersion {
                        tool: "actool".to_string(),
                        version: version.clone(),
                        message: "Unable to parse actool version. Is Xcode 16 or higher installed?"
                            .to_string(),
                    });
                }
            }
            None => {
                return Err(Error::UnsupportedToolVersion {
                    tool: "actool".to_string(),
                    version: "unknown".to_string(),
                    message: "Unable to determine actool version. Is Xcode 16 or higher installed?"
                        .to_string(),
                });
            }
        }

        Ok(())
    }
}

fn check_command_available(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Parse major version number from Darwin version string (e.g., "25.0.0" -> 25)
fn parse_darwin_major_version(version: &str) -> Option<u32> {
    version.split('.').next()?.parse::<u32>().ok()
}

/// Get the actool version by parsing its version output
fn get_actool_version() -> Result<String> {
    let output = Command::new("actool")
        .arg("--version")
        .output()
        .map_err(|_| Error::PlatformToolUnavailable("actool".to_string()))?;

    if !output.status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "actool".to_string(),
            code: output.status.code().unwrap_or(-1),
        });
    }
    let version_output = String::from_utf8_lossy(&output.stdout);

    if let Some(version_section) = version_output.split("com.apple.actool.version").nth(1) {
        if let Some(short_bundle_version) = version_section.split("short-bundle-version").nth(1) {
            if let Some(version_start) = short_bundle_version.find("<string>") {
                if let Some(version_end) = short_bundle_version[version_start..].find("</string>") {
                    let version_value =
                        &short_bundle_version[version_start + 8..version_start + version_end];
                    return Ok(version_value.trim().to_string());
                }
            }
        }
    }

    Err(Error::Config(
        "Failed to parse actool version output".to_string(),
    ))
}

/// Get Darwin kernel version
fn get_darwin_version() -> Result<String> {
    let output = Command::new("uname")
        .arg("-r")
        .output()
        .map_err(|_| Error::PlatformToolUnavailable("uname".to_string()))?;

    if !output.status.success() {
        return Err(Error::PlatformToolFailed {
            tool: "uname".to_string(),
            code: output.status.code().unwrap_or(-1),
        });
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(version)
}
