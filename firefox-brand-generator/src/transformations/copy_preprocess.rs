use crate::config::types::BrandConfig;
use crate::error::{Error, Result};
use crate::utils::string_processing;
use owo_colors::OwoColorize;
use regex::Regex;
use std::fs;
use std::path::Path;

// Helper function to evaluate a single condition
fn evaluate_single_condition(
    var_name: &str,
    operator: &str,
    expected_value: &str,
    brand_config: &BrandConfig,
) -> bool {
    brand_config.env.get(var_name).map_or(false, |value| {
        match operator {
            "==" => value == expected_value,
            "!=" => value != expected_value,
            _ => false, // Unsupported operator
        }
    })
}

// Parse and evaluate a complex condition expression
fn evaluate_condition_expression(expression: &str, brand_config: &BrandConfig) -> bool {
    // Trim whitespace from the expression
    let expression = expression.trim();

    // Early return for empty expressions
    if expression.is_empty() {
        return false;
    }

    // Check if the entire expression is wrapped in parentheses
    if expression.starts_with('(') && expression.ends_with(')') {
        let inner = &expression[1..expression.len() - 1].trim();
        if !inner.is_empty() {
            // Make sure the parentheses are balanced
            let mut depth = 0;
            let mut balanced = true;

            for c in inner.chars() {
                if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                    if depth < 0 {
                        balanced = false;
                        break;
                    }
                }
            }

            if balanced && depth == 0 {
                // If parentheses are balanced, evaluate the inner expression
                return evaluate_condition_expression(inner, brand_config);
            }
        }
    }

    // Handle OR (||) operator - lowest precedence
    if expression.contains("||") {
        // Split by || but respect parentheses
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut paren_depth = 0;
        let mut i = 0;

        while i < expression.len() {
            let c = expression.chars().nth(i).unwrap();

            if c == '(' {
                paren_depth += 1;
                current_part.push(c);
            } else if c == ')' {
                paren_depth -= 1;
                current_part.push(c);
            } else if paren_depth == 0
                && i + 1 < expression.len()
                && c == '|'
                && expression.chars().nth(i + 1).unwrap() == '|'
            {
                // Found || outside of parentheses, split here
                parts.push(current_part);
                current_part = String::new();
                i += 1; // Skip the second '|'
            } else {
                current_part.push(c);
            }

            i += 1;
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        if !parts.is_empty() {
            return parts
                .iter()
                .any(|part| evaluate_condition_expression(part.trim(), brand_config));
        }
    }

    // Handle AND (&&) operator - higher precedence than OR
    if expression.contains("&&") {
        // Split by && but respect parentheses
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut paren_depth = 0;
        let mut i = 0;

        while i < expression.len() {
            let c = expression.chars().nth(i).unwrap();

            if c == '(' {
                paren_depth += 1;
                current_part.push(c);
            } else if c == ')' {
                paren_depth -= 1;
                current_part.push(c);
            } else if paren_depth == 0
                && i + 1 < expression.len()
                && c == '&'
                && expression.chars().nth(i + 1).unwrap() == '&'
            {
                // Found && outside of parentheses, split here
                parts.push(current_part);
                current_part = String::new();
                i += 1; // Skip the second '&'
            } else {
                current_part.push(c);
            }

            i += 1;
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        if !parts.is_empty() {
            return parts
                .iter()
                .all(|part| evaluate_condition_expression(part.trim(), brand_config));
        }
    }

    // Handle basic condition (var == value or var != value)
    let re = Regex::new(r"^\s*([^\s=!]+)\s*(==|!=)\s*([^\s]+)\s*$").unwrap();
    if let Some(caps) = re.captures(expression) {
        let var_name = caps.get(1).map_or("", |m| m.as_str());
        let operator = caps.get(2).map_or("", |m| m.as_str());
        let expected_value = caps.get(3).map_or("", |m| m.as_str());

        return evaluate_single_condition(var_name, operator, expected_value, brand_config);
    }

    // Invalid or unsupported expression format
    println!(
        "{} Invalid condition expression: '{}'",
        "Warning:".yellow().bold(),
        expression.yellow()
    );
    false
}

pub fn execute(input_path: &Path, output_path: &Path, brand_config: &BrandConfig) -> Result<()> {
    // Read the input file
    let content = fs::read_to_string(input_path)
        .map_err(|_| Error::FileNotFound(input_path.to_path_buf()))?;

    // Preprocess the content
    let processed_content = preprocess_content(&content, brand_config)?;

    // Ensure the output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the processed content to the output file
    fs::write(output_path, processed_content)?;

    Ok(())
}

fn preprocess_content(content: &str, brand_config: &BrandConfig) -> Result<String> {
    let mut result = String::from(content);

    // Process conditional blocks
    result = process_if_blocks(&result, brand_config)?;

    // Process string replacements
    result = string_processing::process_string_replacements(&result, brand_config)?;

    Ok(result)
}

fn process_if_blocks(content: &str, brand_config: &BrandConfig) -> Result<String> {
    // Process line-by-line to handle conditionals
    let lines: Vec<&str> = content.lines().collect();

    // Initialize variables for tracking state
    let mut result_lines = Vec::with_capacity(lines.len());
    let mut in_block_conditional = false;
    let mut block_condition_met = false;
    let mut skip_until_endif = false;
    let mut i = 0;

    // Regex patterns for identifying conditional blocks
    let block_if_start = Regex::new(r"^\s*\{\{#if\s+(.*?)\s*\}\}\s*$").unwrap();
    let block_elseif = Regex::new(r"^\s*\{\{#elseif\s+(.*?)\s*\}\}\s*$").unwrap();
    let block_else = Regex::new(r"^\s*\{\{#else\}\}\s*$").unwrap();
    let block_endif = Regex::new(r"^\s*\{\{#endif\}\}\s*$").unwrap();

    while i < lines.len() {
        let line = lines[i];

        // Check for block conditional start
        if block_if_start.is_match(line) {
            // Extract condition expression
            if let Some(caps) = block_if_start.captures(line) {
                let condition_expr = caps.get(1).map_or("", |m| m.as_str());

                // Evaluate the complex condition expression
                let condition_met = evaluate_condition_expression(condition_expr, brand_config);

                in_block_conditional = true;
                block_condition_met = condition_met;
                skip_until_endif = !condition_met;
            }
        }
        // Check for elseif block
        else if block_elseif.is_match(line) {
            if in_block_conditional {
                if block_condition_met {
                    // A previous condition was already met, skip this branch
                    skip_until_endif = true;
                } else {
                    // Check if this condition is met
                    if let Some(caps) = block_elseif.captures(line) {
                        let condition_expr = caps.get(1).map_or("", |m| m.as_str());

                        // Evaluate the complex condition expression
                        let condition_met =
                            evaluate_condition_expression(condition_expr, brand_config);

                        block_condition_met = condition_met;
                        skip_until_endif = !condition_met;
                    }
                }
            } else {
                // Standalone elseif, just include it as text
                result_lines.push(line.to_string());
            }
        }
        // Check for else block
        else if block_else.is_match(line) {
            if in_block_conditional {
                skip_until_endif = block_condition_met; // Skip if any previous condition was met
            } else {
                // Standalone else, just include it as text
                result_lines.push(line.to_string());
            }
        }
        // Check for endif
        else if block_endif.is_match(line) {
            if in_block_conditional {
                in_block_conditional = false;
                skip_until_endif = false;
            } else {
                // Standalone endif, just include it as text
                result_lines.push(line.to_string());
            }
        }
        // Regular line or line inside conditional block
        else {
            if !in_block_conditional || (!skip_until_endif) {
                // Add the line unchanged
                result_lines.push(line.to_string());
            }
        }

        // Move to next line
        i += 1;
    }

    // Join processed lines and return
    Ok(result_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_process_if_blocks() {
        let mut env = HashMap::new();
        env.insert("GL_ES".to_string(), "true".to_string());
        env.insert("PLATFORM".to_string(), "macos".to_string());
        env.insert("DEBUG".to_string(), "true".to_string());

        let brand_config = BrandConfig {
            strings: HashMap::new(),
            env,
        };

        // Test block-style conditional (true case)
        let input = "foo\n{{#if GL_ES == true}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nbar\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test block-style conditional (false case)
        let input = "foo\n{{#if GL_ES == false}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test block-style conditional with else (false case)
        let input = "foo\n{{#if GL_ES == false}}\nbar\n{{#else}}\nbaz\n{{#endif}}\nfoo";
        let expected = "foo\nbaz\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for not equal operator
        let input = "foo\n{{#if GL_ES != false}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nbar\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for AND operator
        let input = "foo\n{{#if GL_ES == true && PLATFORM == macos}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nbar\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for OR operator
        let input = "foo\n{{#if GL_ES == false || PLATFORM == macos}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nbar\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for parentheses - simplify to debug
        let input = "foo\n{{#if GL_ES == true && PLATFORM == macos}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nbar\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case with parentheses around simple conditions
        let input = "foo\n{{#if (GL_ES == true) && (PLATFORM == macos)}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nbar\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for complex condition with not equal
        let input = "foo\n{{#if PLATFORM != windows || DEBUG == true}}\nbar\n{{#endif}}\nfoo";
        let expected = "foo\nbar\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for elseif - first condition true
        let input = "foo\n{{#if GL_ES == true}}\nyes-if\n{{#elseif PLATFORM == windows}}\nyes-elseif\n{{#else}}\nyes-else\n{{#endif}}\nfoo";
        let expected = "foo\nyes-if\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for elseif - elseif condition true
        let input = "foo\n{{#if GL_ES == false}}\nyes-if\n{{#elseif PLATFORM == macos}}\nyes-elseif\n{{#else}}\nyes-else\n{{#endif}}\nfoo";
        let expected = "foo\nyes-elseif\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for elseif - else condition true
        let input = "foo\n{{#if GL_ES == false}}\nyes-if\n{{#elseif PLATFORM == windows}}\nyes-elseif\n{{#else}}\nyes-else\n{{#endif}}\nfoo";
        let expected = "foo\nyes-else\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for multiple elseif conditions
        let input = "foo\n{{#if GL_ES == false}}\n1\n{{#elseif PLATFORM == windows}}\n2\n{{#elseif DEBUG == true}}\n3\n{{#else}}\n4\n{{#endif}}\nfoo";
        let expected = "foo\n3\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Test case for complex conditions in elseif
        let input = "foo\n{{#if GL_ES == false}}\n1\n{{#elseif PLATFORM != macos || DEBUG != true}}\n2\n{{#elseif PLATFORM == macos && DEBUG == true}}\n3\n{{#endif}}\nfoo";
        let expected = "foo\n3\nfoo";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);

        // Note: Inline conditionals are no longer supported - they must be on their own line

        // Case: Testing indentation preservation with block conditions
        let input = "start\n  {{#if GL_ES == true}}\n  indented content\n  {{#endif}}\nend";
        let expected = "start\n  indented content\nend";
        assert_eq!(process_if_blocks(input, &brand_config).unwrap(), expected);
    }

    #[test]
    fn test_process_string_replacements() {
        let mut strings = HashMap::new();
        strings.insert("brand_name".to_string(), "Firefox".to_string());
        strings.insert("company".to_string(), "Mozilla".to_string());

        let brand_config = BrandConfig {
            strings,
            env: HashMap::new(),
        };

        // Test basic replacement
        let input = "Welcome to {{#str brand_name}}!";
        let expected = "Welcome to Firefox!";
        assert_eq!(
            string_processing::process_string_replacements(input, &brand_config).unwrap(),
            expected
        );

        // Test multiple replacements
        let input = "{{#str brand_name}} is made by {{#str company}}";
        let expected = "Firefox is made by Mozilla";
        assert_eq!(
            string_processing::process_string_replacements(input, &brand_config).unwrap(),
            expected
        );

        // Test key doesn't exist
        let input = "Unknown key: {{#str unknown_key}}";
        let expected = "Unknown key: {{#str unknown_key}}";
        assert_eq!(
            string_processing::process_string_replacements(input, &brand_config).unwrap(),
            expected
        );

        // Test within text
        let input = "Product name: {{#str brand_name}} version 123";
        let expected = "Product name: Firefox version 123";
        assert_eq!(
            string_processing::process_string_replacements(input, &brand_config).unwrap(),
            expected
        );
    }

    #[test]
    fn test_combined_processing() {
        let mut strings = HashMap::new();
        strings.insert("brand_name".to_string(), "Firefox".to_string());

        let mut env = HashMap::new();
        env.insert("PLATFORM".to_string(), "macos".to_string());
        env.insert("DEBUG".to_string(), "true".to_string());

        let brand_config = BrandConfig { strings, env };

        // Test with simple condition
        let input = "{{#if PLATFORM == macos}}\n{{#str brand_name}} for Mac\n{{#else}}\n{{#str brand_name}} for Windows\n{{#endif}}";
        let expected = "Firefox for Mac";

        // Process in the correct order - first if blocks, then string replacements
        let result = process_if_blocks(input, &brand_config).unwrap();
        let result =
            string_processing::process_string_replacements(&result, &brand_config).unwrap();

        assert_eq!(result, expected);

        // Test with complex condition
        let input = "{{#if PLATFORM == macos && DEBUG == true}}\n{{#str brand_name}} for Mac (Debug)\n{{#else}}\n{{#str brand_name}} for Windows\n{{#endif}}";
        let expected = "Firefox for Mac (Debug)";

        let result = process_if_blocks(input, &brand_config).unwrap();
        let result =
            string_processing::process_string_replacements(&result, &brand_config).unwrap();

        assert_eq!(result, expected);

        // Test with not equal and parentheses
        let input = "{{#if PLATFORM != windows && (DEBUG == true)}}\n{{#str brand_name}} for Non-Windows (Debug)\n{{#else}}\n{{#str brand_name}} for Windows\n{{#endif}}";
        let expected = "Firefox for Non-Windows (Debug)";

        let result = process_if_blocks(input, &brand_config).unwrap();
        let result =
            string_processing::process_string_replacements(&result, &brand_config).unwrap();

        assert_eq!(result, expected);

        // Test with elseif chain
        let input = "{{#if PLATFORM == windows}}\n{{#str brand_name}} for Windows\n{{#elseif PLATFORM == linux}}\n{{#str brand_name}} for Linux\n{{#elseif PLATFORM == macos}}\n{{#str brand_name}} for macOS\n{{#else}}\n{{#str brand_name}} for Unknown Platform\n{{#endif}}";
        let expected = "Firefox for macOS";

        let result = process_if_blocks(input, &brand_config).unwrap();
        let result =
            string_processing::process_string_replacements(&result, &brand_config).unwrap();

        assert_eq!(result, expected);
    }
}
