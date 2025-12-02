use crate::config::Transformation;
use crate::platform::PlatformCapabilities;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum MacMode {
    None,   // Skip ds-store, icns, assets-car, and copy-image-mac
    Simple, // Run icns, assets-car, and copy-image-mac only (skip ds-store)
    All,    // Run all transformations (default behavior)
}

impl Default for MacMode {
    fn default() -> Self {
        MacMode::All
    }
}

pub struct FilterOptions {
    pub only_types: Option<HashSet<String>>,
    pub mac_mode: MacMode,
}

impl FilterOptions {
    pub fn new() -> Self {
        Self {
            only_types: None,
            mac_mode: MacMode::default(),
        }
    }

    pub fn with_types(mut self, types: Vec<String>) -> Self {
        self.only_types = Some(types.into_iter().collect());
        self
    }

    pub fn with_mac_mode(mut self, mac_mode: MacMode) -> Self {
        self.mac_mode = mac_mode;
        self
    }
}

pub fn filter_transformations(
    transformations: &[Transformation],
    options: &FilterOptions,
    capabilities: &PlatformCapabilities,
) -> Vec<(Transformation, bool)> {
    transformations
        .iter()
        .filter_map(|t| {
            let transformation_type = t.transformation_type();

            // Check Mac mode filtering first
            let mac_allowed = match options.mac_mode {
                MacMode::None => !matches!(transformation_type, "ds-store" | "icns" | "assets-car" | "copy-image-mac"),
                MacMode::Simple => !matches!(transformation_type, "ds-store"),
                MacMode::All => true,
            };

            if !mac_allowed {
                return None;
            }

            // Check if type filtering is enabled and this type should be included
            let type_match = match &options.only_types {
                Some(types) => types.contains(transformation_type),
                None => true,
            };

            if !type_match {
                return None;
            }

            // Check if platform capabilities are available
            let platform_available = match transformation_type {
                "icns" => capabilities.has_iconutil,
                "assets-car" => capabilities.has_actool,
                "copy-image-mac" => capabilities.has_sips,
                _ => true,
            };

            let should_warn = !platform_available;

            Some((t.clone(), should_warn))
        })
        .collect()
}
