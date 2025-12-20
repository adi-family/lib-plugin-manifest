//! Single plugin manifest (plugin.toml).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::ManifestError;
use crate::platform::{current_platform, library_filename};

/// A single plugin manifest parsed from plugin.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginMeta,

    /// Compatibility information
    #[serde(default)]
    pub compatibility: CompatibilityInfo,

    /// Binary information
    #[serde(default)]
    pub binary: BinaryInfo,

    /// Signature information (optional)
    #[serde(default)]
    pub signature: Option<SignatureInfo>,

    /// Default configuration values
    #[serde(default)]
    pub config: ConfigInfo,

    /// Services this plugin provides
    #[serde(default)]
    pub provides: Vec<ServiceDeclaration>,

    /// Services this plugin requires
    #[serde(default)]
    pub requires: Vec<ServiceRequirement>,
}

impl PluginManifest {
    /// Parse from TOML string.
    pub fn from_toml(content: &str) -> Result<Self, ManifestError> {
        toml::from_str(content).map_err(ManifestError::TomlParse)
    }

    /// Parse from file.
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Get the binary filename for the current platform.
    pub fn binary_filename(&self) -> String {
        library_filename(&self.binary.name)
    }

    /// Get the checksum for the current platform (if available).
    pub fn checksum_for_current_platform(&self) -> Option<&str> {
        self.binary
            .checksums
            .get(&current_platform())
            .map(|s| s.as_str())
    }

    /// Check if the current platform is supported.
    pub fn supports_current_platform(&self) -> bool {
        if self.compatibility.platforms.is_empty() {
            return true; // No platform restriction
        }
        let current = current_platform();
        self.compatibility
            .platforms
            .iter()
            .any(|p| p == &current || p == "all")
    }
}

/// Plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// Unique identifier (e.g., "vendor.plugin-name")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Version string (semver)
    pub version: String,

    /// Plugin type (e.g., "theme", "extension", "font")
    #[serde(rename = "type")]
    pub plugin_type: String,

    /// Author
    #[serde(default)]
    pub author: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// License identifier (SPDX)
    #[serde(default)]
    pub license: Option<String>,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Compatibility information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    /// Plugin API version
    #[serde(default = "default_api_version")]
    pub api_version: u32,

    /// Minimum host version required
    #[serde(default)]
    pub min_host_version: Option<String>,

    /// Maximum host version (optional)
    #[serde(default)]
    pub max_host_version: Option<String>,

    /// Supported platforms (empty = all platforms)
    #[serde(default)]
    pub platforms: Vec<String>,

    /// Plugin dependencies (other plugin IDs that must be loaded first)
    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl Default for CompatibilityInfo {
    fn default() -> Self {
        Self {
            api_version: default_api_version(),
            min_host_version: None,
            max_host_version: None,
            platforms: Vec::new(),
            depends_on: Vec::new(),
        }
    }
}

fn default_api_version() -> u32 {
    2 // Match PLUGIN_API_VERSION in lib-plugin-abi
}

/// Binary information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    /// Binary name (without lib prefix and extension)
    #[serde(default = "default_binary_name")]
    pub name: String,

    /// SHA256 checksums per platform
    #[serde(default)]
    pub checksums: HashMap<String, String>,
}

fn default_binary_name() -> String {
    "plugin".to_string()
}

impl Default for BinaryInfo {
    fn default() -> Self {
        Self {
            name: default_binary_name(),
            checksums: HashMap::new(),
        }
    }
}

/// Signature information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    /// Ed25519 public key (base64 encoded)
    pub public_key: String,

    /// Signature file path (relative to manifest)
    pub signature_file: String,
}

/// Default configuration values.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigInfo {
    /// Default configuration values
    #[serde(default)]
    pub defaults: HashMap<String, toml::Value>,
}

/// Service provided by this plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeclaration {
    /// Service ID (e.g., "adi.indexer.search")
    pub id: String,

    /// Service version (semver)
    pub version: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,
}

/// Service required by this plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequirement {
    /// Required service ID
    pub id: String,

    /// Minimum version required (optional)
    #[serde(default)]
    pub min_version: Option<String>,

    /// Whether this requirement is optional (defaults to false = required)
    #[serde(default)]
    pub optional: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin_manifest() {
        let toml = r#"
[plugin]
id = "vendor.test-plugin"
name = "Test Plugin"
version = "1.0.0"
type = "extension"
author = "Test Author"

[compatibility]
api_version = 1
min_host_version = "0.8.0"
platforms = ["darwin-aarch64", "linux-x86_64"]

[binary]
name = "test_plugin"
[binary.checksums]
darwin-aarch64 = "sha256:abc123"

[config.defaults]
enabled = true
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.plugin.id, "vendor.test-plugin");
        assert_eq!(manifest.plugin.name, "Test Plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert_eq!(manifest.plugin.plugin_type, "extension");
        assert_eq!(manifest.compatibility.api_version, 1);
        assert_eq!(manifest.binary.name, "test_plugin");
    }

    #[test]
    fn test_binary_filename() {
        let toml = r#"
[plugin]
id = "test.plugin"
name = "Test"
version = "1.0.0"
type = "test"

[binary]
name = "my_plugin"
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        let filename = manifest.binary_filename();
        assert!(filename.contains("my_plugin"));
    }
}
