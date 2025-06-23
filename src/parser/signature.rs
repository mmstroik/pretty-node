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

    let entry_path = package_path.join(&main_file);

    // If the specified file doesn't exist, try common variations
    let file_to_parse = if entry_path.exists() {
        entry_path
    } else {
        // Try index.js, index.ts, etc.
        let alternatives = vec![
            package_path.join("index.js"),
            package_path.join("index.ts"),
            package_path.join("index.d.ts"),
            package_path.join("lib/index.js"),
            package_path.join("lib/index.ts"),
            package_path.join("src/index.js"),
            package_path.join("src/index.ts"),
        ];

        alternatives
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!("Could not find entry point for package"))?
    };

    // Parse the file
    let module_info = parser.parse_file(&file_to_parse)?;

    // Look for the symbol in functions, classes, etc.
    for function in &module_info.functions {
        if function.name == symbol_name {
            return Ok(SignatureInfo {
                name: function.name.clone(),
                kind: crate::module_info::SignatureKind::Function,
                parameters: function.parameters.clone(),
                return_type: function.return_type.clone(),
                doc_comment: function.doc_comment.clone(),
            });
        }
    }

    for class in &module_info.classes {
        if class.name == symbol_name {
            // Return constructor signature for classes
            if let Some(constructor) = &class.constructor {
                return Ok(SignatureInfo {
                    name: class.name.clone(),
                    kind: crate::module_info::SignatureKind::Constructor,
                    parameters: constructor.parameters.clone(),
                    return_type: Some(class.name.clone()),
                    doc_comment: class.doc_comment.clone(),
                });
            } else {
                return Ok(SignatureInfo {
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
                return Ok(SignatureInfo {
                    name: format!("{}.{}", class.name, method.name),
                    kind: crate::module_info::SignatureKind::Method,
                    parameters: method.parameters.clone(),
                    return_type: method.return_type.clone(),
                    doc_comment: method.doc_comment.clone(),
                });
            }
        }
    }

    Err(anyhow!(
        "Symbol '{}' not found in module '{}'",
        symbol_name,
        module_path
    ))
}
