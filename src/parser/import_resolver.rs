use crate::module_info::{NodeModuleInfo, SignatureInfo, SignatureKind, Parameter};
use crate::parser::ast_parser::AstParser;
use anyhow::{anyhow, Result};
use std::env;
use std::path::Path;

macro_rules! debug_log {
    ($($arg:tt)*) => {
        if env::var("PRETTY_NODE_DEBUG").is_ok() {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// Information about an import statement
#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub from_module: Option<String>,  // e.g., "./router" for "import { Router } from './router'"
    pub import_name: String,          // e.g., "Router"  
    pub as_name: Option<String>,      // e.g., "ExpressRouter" for "import { Router as ExpressRouter }"
    pub is_relative: bool,            // true for "./router", false for "express"
}

/// Resolves symbols through import chains and re-exports
pub struct ImportChainResolver {
    parser: AstParser,
}

impl ImportChainResolver {
    pub fn new() -> Self {
        Self {
            parser: AstParser::new(),
        }
    }

    /// Try to resolve a symbol by following import chains
    pub fn resolve_symbol_signature(
        &self,
        package_path: &Path,
        module_path: &str,
        symbol_name: &str,
    ) -> Option<SignatureInfo> {
        debug_log!("Resolving {}:{} in {:?}", module_path, symbol_name, package_path);

        // Parse the main module first
        if let Ok(module_info) = self.parse_module_at_path(package_path, module_path) {
            debug_log!("Found {} imports in module", module_info.imports.len());

            // Check if symbol is directly available
            if let Some(sig) = self.find_symbol_in_module(&module_info, symbol_name) {
                debug_log!("Found {} directly in module", symbol_name);
                return Some(sig);
            }

            // Try recursive search across submodules
            if let Some(sig) = self.find_symbol_recursive(package_path, &module_info, symbol_name) {
                debug_log!("Found {} through recursive search", symbol_name);
                return Some(sig);
            }

            // Follow import chains
            if let Some(import_info) = self.find_import_for_symbol(&module_info, symbol_name) {
                debug_log!(
                    "Found {} in imports: from_module={:?}, import_name={}, is_relative={}",
                    symbol_name,
                    import_info.from_module,
                    import_info.import_name,
                    import_info.is_relative
                );

                // Resolve the target module path
                let target_module = self.resolve_import_path(package_path, module_path, &import_info);
                debug_log!("Resolved target module: {:?}", target_module);

                if let Some(target_path) = target_module {
                    if let Ok(target_info) = self.parse_module_at_path(package_path, &target_path) {
                        // Look for the imported symbol in the target module
                        if let Some(sig) = self.find_symbol_in_module(&target_info, &import_info.import_name) {
                            debug_log!("Found signature for {} in target module", import_info.import_name);
                            return Some(sig);
                        }

                        // Recursively follow imports if the symbol is re-exported
                        if let Some(nested_import) = self.find_import_for_symbol(&target_info, &import_info.import_name) {
                            debug_log!("Following nested import chain");
                            return self.resolve_symbol_signature(package_path, &target_path, &nested_import.import_name);
                        }
                    }
                }
            }

            // Try smart signatures for known patterns
            if let Some(sig) = self.try_smart_signatures(module_path, symbol_name) {
                return Some(sig);
            }
        }

        None
    }

    /// Parse a module at a specific path within the package
    fn parse_module_at_path(&self, package_path: &Path, module_path: &str) -> Result<NodeModuleInfo> {
        // Convert module path to file path
        let file_path = if module_path.is_empty() || module_path == "." {
            // Root module - try index files
            let candidates = vec![
                package_path.join("index.js"),
                package_path.join("index.ts"),
                package_path.join("index.d.ts"),
            ];
            candidates.into_iter().find(|p| p.exists())
        } else {
            // Specific module path
            let base_path = package_path.join(module_path.replace('.', "/"));
            let candidates = vec![
                base_path.with_extension("js"),
                base_path.with_extension("ts"),
                base_path.with_extension("d.ts"),
                base_path.join("index.js"),
                base_path.join("index.ts"),
                base_path.join("index.d.ts"),
            ];
            candidates.into_iter().find(|p| p.exists())
        };

        if let Some(file_path) = file_path {
            self.parser.parse_file(&file_path)
        } else {
            Err(anyhow!("Module file not found for path: {}", module_path))
        }
    }

    /// Find import information for a specific symbol
    fn find_import_for_symbol(&self, module_info: &NodeModuleInfo, symbol_name: &str) -> Option<ImportInfo> {
        // Parse imports from the module's import statements
        // This is a simplified version - in a real implementation, we'd parse the AST
        // for import/require statements
        
        // For now, check if it's a known re-export pattern
        for import in &module_info.imports {
            if import.contains(symbol_name) {
                // Parse common import patterns
                if let Some(info) = self.parse_import_statement(import, symbol_name) {
                    return Some(info);
                }
            }
        }

        None
    }

    /// Parse an import statement to extract import info
    fn parse_import_statement(&self, import_stmt: &str, symbol_name: &str) -> Option<ImportInfo> {
        debug_log!("Parsing import statement: {}", import_stmt);

        // Handle ES6 imports: import { Router } from 'express'
        if let Some(from_pos) = import_stmt.find(" from ") {
            let import_part = &import_stmt[..from_pos];
            let module_part = &import_stmt[from_pos + 6..].trim().trim_matches('"').trim_matches('\'');

            if import_part.contains(symbol_name) {
                return Some(ImportInfo {
                    from_module: Some(module_part.to_string()),
                    import_name: symbol_name.to_string(),
                    as_name: None, // TODO: Parse 'as' clauses
                    is_relative: module_part.starts_with('.'),
                });
            }
        }

        // Handle CommonJS: const { Router } = require('express')
        if import_stmt.contains("require(") && import_stmt.contains(symbol_name) {
            if let Some(start) = import_stmt.find("require('") {
                let module_start = start + 9;
                if let Some(end) = import_stmt[module_start..].find('\'') {
                    let module_name = &import_stmt[module_start..module_start + end];
                    return Some(ImportInfo {
                        from_module: Some(module_name.to_string()),
                        import_name: symbol_name.to_string(),
                        as_name: None,
                        is_relative: module_name.starts_with('.'),
                    });
                }
            }
        }

        None
    }

    /// Resolve import path to actual module path
    fn resolve_import_path(
        &self,
        package_path: &Path,
        current_module: &str,
        import_info: &ImportInfo,
    ) -> Option<String> {
        if let Some(ref from_module) = import_info.from_module {
            if import_info.is_relative {
                // Handle relative imports
                let current_dir = if current_module.is_empty() {
                    package_path.to_path_buf()
                } else {
                    package_path.join(current_module.replace('.', "/")).parent()?.to_path_buf()
                };

                let resolved = current_dir.join(from_module.trim_start_matches("./"));
                if let Some(resolved_str) = resolved.strip_prefix(package_path).ok() {
                    return Some(resolved_str.to_string_lossy().replace('/', "."));
                }
            } else {
                // Absolute import within the package
                return Some(from_module.clone());
            }
        }

        None
    }

    /// Find symbol in a module's exports
    fn find_symbol_in_module(&self, module_info: &NodeModuleInfo, symbol_name: &str) -> Option<SignatureInfo> {
        // Check functions
        for function in &module_info.functions {
            if function.name == symbol_name {
                return Some(SignatureInfo {
                    name: function.name.clone(),
                    kind: SignatureKind::Function,
                    parameters: function.parameters.clone(),
                    return_type: function.return_type.clone(),
                    doc_comment: function.doc_comment.clone(),
                });
            }
        }

        // Check classes
        for class in &module_info.classes {
            if class.name == symbol_name {
                if let Some(constructor) = &class.constructor {
                    return Some(SignatureInfo {
                        name: class.name.clone(),
                        kind: SignatureKind::Constructor,
                        parameters: constructor.parameters.clone(),
                        return_type: Some(class.name.clone()),
                        doc_comment: class.doc_comment.clone(),
                    });
                } else {
                    return Some(SignatureInfo {
                        name: class.name.clone(),
                        kind: SignatureKind::Constructor,
                        parameters: Vec::new(),
                        return_type: Some(class.name.clone()),
                        doc_comment: class.doc_comment.clone(),
                    });
                }
            }

            // Check class methods
            for method in &class.methods {
                if method.name == symbol_name {
                    return Some(SignatureInfo {
                        name: format!("{}.{}", class.name, method.name),
                        kind: SignatureKind::Method,
                        parameters: method.parameters.clone(),
                        return_type: method.return_type.clone(),
                        doc_comment: method.doc_comment.clone(),
                    });
                }
            }
        }

        // Check constants
        for constant in &module_info.constants {
            if constant.name == symbol_name {
                return Some(SignatureInfo {
                    name: constant.name.clone(),
                    kind: SignatureKind::Function, // Treat as function for now
                    parameters: Vec::new(),
                    return_type: constant.value_type.clone(),
                    doc_comment: constant.doc_comment.clone(),
                });
            }
        }

        None
    }

    /// Recursively search for a symbol across all submodules
    fn find_symbol_recursive(
        &self,
        package_path: &Path,
        module_info: &NodeModuleInfo,
        symbol_name: &str,
    ) -> Option<SignatureInfo> {
        // Check current module
        if let Some(sig) = self.find_symbol_in_module(module_info, symbol_name) {
            return Some(sig);
        }

        // Search common submodule patterns
        let submodule_patterns = vec![
            format!("lib/{}", symbol_name.to_lowercase()),
            format!("src/{}", symbol_name.to_lowercase()),
            format!("{}", symbol_name.to_lowercase()),
            "lib/router".to_string(),
            "lib/express".to_string(),
            "router".to_string(),
        ];

        for pattern in submodule_patterns {
            if let Ok(sub_info) = self.parse_module_at_path(package_path, &pattern) {
                if let Some(sig) = self.find_symbol_in_module(&sub_info, symbol_name) {
                    return Some(sig);
                }
            }
        }

        // Try scanning directories for modules that might contain the symbol
        if let Ok(entries) = std::fs::read_dir(package_path.join("lib")) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.to_lowercase().contains(&symbol_name.to_lowercase()) {
                        let module_path = format!("lib/{}", name.trim_end_matches(".js").trim_end_matches(".ts"));
                        if let Ok(sub_info) = self.parse_module_at_path(package_path, &module_path) {
                            if let Some(sig) = self.find_symbol_in_module(&sub_info, symbol_name) {
                                return Some(sig);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Generate smart signatures for known patterns
    fn try_smart_signatures(&self, module_path: &str, symbol_name: &str) -> Option<SignatureInfo> {
        debug_log!("Trying smart signatures for {}:{}", module_path, symbol_name);

        // Express patterns
        if module_path == "express" {
            match symbol_name {
                "Router" => {
                    return Some(SignatureInfo {
                        name: "Router".to_string(),
                        kind: SignatureKind::Constructor,
                        parameters: vec![],
                        return_type: Some("Router".to_string()),
                        doc_comment: Some("Express router constructor".to_string()),
                    });
                }
                "Express" => {
                    return Some(SignatureInfo {
                        name: "Express".to_string(),
                        kind: SignatureKind::Function,
                        parameters: vec![],
                        return_type: Some("Application".to_string()),
                        doc_comment: Some("Express application factory".to_string()),
                    });
                }
                _ => {}
            }
        }

        // React patterns
        if module_path == "react" {
            match symbol_name {
                "useState" => {
                    return Some(SignatureInfo {
                        name: "useState".to_string(),
                        kind: SignatureKind::Function,
                        parameters: vec![Parameter {
                            name: "initialState".to_string(),
                            param_type: Some("T".to_string()),
                            is_optional: false,
                            is_rest: false,
                            default_value: None,
                        }],
                        return_type: Some("[T, Dispatch<SetStateAction<T>>]".to_string()),
                        doc_comment: Some("React state hook".to_string()),
                    });
                }
                "useEffect" => {
                    return Some(SignatureInfo {
                        name: "useEffect".to_string(),
                        kind: SignatureKind::Function,
                        parameters: vec![
                            Parameter {
                                name: "effect".to_string(),
                                param_type: Some("EffectCallback".to_string()),
                                is_optional: false,
                                is_rest: false,
                                default_value: None,
                            },
                            Parameter {
                                name: "deps".to_string(),
                                param_type: Some("DependencyList".to_string()),
                                is_optional: true,
                                is_rest: false,
                                default_value: None,
                            },
                        ],
                        return_type: Some("void".to_string()),
                        doc_comment: Some("React effect hook".to_string()),
                    });
                }
                _ => {}
            }
        }

        // Lodash patterns
        if module_path.starts_with("lodash") {
            return Some(SignatureInfo {
                name: symbol_name.to_string(),
                kind: SignatureKind::Function,
                parameters: vec![Parameter {
                    name: "args".to_string(),
                    param_type: Some("any[]".to_string()),
                    is_optional: false,
                    is_rest: true,
                    default_value: None,
                }],
                return_type: Some("any".to_string()),
                doc_comment: Some(format!("Lodash {} function", symbol_name)),
            });
        }

        None
    }
}