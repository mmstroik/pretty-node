use crate::module_info::*;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// TypeScript-specific parsing utilities
pub struct TypeScriptParser;

impl TypeScriptParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse TypeScript definition files (.d.ts)
    pub fn parse_declaration_file(&self, file_path: &Path) -> Result<NodeModuleInfo> {
        let content = fs::read_to_string(file_path)?;
        self.parse_declaration_content(
            &content,
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown"),
        )
    }

    /// Parse TypeScript declaration content
    pub fn parse_declaration_content(
        &self,
        content: &str,
        module_name: &str,
    ) -> Result<NodeModuleInfo> {
        let mut module_info = NodeModuleInfo::new(module_name.to_string());

        // Simple regex-based parsing for common TypeScript patterns
        // This is a simplified approach - a full implementation would use a proper TS parser

        self.extract_interfaces(content, &mut module_info);
        self.extract_type_aliases(content, &mut module_info);
        self.extract_function_declarations(content, &mut module_info);
        self.extract_class_declarations(content, &mut module_info);
        self.extract_exports(content, &mut module_info);

        Ok(module_info)
    }

    fn extract_interfaces(&self, content: &str, module_info: &mut NodeModuleInfo) {
        let interface_regex = regex::Regex::new(r"(?m)^export\s+interface\s+(\w+)").unwrap();

        for cap in interface_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let type_info = TypeInfo {
                    name: name.as_str().to_string(),
                    kind: TypeKind::Interface,
                    definition: format!("interface {}", name.as_str()),
                    doc_comment: None,
                };
                module_info.add_type(type_info);
                module_info.exports.push(name.as_str().to_string());
            }
        }
    }

    fn extract_type_aliases(&self, content: &str, module_info: &mut NodeModuleInfo) {
        let type_regex = regex::Regex::new(r"(?m)^export\s+type\s+(\w+)").unwrap();

        for cap in type_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let type_info = TypeInfo {
                    name: name.as_str().to_string(),
                    kind: TypeKind::Type,
                    definition: format!("type {}", name.as_str()),
                    doc_comment: None,
                };
                module_info.add_type(type_info);
                module_info.exports.push(name.as_str().to_string());
            }
        }
    }

    fn extract_function_declarations(&self, content: &str, module_info: &mut NodeModuleInfo) {
        let func_regex =
            regex::Regex::new(r"(?m)^export\s+(?:declare\s+)?function\s+(\w+)\s*\(([^)]*)\)")
                .unwrap();

        for cap in func_regex.captures_iter(content) {
            if let (Some(name), Some(params)) = (cap.get(1), cap.get(2)) {
                let parameters = self.parse_parameter_list(params.as_str());

                let func_info = FunctionInfo {
                    name: name.as_str().to_string(),
                    parameters,
                    return_type: None, // TODO: extract return type
                    is_async: false,
                    is_generator: false,
                    doc_comment: None,
                };

                module_info.add_function(func_info);
                module_info.exports.push(name.as_str().to_string());
            }
        }
    }

    fn extract_class_declarations(&self, content: &str, module_info: &mut NodeModuleInfo) {
        let class_regex = regex::Regex::new(r"(?m)^export\s+(?:declare\s+)?class\s+(\w+)").unwrap();

        for cap in class_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let class_info = ClassInfo {
                    name: name.as_str().to_string(),
                    constructor: None,      // TODO: extract constructor
                    methods: Vec::new(),    // TODO: extract methods
                    properties: Vec::new(), // TODO: extract properties
                    extends: None,
                    implements: Vec::new(),
                    doc_comment: None,
                };

                module_info.add_class(class_info);
                module_info.exports.push(name.as_str().to_string());
            }
        }
    }

    fn extract_exports(&self, content: &str, module_info: &mut NodeModuleInfo) {
        // Extract named exports
        let export_regex = regex::Regex::new(r"export\s*\{\s*([^}]+)\s*\}").unwrap();

        for cap in export_regex.captures_iter(content) {
            if let Some(exports) = cap.get(1) {
                for export in exports.as_str().split(',') {
                    let export_name = export.split_whitespace().next().unwrap_or("").to_string();

                    if !export_name.is_empty() && !module_info.exports.contains(&export_name) {
                        module_info.exports.push(export_name);
                    }
                }
            }
        }
    }

    fn parse_parameter_list(&self, params_str: &str) -> Vec<Parameter> {
        if params_str.trim().is_empty() {
            return Vec::new();
        }

        // Simple parameter parsing - split by comma but respect nested brackets
        let mut parameters = Vec::new();
        let mut current_param = String::new();
        let mut bracket_depth = 0;
        let mut in_string = false;
        let mut string_char = '\0';

        for ch in params_str.chars() {
            match ch {
                '"' | '\'' if !in_string => {
                    in_string = true;
                    string_char = ch;
                    current_param.push(ch);
                }
                ch if in_string && ch == string_char => {
                    in_string = false;
                    current_param.push(ch);
                }
                '(' | '[' | '{' if !in_string => {
                    bracket_depth += 1;
                    current_param.push(ch);
                }
                ')' | ']' | '}' if !in_string => {
                    bracket_depth -= 1;
                    current_param.push(ch);
                }
                ',' if !in_string && bracket_depth == 0 => {
                    if !current_param.trim().is_empty() {
                        parameters.push(self.parse_parameter(current_param.trim()));
                    }
                    current_param.clear();
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }

        // Add the last parameter
        if !current_param.trim().is_empty() {
            parameters.push(self.parse_parameter(current_param.trim()));
        }

        parameters
    }

    fn parse_parameter(&self, param_str: &str) -> Parameter {
        let param_str = param_str.trim();

        // Check for rest parameter
        let is_rest = param_str.starts_with("...");
        let param_str = if is_rest { &param_str[3..] } else { param_str };

        // Check for optional parameter
        let is_optional = param_str.contains('?');

        // Split on colon to separate name and type
        let parts: Vec<&str> = param_str.split(':').collect();
        let name_part = parts[0].trim();
        let type_part = if parts.len() > 1 {
            Some(parts[1].trim().to_string())
        } else {
            None
        };

        // Extract parameter name (remove ? if present)
        let name = name_part.replace('?', "");

        // Check for default value
        let (name, default_value) = if let Some(eq_pos) = name.find('=') {
            let param_name = name[..eq_pos].trim().to_string();
            let default_val = name[eq_pos + 1..].trim().to_string();
            (param_name, Some(default_val))
        } else {
            (name, None)
        };

        Parameter {
            name,
            param_type: type_part,
            is_optional: is_optional || default_value.is_some(),
            is_rest,
            default_value,
        }
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}
