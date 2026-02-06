//! Multi-plugin package manifest (package.toml).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::error::ManifestError;
use crate::platform::{current_platform, library_filename};
use crate::plugin::{
    BinaryInfo, CompatibilityInfo, ConfigInfo, PluginManifest, PluginMeta, ServiceDeclaration,
    ServiceRequirement, SignatureInfo,
};

/// A multi-plugin package manifest parsed from package.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package metadata
    pub package: PackageMeta,

    /// Compatibility information (shared by all plugins)
    #[serde(default)]
    pub compatibility: CompatibilityInfo,

    /// Plugins in this package
    pub plugins: Vec<PluginDef>,

    /// Binary information (shared checksums)
    #[serde(default)]
    pub binary: PackageBinaryInfo,

    /// Signature information (optional)
    #[serde(default)]
    pub signature: Option<SignatureInfo>,
}

impl PackageManifest {
    /// Parse from TOML string.
    pub fn from_toml(content: &str) -> Result<Self, ManifestError> {
        toml::from_str(content).map_err(ManifestError::TomlParse)
    }

    /// Parse from file.
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Expand package into individual PluginManifest instances.
    ///
    /// Each plugin in the package gets its own manifest with inherited
    /// compatibility and signature information.
    pub fn expand_plugins(&self) -> Vec<PluginManifest> {
        self.plugins
            .iter()
            .map(|plugin_def| {
                let mut checksums = HashMap::new();
                // Copy package checksums for this plugin's binary
                for (platform, checksum) in &self.binary.checksums {
                    checksums.insert(platform.clone(), checksum.clone());
                }

                // Merge plugin-specific depends_on with package compatibility
                let mut compatibility = self.compatibility.clone();
                if !plugin_def.depends_on.is_empty() {
                    compatibility.depends_on = plugin_def.depends_on.clone();
                }

                PluginManifest {
                    plugin: PluginMeta {
                        id: plugin_def.id.clone(),
                        name: plugin_def.name.clone(),
                        version: self.package.version.clone(),
                        plugin_type: plugin_def.plugin_type.clone(),
                        author: self.package.author.clone(),
                        description: plugin_def
                            .description
                            .clone()
                            .unwrap_or_else(|| self.package.description.clone()),
                        license: self.package.license.clone(),
                        homepage: self.package.homepage.clone(),
                    },
                    compatibility,
                    binary: BinaryInfo {
                        name: plugin_def.binary.clone(),
                        checksums,
                    },
                    signature: self.signature.clone(),
                    config: plugin_def.config.clone().unwrap_or_default(),
                    provides: plugin_def.provides.clone(),
                    requires: plugin_def.requires.clone(),
                    // Packages don't support CLI commands - only single plugins do
                    cli: None,
                    // Packages don't support capabilities - only single plugins do
                    capabilities: Vec::new(),
                    tags: None,
                    hive: None,
                    translation: None,
                    language: None,
                    requirements: None,
                }
            })
            .collect()
    }

    /// Get the installation order of plugins, respecting dependencies.
    ///
    /// Returns plugins sorted so that dependencies come before dependents.
    /// Returns an error if there are circular dependencies.
    pub fn install_order(&self) -> Result<Vec<&PluginDef>, ManifestError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        // Build a map of plugin id -> plugin def
        let plugin_map: HashMap<&str, &PluginDef> =
            self.plugins.iter().map(|p| (p.id.as_str(), p)).collect();

        fn visit<'a>(
            plugin_id: &str,
            plugin_map: &HashMap<&str, &'a PluginDef>,
            visited: &mut HashSet<String>,
            in_progress: &mut HashSet<String>,
            result: &mut Vec<&'a PluginDef>,
        ) -> Result<(), ManifestError> {
            if visited.contains(plugin_id) {
                return Ok(());
            }

            if in_progress.contains(plugin_id) {
                return Err(ManifestError::CircularDependency(plugin_id.to_string()));
            }

            in_progress.insert(plugin_id.to_string());

            if let Some(plugin) = plugin_map.get(plugin_id) {
                for dep in &plugin.depends_on {
                    visit(dep, plugin_map, visited, in_progress, result)?;
                }

                in_progress.remove(plugin_id);
                visited.insert(plugin_id.to_string());
                result.push(plugin);
            }

            Ok(())
        }

        for plugin in &self.plugins {
            visit(
                &plugin.id,
                &plugin_map,
                &mut visited,
                &mut in_progress,
                &mut result,
            )?;
        }

        Ok(result)
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
            return true;
        }
        let current = current_platform();
        self.compatibility
            .platforms
            .iter()
            .any(|p| p == &current || p == "all")
    }
}

/// Package metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    /// Unique identifier (e.g., "vendor.theme-pack")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Version string (semver)
    pub version: String,

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

