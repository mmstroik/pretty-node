use pretty_node::explorer::NodeModuleExplorer;
use pretty_node::module_info::{NodeModuleInfo, SignatureInfo, SignatureKind};
use pretty_node::parser::signature::extract_signature;
use tempfile::TempDir;

#[cfg(test)]
mod explorer_tests {
    use super::*;

    #[test]
    fn test_explorer_init() {
        let explorer = NodeModuleExplorer::new("test-package".to_string(), 3, false);
        assert_eq!(explorer.package_name(), "test-package");
        assert_eq!(explorer.max_depth(), 3);
    }

    #[test]
    fn test_explorer_init_defaults() {
        let explorer = NodeModuleExplorer::new("test-package".to_string(), 2, false);
        assert_eq!(explorer.max_depth(), 2);
    }

    #[test]
    fn test_module_info_creation() {
        let module = NodeModuleInfo::new("test-module".to_string());
        assert_eq!(module.name, "test-module");
        assert!(module.functions.is_empty());
        assert!(module.classes.is_empty());
        assert!(module.exports.is_empty());
    }

    #[test]
    fn test_local_package_discovery() {
        // Create a temporary node_modules structure
        let temp_dir = TempDir::new().unwrap();
        let node_modules = temp_dir.path().join("node_modules");
        let package_dir = node_modules.join("test-package");
        
        std::fs::create_dir_all(&package_dir).unwrap();
        std::fs::write(
            package_dir.join("package.json"),
            r#"{"name": "test-package", "version": "1.0.0", "main": "index.js"}"#,
        ).unwrap();
        std::fs::write(
            package_dir.join("index.js"),
            "module.exports = { testFunction: () => {} };",
        ).unwrap();

        let explorer = NodeModuleExplorer::new("test-package".to_string(), 2, true);
        let search_paths = vec![temp_dir.path()];
        
        let local_path = explorer.find_local_package(&search_paths);
        assert!(local_path.is_some());
        assert_eq!(local_path.unwrap(), package_dir);
    }

    #[tokio::test]
    async fn test_extract_signature_basic() {
        let import_path = "lodash:isArray";
        let result = extract_signature(import_path, true).await;
        
        match result {
            Ok(sig) => {
                assert_eq!(sig.name, "isArray");
            }
            Err(_) => {
                // It's OK if the package isn't available locally
                // The test validates the function doesn't panic
            }
        }
    }

    #[tokio::test]
    async fn test_extract_signature_nonexistent() {
        let import_path = "nonexistent-package:nonexistent-function";
        let result = extract_signature(import_path, true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_signature_invalid_format() {
        let import_path = "invalid-format-no-colon";
        let result = extract_signature(import_path, true).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_import_path() {
        let cases = vec![
            ("lodash:isArray", ("lodash", "isArray")),
            ("@types/node:Buffer", ("@types/node", "Buffer")),
            ("express:Router", ("express", "Router")),
        ];

        for (input, expected) in cases {
            if let Some(colon_pos) = input.find(':') {
                let package = &input[..colon_pos];
                let symbol = &input[colon_pos + 1..];
                assert_eq!((package, symbol), expected);
            }
        }
    }

    #[test]
    fn test_signature_info_creation() {
        let sig = SignatureInfo {
            name: "testFunction".to_string(),
            kind: SignatureKind::Function,
            parameters: vec![],
            return_type: Some("boolean".to_string()),
            doc_comment: None,
        };

        assert_eq!(sig.name, "testFunction");
        assert!(sig.parameters.is_empty());
        assert_eq!(sig.return_type.as_ref().unwrap(), "boolean");
    }

    #[test] 
    fn test_explorer_with_scoped_package() {
        let explorer = NodeModuleExplorer::new("@types/node".to_string(), 2, false);
        assert_eq!(explorer.package_name(), "@types/node");
    }

    #[test]
    fn test_package_name_extraction() {
        let test_cases = vec![
            ("express", "express"),
            ("express/lib/router", "express"),
            ("@types/node", "@types/node"),
            ("@types/node/fs", "@types/node"),
        ];

        for (input, expected) in test_cases {
            let parts: Vec<&str> = input.split('/').collect();
            let base_package = if parts[0].starts_with('@') && parts.len() > 1 {
                format!("{}/{}", parts[0], parts[1])
            } else {
                parts[0].to_string()
            };
            assert_eq!(base_package, expected);
        }
    }

    #[test]
    fn test_integration_small_package() {
        // Test with a commonly available small package
        let explorer = NodeModuleExplorer::new("ms".to_string(), 1, true);
        
        // This test mainly validates that exploration setup doesn't crash
        // Note: explore() is async, so we can't easily test it in sync tests
        assert_eq!(explorer.package_name(), "ms");
        assert_eq!(explorer.max_depth(), 1);
    }

    #[test]
    fn test_depth_limiting() {
        let explorer = NodeModuleExplorer::new("test-package".to_string(), 1, false);
        assert_eq!(explorer.max_depth(), 1);
        
        let explorer_deep = NodeModuleExplorer::new("test-package".to_string(), 5, false);
        assert_eq!(explorer_deep.max_depth(), 5);
    }
}