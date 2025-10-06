use crate::config::types::BrandConfig;
use crate::error::Result;
use regex::{Captures, Regex};

/// Process string replacements in the given content using the brand configuration.
/// This function replaces occurrences of {{#str key}} with the corresponding value
/// from the brand_config.strings map.
pub fn process_string_replacements(content: &str, brand_config: &BrandConfig) -> Result<String> {
    // Create the regex pattern
    let str_regex = Regex::new(r"\{\{#str\s+([^\s\}]+)\}\}").unwrap();

    // Process each match with proper lifetimes in closure
    let result = str_regex
        .replace_all(content, |caps: &Captures| {
            // Extract the key
            let key = caps.get(1).map_or("", |m| m.as_str());

            // Look up the key in the strings map
            match brand_config.strings.get(key) {
                Some(value) => value.to_string(),
                None => {
                    // If the key doesn't exist, leave the placeholder unchanged
                    caps.get(0).map_or("", |m| m.as_str()).to_string()
                }
            }
        })
        .to_string();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_string_replacement_basic() {
        let mut strings = HashMap::new();
        strings.insert("app_name".to_string(), "Firefox".to_string());
        strings.insert("version".to_string(), "1.0".to_string());

        let brand_config = BrandConfig {
            strings,
            env: HashMap::new(),
        };

        let input = "Welcome to {{#str app_name}} version {{#str version}}!";
        let expected = "Welcome to Firefox version 1.0!";

        let result = process_string_replacements(input, &brand_config).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_replacement_with_extensions() {
        let mut strings = HashMap::new();
        strings.insert("appName".to_string(), "Firefox".to_string());
        strings.insert("shortAppName".to_string(), "FF".to_string());

        let brand_config = BrandConfig {
            strings,
            env: HashMap::new(),
        };

        let input = "{{#str appName}}.app";
        let expected = "Firefox.app";

        let result = process_string_replacements(input, &brand_config).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_replacement_missing_key() {
        let brand_config = BrandConfig {
            strings: HashMap::new(),
            env: HashMap::new(),
        };

        let input = "{{#str missing_key}}";
        let expected = "{{#str missing_key}}"; // Should remain unchanged

        let result = process_string_replacements(input, &brand_config).unwrap();
        assert_eq!(result, expected);
    }
}
