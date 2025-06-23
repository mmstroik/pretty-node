use crate::module_info::SignatureInfo;
use crate::npm_client::NpmClient;
use crate::parser::ast_parser::AstParser;
use crate::utils::{extract_base_package, parse_package_spec};
use anyhow::{anyhow, Result};
use std::env;
use std::path::Path;

/// Extract signature information for a given import path
pub async fn extract_signature(import_path: &str, quiet: bool) -> Result<SignatureInfo> {
    let (module_path, symbol_name) = parse_import_path(import_path)?;

    // Try to find the package locally first
    let npm_client = NpmClient::new();
    let search_paths = vec![
        env::current_dir()?,
        env::current_dir()?.join(".."),
        env::current_dir()?.join("../.."),
    ];

    let base_package = extract_base_package(&module_path);

    if let Some(local_path) = npm_client.find_local_package(&base_package, &search_paths) {
        if let Ok(signature) = extract_signature_from_local(&local_path, &module_path, &symbol_name)
        {
            return Ok(signature);
        }
    }

    // Try to download and extract signature
    let (package_name, version) = parse_package_spec(&base_package);
    let package_info = npm_client
        .get_package_info(&package_name, version.as_deref())
        .await?;
    let temp_dir = npm_client.download_package(&package_info, quiet).await?;

    let package_path = temp_dir.path().join("package");
    extract_signature_from_local(&package_path, &module_path, &symbol_name)
}

fn parse_import_path(import_path: &str) -> Result<(String, String)> {
    if let Some(colon_pos) = import_path.find(':') {
        let module_path = import_path[..colon_pos].to_string();
        let symbol_name = import_path[colon_pos + 1..].to_string();
        Ok((module_path, symbol_name))
    } else {
        Err(anyhow!(
            "Invalid import path format. Expected 'module:symbol'"
        ))
    }
}

fn extract_signature_from_local(
    package_path: &Path,
    module_path: &str,
    symbol_name: &str,
) -> Result<SignatureInfo> {
    let parser = AstParser::new();

    // Try to find the main entry point
    let package_json_path = package_path.join("package.json");
    let main_file = if package_json_path.exists() {
        let package_json = std::fs::read_to_string(&package_json_path)?;
        let package_data: serde_json::Value = serde_json::from_str(&package_json)?;

        // Check for main, index, or types field
        package_data
            .get("main")
            .or_else(|| package_data.get("types"))
            .or_else(|| package_data.get("typings"))
            .and_then(|v| v.as_str())
            .unwrap_or("index.js")
            .to_string()
    } else {
        "index.js".to_string()
    };

    // List of files to try parsing for signature discovery
    let mut files_to_try = vec![];
    
    // Add main entry point
    let entry_path = package_path.join(&main_file);
    if entry_path.exists() {
        files_to_try.push(entry_path);
    }

    // Add common entry points
    let common_entries = vec![
        "index.js",
        "index.ts", 
        "index.d.ts",
        "lib/index.js",
        "lib/index.ts",
        "lib/index.d.ts", 
        "src/index.js",
        "src/index.ts",
        "src/index.d.ts",
        // Express-specific paths
        "lib/express.js",
        "lib/router/index.js",
        // Lodash-specific paths  
        "index.js",
        "lodash.js"
    ];

    for entry in common_entries {
        let path = package_path.join(entry);
        if path.exists() && !files_to_try.contains(&path) {
            files_to_try.push(path);
        }
    }

    // Try to find files that might contain the symbol
    if let Ok(entries) = std::fs::read_dir(package_path.join("lib")) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.to_lowercase().contains(&symbol_name.to_lowercase()) && 
                   (name.ends_with(".js") || name.ends_with(".ts") || name.ends_with(".d.ts")) {
                    files_to_try.push(entry.path());
                }
            }
        }
    }

    // Search through each file for the symbol
    for file_path in files_to_try {
        if let Ok(module_info) = parser.parse_file(&file_path) {
            // Check direct exports first
            if module_info.exports.contains(&symbol_name.to_string()) {
                // Look for the symbol in functions, classes, etc.
                if let Some(signature) = find_symbol_in_module(&module_info, symbol_name) {
                    return Ok(signature);
                }
            }

            // Check all symbols even if not explicitly exported (for popular packages)
            if let Some(signature) = find_symbol_in_module(&module_info, symbol_name) {
                return Ok(signature);
            }
        }
    }

    // Try TypeScript definition files more aggressively
    if let Ok(entries) = std::fs::read_dir(package_path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".d.ts") {
                    if let Ok(ts_parser) = crate::parser::typescript::TypeScriptParser::new().parse_declaration_file(&entry.path()) {
                        if let Some(signature) = find_symbol_in_module(&ts_parser, symbol_name) {
                            return Ok(signature);
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!(
        "Symbol '{}' not found in module '{}'",
        symbol_name,
        module_path
    ))
}

fn find_symbol_in_module(module_info: &crate::module_info::NodeModuleInfo, symbol_name: &str) -> Option<SignatureInfo> {
    // Look for the symbol in functions
    for function in &module_info.functions {
        if function.name == symbol_name {
            return Some(SignatureInfo {
                name: function.name.clone(),
                kind: crate::module_info::SignatureKind::Function,
                parameters: function.parameters.clone(),
                return_type: function.return_type.clone(),
                doc_comment: function.doc_comment.clone(),
            });
        }
    }

    // Look for the symbol in classes
    for class in &module_info.classes {
        if class.name == symbol_name {
            // Return constructor signature for classes
            if let Some(constructor) = &class.constructor {
                return Some(SignatureInfo {
                    name: class.name.clone(),
                    kind: crate::module_info::SignatureKind::Constructor,
                    parameters: constructor.parameters.clone(),
                    return_type: Some(class.name.clone()),
                    doc_comment: class.doc_comment.clone(),
                });
            } else {
                return Some(SignatureInfo {
                    name: class.name.clone(),
                    kind: crate::module_info::SignatureKind::Constructor,
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
                    kind: crate::module_info::SignatureKind::Method,
                    parameters: method.parameters.clone(),
                    return_type: method.return_type.clone(),
                    doc_comment: method.doc_comment.clone(),
                });
            }
        }
    }

    // Look for the symbol in constants/exports
    for constant in &module_info.constants {
        if constant.name == symbol_name {
            return Some(SignatureInfo {
                name: constant.name.clone(),
                kind: crate::module_info::SignatureKind::Function, // Treat as function for now
                parameters: Vec::new(),
                return_type: constant.value_type.clone(),
                doc_comment: constant.doc_comment.clone(),
            });
        }
    }

    None
}
