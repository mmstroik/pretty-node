use std::path::Path;

/// Parse package specification (e.g., "express@4.18.0", "@types/node", "lodash")
pub fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if let Some(at_pos) = spec.rfind('@') {
        // Handle scoped packages like @types/node@1.0.0
        if spec.starts_with('@') {
            // Find the second @ for version
            if let Some(second_at) = spec[1..].find('@') {
                let version_at = second_at + 1;
                let package_name = spec[..=version_at].to_string();
                let version = spec[version_at + 1..].to_string();
                return (package_name, Some(version));
            }
        } else {
            // Regular package like express@4.18.0
            let package_name = spec[..at_pos].to_string();
            let version = spec[at_pos + 1..].to_string();
            return (package_name, Some(version));
        }
    }
    
    (spec.to_string(), None)
}

/// Extract base package name from a module path (e.g., "express/lib/router" -> "express")
pub fn extract_base_package(module_path: &str) -> String {
    let parts: Vec<&str> = module_path.split('/').collect();
    
    if parts[0].starts_with('@') && parts.len() > 1 {
        // Scoped package like @types/node
        format!("{}/{}", parts[0], parts[1])
    } else {
        // Regular package
        parts[0].to_string()
    }
}

/// Check if a path is likely a JavaScript/TypeScript file
pub fn is_js_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(ext.to_str(), Some("js") | Some("mjs") | Some("ts") | Some("tsx") | Some("jsx"))
    } else {
        false
    }
}

/// Check if a path is a TypeScript definition file
pub fn is_dts_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "ts")
        .unwrap_or(false)
        && path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(".d.ts"))
            .unwrap_or(false)
}

/// Resolve import path relative to a base path
pub fn resolve_import_path(base_path: &Path, import_path: &str) -> Option<std::path::PathBuf> {
    if import_path.starts_with('.') {
        // Relative import
        Some(base_path.parent()?.join(import_path))
    } else {
        // Absolute import - would need node_modules resolution
        None
    }
}

/// Check if a string is a valid JavaScript identifier
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let mut chars = s.chars();
    
    // First character must be letter, underscore, or dollar sign
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    
    // Subsequent characters can be letters, digits, underscores, or dollar signs
    for c in chars {
        if !(c.is_alphanumeric() || c == '_' || c == '$') {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_package_spec() {
        assert_eq!(parse_package_spec("express"), ("express".to_string(), None));
        assert_eq!(parse_package_spec("express@4.18.0"), ("express".to_string(), Some("4.18.0".to_string())));
        assert_eq!(parse_package_spec("@types/node"), ("@types/node".to_string(), None));
        assert_eq!(parse_package_spec("@types/node@18.0.0"), ("@types/node".to_string(), Some("18.0.0".to_string())));
    }
    
    #[test]
    fn test_extract_base_package() {
        assert_eq!(extract_base_package("express"), "express");
        assert_eq!(extract_base_package("express/lib/router"), "express");
        assert_eq!(extract_base_package("@types/node"), "@types/node");
        assert_eq!(extract_base_package("@types/node/fs"), "@types/node");
    }
    
    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("hello"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("$jquery"));
        assert!(is_valid_identifier("test123"));
        assert!(!is_valid_identifier("123invalid"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("kebab-case"));
    }
}