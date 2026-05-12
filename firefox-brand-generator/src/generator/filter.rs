use crate::config::{Transformation, TransformationEntry};
use crate::platform::PlatformCapabilities;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum MacMode {
    None,   // Skip ds-store, icns, and assets-car
    Simple, // Run icns and assets-car only
    All,    // Run all transformations (default behavior)
}

impl Default for MacMode {
    fn default() -> Self {
        MacMode::All
    }
}

#[derive(Clone)]
pub struct FilterOptions {
    pub only_types: Option<HashSet<String>>,
    pub mac_mode: MacMode,
    pub brand_name: Option<String>,
}

impl FilterOptions {
    pub fn new() -> Self {
        Self {
            only_types: None,
            mac_mode: MacMode::default(),
            brand_name: None,
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

    pub fn with_brand_name(mut self, brand_name: String) -> Self {
        self.brand_name = Some(brand_name);
        self
    }
}

pub fn filter_transformations(
    transformations: &[TransformationEntry],
    options: &FilterOptions,
    capabilities: &PlatformCapabilities,
) -> Vec<(Transformation, bool)> {
    transformations
        .iter()
        .filter_map(|entry| {
            // Skip if the entry's `only` list is set and this brand isn't in it
            if let Some(ref only_brands) = entry.only {
                let brand_matches = options
                    .brand_name
                    .as_deref()
                    .map_or(false, |name| only_brands.iter().any(|b| b == name));
                if !brand_matches {
                    return None;
                }
            }

            let t = &entry.transformation;
            let transformation_type = t.transformation_type();

            // Check Mac mode filtering
            let mac_allowed = match options.mac_mode {
                MacMode::None => !matches!(transformation_type, "ds-store" | "icns" | "assets-car"),
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
                _ => true,
            };

            let should_warn = !platform_available;

            Some((t.clone(), should_warn))
        })
        .collect()
}
