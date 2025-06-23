use crate::module_info::NodeModuleInfo;
use crate::npm_client::NpmClient;
use crate::parser::ast_parser::AstParser;
use crate::parser::typescript::TypeScriptParser;
use crate::utils::{is_dts_file, is_js_file, parse_package_spec};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct NodeModuleExplorer {
    package_name: String,
    max_depth: usize,
    quiet: bool,
    npm_client: NpmClient,
    ast_parser: AstParser,
    ts_parser: TypeScriptParser,
}

impl NodeModuleExplorer {
    pub fn new(package_name: String, max_depth: usize, quiet: bool) -> Self {
        Self {
            package_name,
            max_depth,
            quiet,
            npm_client: NpmClient::new(),
            ast_parser: AstParser::new(),
            ts_parser: TypeScriptParser::new(),
        }
    }

    /// Explore the package and return module information
    pub async fn explore(&self) -> Result<NodeModuleInfo> {
        // Parse package specification
        let (package_name, version) = parse_package_spec(&self.package_name);

        // Try to find locally installed package first
        let search_paths = vec![
            env::current_dir()?,
            env::current_dir()?.join(".."),
            env::current_dir()?.join("../.."),
        ];

        if let Some(local_path) = self
            .npm_client
            .find_local_package(&package_name, &search_paths)
        {
            if !self.quiet {
                eprintln!("📦 Using locally installed {}", package_name);
            }
            return self.explore_local_package(&local_path, &package_name).await;
        }

        // Download package from npm
        let package_info = self
            .npm_client
            .get_package_info(&package_name, version.as_deref())
            .await?;
        let temp_dir = self
            .npm_client
            .download_package(&package_info, self.quiet)
            .await?;

        let package_path = temp_dir.path().join("package");
        self.explore_local_package(&package_path, &package_name)
            .await
    }

    async fn explore_local_package(
        &self,
        package_path: &Path,
        package_name: &str,
    ) -> Result<NodeModuleInfo> {
        let mut root_module = NodeModuleInfo::new(package_name.to_string());

        // Read package.json for metadata
        let package_json_path = package_path.join("package.json");
        if package_json_path.exists() {
            self.parse_package_json(&package_json_path, &mut root_module)?;
        }

        // Find entry points
        let entry_points = self.find_entry_points(package_path, &root_module)?;

        // Parse main entry point
        if let Some(main_entry) = entry_points.get("main") {
            if let Ok(module_info) = self.parse_file(main_entry) {
                // Copy information from parsed entry point
                root_module.exports.extend(module_info.exports);
                root_module.functions.extend(module_info.functions);
                root_module.classes.extend(module_info.classes);
                root_module.types.extend(module_info.types);
                root_module.constants.extend(module_info.constants);
            }
        }

        // Explore submodules if depth allows
        if self.max_depth > 1 {
            self.explore_submodules(package_path, &mut root_module, 1)
                .await?;
        }

        Ok(root_module)
    }

    fn parse_package_json(
        &self,
        package_json_path: &Path,
        module_info: &mut NodeModuleInfo,
    ) -> Result<()> {
        let content = fs::read_to_string(package_json_path)?;
        let package_data: serde_json::Value = serde_json::from_str(&content)?;

        // Extract version
        if let Some(version) = package_data.get("version").and_then(|v| v.as_str()) {
            module_info.version = Some(version.to_string());
        }

        // Extract main entry point
        if let Some(main) = package_data.get("main").and_then(|v| v.as_str()) {
            module_info.main = Some(main.to_string());
        }

        Ok(())
    }

    fn find_entry_points(
        &self,
        package_path: &Path,
        module_info: &NodeModuleInfo,
    ) -> Result<HashMap<String, PathBuf>> {
        let mut entry_points = HashMap::new();

        // Check package.json main field
        if let Some(main) = &module_info.main {
            let main_path = package_path.join(main);
            if main_path.exists() {
                entry_points.insert("main".to_string(), main_path);
            } else {
                // Try with .js extension
                let main_js = main_path.with_extension("js");
                if main_js.exists() {
                    entry_points.insert("main".to_string(), main_js);
                }
            }
        }

        // Fallback to common entry points
        if entry_points.is_empty() {
            let common_entries = vec![
                "index.js",
                "index.ts",
                "index.d.ts",
                "lib/index.js",
                "lib/index.ts",
                "src/index.js",
                "src/index.ts",
                "dist/index.js",
                "build/index.js",
            ];

            for entry in common_entries {
                let entry_path = package_path.join(entry);
                if entry_path.exists() {
                    entry_points.insert("main".to_string(), entry_path);
                    break;
                }
            }
        }

        // Look for TypeScript definitions
        let ts_entries = vec![
            "index.d.ts",
            "lib/index.d.ts",
            "types/index.d.ts",
            "dist/index.d.ts",
        ];

        for entry in ts_entries {
            let entry_path = package_path.join(entry);
            if entry_path.exists() {
                entry_points.insert("types".to_string(), entry_path);
                break;
            }
        }

        Ok(entry_points)
    }

    async fn explore_submodules(
        &self,
        package_path: &Path,
        parent_module: &mut NodeModuleInfo,
        current_depth: usize,
    ) -> Result<()> {
        if current_depth >= self.max_depth {
            return Ok(());
        }

        // Look for common subdirectories that might contain modules
        let subdirs_to_check = vec!["lib", "src", "dist", "build", "types"];

        for subdir in subdirs_to_check {
            let subdir_path = package_path.join(subdir);
            if subdir_path.exists() && subdir_path.is_dir() {
                self.explore_directory(&subdir_path, parent_module, current_depth)
                    .await?;
            }
        }

        Ok(())
    }

    async fn explore_directory(
        &self,
        dir_path: &Path,
        parent_module: &mut NodeModuleInfo,
        current_depth: usize,
    ) -> Result<()> {
        if current_depth >= self.max_depth {
            return Ok(());
        }

        for entry in WalkDir::new(dir_path)
            .max_depth(2) // Limit filesystem traversal depth
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() && (is_js_file(path) || is_dts_file(path)) {
                // Skip if it's the same as main entry point
                if let Some(main) = &parent_module.main {
                    if path.ends_with(main) {
                        continue;
                    }
                }

                if let Ok(module_info) = self.parse_file(path) {
                    let module_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    parent_module.add_submodule(module_name, module_info);
                }
            }
        }

        Ok(())
    }

    fn parse_file(&self, file_path: &Path) -> Result<NodeModuleInfo> {
        if is_dts_file(file_path) {
            self.ts_parser.parse_declaration_file(file_path)
        } else if is_js_file(file_path) {
            self.ast_parser.parse_file(file_path)
        } else {
            Err(anyhow!("Unsupported file type: {:?}", file_path))
        }
    }
}
