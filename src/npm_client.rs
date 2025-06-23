use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::process::Command;

#[derive(Debug)]
pub struct NpmPackageInfo {
    pub name: String,
    pub version: String,
    pub tarball_url: String,
    pub main: Option<String>,
    pub types: Option<String>,
}

pub struct NpmClient {
    client: Client,
    registry_url: String,
}

impl Default for NpmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NpmClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            registry_url: "https://registry.npmjs.org".to_string(),
        }
    }
    
    /// Fetch package metadata from npm registry
    pub async fn get_package_info(&self, package_name: &str, version: Option<&str>) -> Result<NpmPackageInfo> {
        let url = if let Some(v) = version {
            format!("{}/{}@{}", self.registry_url, package_name, v)
        } else {
            format!("{}/{}", self.registry_url, package_name)
        };
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Package '{}' not found in npm registry", package_name));
        }
        
        let data: Value = response.json().await?;
        
        // Handle both package@version and latest package responses
        let pkg_data = if let Some(versions) = data.get("versions") {
            // Response contains all versions, get the latest or specified one
            if let Some(v) = version {
                versions.get(v).ok_or_else(|| anyhow!("Version {} not found", v))?
            } else {
                let latest_version = data["dist-tags"]["latest"].as_str()
                    .ok_or_else(|| anyhow!("No latest version found"))?;
                versions.get(latest_version)
                    .ok_or_else(|| anyhow!("Latest version not found in versions"))?
            }
        } else {
            // Response is for a specific version
            &data
        };
        
        let name = pkg_data["name"].as_str()
            .ok_or_else(|| anyhow!("No name field in package info"))?;
        let version = pkg_data["version"].as_str()
            .ok_or_else(|| anyhow!("No version field in package info"))?;
        let tarball_url = pkg_data["dist"]["tarball"].as_str()
            .ok_or_else(|| anyhow!("No tarball URL in package info"))?;
        
        let main = pkg_data.get("main").and_then(|v| v.as_str()).map(|s| s.to_string());
        let types = pkg_data.get("types")
            .or_else(|| pkg_data.get("typings"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(NpmPackageInfo {
            name: name.to_string(),
            version: version.to_string(),
            tarball_url: tarball_url.to_string(),
            main,
            types,
        })
    }
    
    /// Download and extract package to a temporary directory
    pub async fn download_package(&self, package_info: &NpmPackageInfo, quiet: bool) -> Result<TempDir> {
        if !quiet {
            eprintln!("📦 Downloading package {}@{}", package_info.name, package_info.version);
        }
        
        // Download tarball
        let response = self.client.get(&package_info.tarball_url).send().await?;
        let bytes = response.bytes().await?;
        
        // Create temp directory
        let temp_dir = TempDir::new()?;
        let tarball_path = temp_dir.path().join("package.tgz");
        
        // Write tarball to temp file
        fs::write(&tarball_path, bytes)?;
        
        // Extract tarball
        let tar_gz = fs::File::open(&tarball_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(temp_dir.path())?;
        
        Ok(temp_dir)
    }
    
    /// Check if package is locally installed in node_modules
    pub fn find_local_package<P: AsRef<Path>>(&self, package_name: &str, search_paths: &[P]) -> Option<PathBuf> {
        for search_path in search_paths {
            let node_modules = search_path.as_ref().join("node_modules");
            let package_path = node_modules.join(package_name);
            
            if package_path.exists() && package_path.is_dir() {
                return Some(package_path);
            }
        }
        None
    }
    
    /// Try to install package locally using npm
    pub async fn install_package_locally(&self, package_spec: &str, quiet: bool) -> Result<()> {
        if !quiet {
            eprintln!("📦 Installing package {}", package_spec);
        }
        
        let output = Command::new("npm")
            .args(&["install", "--no-save", package_spec])
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to install package: {}", stderr));
        }
        
        Ok(())
    }
}