use crate::module_info::Parameter;
use std::env;

macro_rules! debug_log {
    ($($arg:tt)*) => {
        if env::var("PRETTY_NODE_DEBUG").is_ok() {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// Parse parameter strings with proper handling of nested brackets, quotes, and complex types
pub struct ParameterParser;

impl ParameterParser {
    pub fn new() -> Self {
        Self
    }

    /// Split parameters string respecting nested brackets, quotes, and generics
    /// This is similar to pretty-mod's split_parameters function
    pub fn split_parameters(&self, params: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut angle_depth = 0; // For TypeScript generics like Array<T>
        let mut in_quotes = false;
        let mut quote_char = '\0';
        let mut prev_char = '\0';
        
        debug_log!("Splitting parameters: {}", params);
        
        for ch in params.chars() {
            match ch {
                '\'' | '"' | '`' if prev_char != '\\' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                        quote_char = '\0';
                    }
                }
                '[' | '(' | '{' if !in_quotes => depth += 1,
                ']' | ')' | '}' if !in_quotes => depth -= 1,
                '<' if !in_quotes => angle_depth += 1,
                '>' if !in_quotes => angle_depth -= 1,
                ',' if depth == 0 && angle_depth == 0 && !in_quotes => {
                    // Found a top-level comma
                    let param = current.trim().to_string();
                    if !param.is_empty() {
                        result.push(param);
                    }
                    current.clear();
                    prev_char = ch;
                    continue;
                }
                _ => {}
            }
            current.push(ch);
            prev_char = ch;
        }
        
        // Don't forget the last parameter
        let param = current.trim().to_string();
        if !param.is_empty() {
            result.push(param);
        }
        
        debug_log!("Split result: {:?}", result);
        result
    }

    /// Parse a single parameter string into a Parameter struct
    pub fn parse_parameter(&self, param_str: &str) -> Parameter {
        debug_log!("Parsing parameter: {}", param_str);
        
        let trimmed = param_str.trim();
        
        // Handle rest parameters (...args)
        if trimmed.starts_with("...") {
            let rest_param = &trimmed[3..];
            let (name, param_type) = self.extract_name_and_type(rest_param);
            return Parameter {
                name,
                param_type,
                is_optional: false,
                is_rest: true,
                default_value: None,
            };
        }
        
        // Check for default value (param = value)
        if let Some(eq_pos) = self.find_assignment_operator(trimmed) {
            let param_part = trimmed[..eq_pos].trim();
            let default_part = trimmed[eq_pos + 1..].trim();
            
            let (name, param_type) = self.extract_name_and_type(param_part);
            return Parameter {
                name,
                param_type,
                is_optional: true, // Parameters with defaults are optional
                is_rest: false,
                default_value: Some(default_part.to_string()),
            };
        }
        
        // Check for optional parameter (param? or param?: type)
        let (name_type_part, is_optional) = if trimmed.ends_with('?') {
            (&trimmed[..trimmed.len() - 1], true)
        } else {
            (trimmed, false)
        };
        
        let (mut name, param_type) = self.extract_name_and_type(name_type_part);
        
        // Handle case where ? is attached to the parameter name (name?: type)
        let is_optional = if name.ends_with('?') {
            name = name[..name.len() - 1].to_string();
            true
        } else {
            is_optional
        };
        
        Parameter {
            name,
            param_type,
            is_optional,
            is_rest: false,
            default_value: None,
        }
    }
    
    /// Extract parameter name and type from a string like "name: Type"
    fn extract_name_and_type(&self, param_str: &str) -> (String, Option<String>) {
        if let Some(colon_pos) = self.find_type_separator(param_str) {
            let name = param_str[..colon_pos].trim().to_string();
            let type_part = param_str[colon_pos + 1..].trim();
            let param_type = if type_part.is_empty() {
                None
            } else {
                Some(type_part.to_string())
            };
            (name, param_type)
        } else {
            // No type annotation
            (param_str.trim().to_string(), None)
        }
    }
    
    /// Find the colon that separates name from type, respecting nested structures
    fn find_type_separator(&self, param_str: &str) -> Option<usize> {
        let mut depth = 0;
        let mut angle_depth = 0;
        let mut in_quotes = false;
        let mut quote_char = '\0';
        let mut prev_char = '\0';
        
        for (i, ch) in param_str.char_indices() {
            match ch {
                '\'' | '"' | '`' if prev_char != '\\' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                        quote_char = '\0';
                    }
                }
                '[' | '(' | '{' if !in_quotes => depth += 1,
                ']' | ')' | '}' if !in_quotes => depth -= 1,
                '<' if !in_quotes => angle_depth += 1,
                '>' if !in_quotes => angle_depth -= 1,
                ':' if depth == 0 && angle_depth == 0 && !in_quotes => {
                    return Some(i);
                }
                _ => {}
            }
            prev_char = ch;
        }
        
        None
    }
    
    /// Find the assignment operator (=) respecting nested structures
    fn find_assignment_operator(&self, param_str: &str) -> Option<usize> {
        let mut depth = 0;
        let mut angle_depth = 0;
        let mut in_quotes = false;
        let mut quote_char = '\0';
        let mut prev_char = '\0';
        
        for (i, ch) in param_str.char_indices() {
            match ch {
                '\'' | '"' | '`' if prev_char != '\\' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                        quote_char = '\0';
                    }
                }
                '[' | '(' | '{' if !in_quotes => depth += 1,
                ']' | ')' | '}' if !in_quotes => depth -= 1,
                '<' if !in_quotes => angle_depth += 1,
                '>' if !in_quotes => angle_depth -= 1,
                '=' if depth == 0 && angle_depth == 0 && !in_quotes => {
                    // Make sure it's not == or ===
                    let next_chars: String = param_str.chars().skip(i + 1).take(2).collect();
                    if !next_chars.starts_with('=') {
                        return Some(i);
                    }
                }
                _ => {}
            }
            prev_char = ch;
        }
        
        None
    }
    
    /// Parse parameters from a function signature string
    pub fn parse_parameters_from_signature(&self, signature: &str) -> Vec<Parameter> {
        debug_log!("Parsing parameters from signature: {}", signature);
        
        // Extract parameters part from signature (between first ( and last ))
        let param_start = signature.find('(').unwrap_or(0);
        let param_end = signature.rfind(')').unwrap_or(signature.len());
        
        if param_start >= param_end {
            return Vec::new();
        }
        
        let params_str = &signature[param_start + 1..param_end];
        let param_strings = self.split_parameters(params_str);
        
        param_strings.iter()
            .filter(|s| !s.is_empty())
            .map(|s| self.parse_parameter(s))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split_simple_parameters() {
        let parser = ParameterParser::new();
        let result = parser.split_parameters("a, b, c");
        assert_eq!(result, vec!["a", "b", "c"]);
    }
    
    #[test]
    fn test_split_parameters_with_types() {
        let parser = ParameterParser::new();
        let result = parser.split_parameters("name: string, age: number, isActive: boolean");
        assert_eq!(result, vec!["name: string", "age: number", "isActive: boolean"]);
    }
    
    #[test]
    fn test_split_parameters_with_generics() {
        let parser = ParameterParser::new();
        let result = parser.split_parameters("items: Array<T>, callback: (item: T) => boolean");
        assert_eq!(result, vec!["items: Array<T>", "callback: (item: T) => boolean"]);
    }
    
    #[test]
    fn test_split_parameters_with_nested_brackets() {
        let parser = ParameterParser::new();
        let result = parser.split_parameters("config: { host: string, port: number }, options: [string, number]");
        assert_eq!(result, vec!["config: { host: string, port: number }", "options: [string, number]"]);
    }
    
    #[test]
    fn test_parse_optional_parameter() {
        let parser = ParameterParser::new();
        let result = parser.parse_parameter("name?: string");
        assert_eq!(result.name, "name");
        assert_eq!(result.param_type, Some("string".to_string()));
        assert!(result.is_optional);
    }
    
    #[test]
    fn test_parse_rest_parameter() {
        let parser = ParameterParser::new();
        let result = parser.parse_parameter("...args: any[]");
        assert_eq!(result.name, "args");
        assert_eq!(result.param_type, Some("any[]".to_string()));
        assert!(result.is_rest);
    }
    
    #[test]
    fn test_parse_parameter_with_default() {
        let parser = ParameterParser::new();
        let result = parser.parse_parameter("timeout: number = 5000");
        assert_eq!(result.name, "timeout");
        assert_eq!(result.param_type, Some("number".to_string()));
        assert!(result.is_optional);
        assert_eq!(result.default_value, Some("5000".to_string()));
    }
}