/// Plugin definition within a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDef {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Plugin type
    #[serde(rename = "type")]
    pub plugin_type: String,

    /// Binary name (without lib prefix and extension)
    pub binary: String,

    /// Description (optional, inherits from package)
    #[serde(default)]
    pub description: Option<String>,

    /// Dependencies on other plugins in this package
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Plugin-specific configuration
    #[serde(default)]
    pub config: Option<ConfigInfo>,

    /// Services this plugin provides
    #[serde(default)]
    pub provides: Vec<ServiceDeclaration>,

    /// Services this plugin requires
    #[serde(default)]
    pub requires: Vec<ServiceRequirement>,
}

impl PluginDef {
    /// Get the binary filename for the current platform.
    pub fn binary_filename(&self) -> String {
        library_filename(&self.binary)
    }
}

/// Package binary information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageBinaryInfo {
    /// SHA256 checksums per platform (for the whole package archive)
    #[serde(default)]
    pub checksums: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_manifest() {
        let toml = r#"
[package]
id = "vendor.theme-pack"
name = "Theme Collection"
version = "2.0.0"
author = "Vendor Team"
description = "A collection of themes"

[compatibility]
api_version = 1
min_host_version = "0.8.0"

[[plugins]]
id = "vendor.theme-dark"
name = "Dark Theme"
type = "theme"
binary = "dark_theme"

[[plugins]]
id = "vendor.theme-light"
name = "Light Theme"
type = "theme"
binary = "light_theme"

[[plugins]]
id = "vendor.theme-custom"
name = "Custom Theme Builder"
type = "extension"
binary = "custom_builder"
depends_on = ["vendor.theme-dark"]

[binary.checksums]
darwin-aarch64 = "sha256:abc123"
"#;

        let manifest = PackageManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.package.id, "vendor.theme-pack");
        assert_eq!(manifest.package.name, "Theme Collection");
        assert_eq!(manifest.plugins.len(), 3);
    }

    #[test]
    fn test_expand_plugins() {
        let toml = r#"
[package]
id = "vendor.pack"
name = "Test Pack"
version = "1.0.0"

[[plugins]]
id = "vendor.plugin-a"
name = "Plugin A"
type = "extension"
binary = "plugin_a"

[[plugins]]
id = "vendor.plugin-b"
name = "Plugin B"
type = "theme"
binary = "plugin_b"
"#;

        let manifest = PackageManifest::from_toml(toml).unwrap();
        let expanded = manifest.expand_plugins();

        assert_eq!(expanded.len(), 2);
        assert_eq!(expanded[0].plugin.id, "vendor.plugin-a");
        assert_eq!(expanded[1].plugin.id, "vendor.plugin-b");
        // All inherit package version
        assert_eq!(expanded[0].plugin.version, "1.0.0");
        assert_eq!(expanded[1].plugin.version, "1.0.0");
    }

    #[test]
    fn test_install_order() {
        let toml = r#"
[package]
id = "vendor.pack"
name = "Test Pack"
version = "1.0.0"

[[plugins]]
id = "vendor.plugin-c"
name = "Plugin C"
type = "extension"
binary = "plugin_c"
depends_on = ["vendor.plugin-a", "vendor.plugin-b"]

[[plugins]]
id = "vendor.plugin-a"
name = "Plugin A"
type = "extension"
binary = "plugin_a"

[[plugins]]
id = "vendor.plugin-b"
name = "Plugin B"
type = "extension"
binary = "plugin_b"
depends_on = ["vendor.plugin-a"]
"#;

        let manifest = PackageManifest::from_toml(toml).unwrap();
        let order = manifest.install_order().unwrap();

        // A must come before B (B depends on A)
        // A and B must come before C (C depends on both)
        let ids: Vec<&str> = order.iter().map(|p| p.id.as_str()).collect();
        let pos_a = ids.iter().position(|&id| id == "vendor.plugin-a").unwrap();
        let pos_b = ids.iter().position(|&id| id == "vendor.plugin-b").unwrap();
        let pos_c = ids.iter().position(|&id| id == "vendor.plugin-c").unwrap();

        assert!(pos_a < pos_b, "A should come before B");
        assert!(pos_a < pos_c, "A should come before C");
        assert!(pos_b < pos_c, "B should come before C");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let toml = r#"
[package]
id = "vendor.pack"
name = "Test Pack"
version = "1.0.0"

[[plugins]]
id = "vendor.plugin-a"
name = "Plugin A"
type = "extension"
binary = "plugin_a"
depends_on = ["vendor.plugin-b"]

[[plugins]]
id = "vendor.plugin-b"
name = "Plugin B"
type = "extension"
binary = "plugin_b"
depends_on = ["vendor.plugin-a"]
"#;

        let manifest = PackageManifest::from_toml(toml).unwrap();
        let result = manifest.install_order();

        assert!(result.is_err());
        assert!(matches!(result, Err(ManifestError::CircularDependency(_))));
    }
}
