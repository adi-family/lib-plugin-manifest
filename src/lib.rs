//! Plugin manifest parsing.
//!
//! Supports both single plugin manifests (plugin.toml) and
//! multi-plugin package manifests (package.toml).
//!
//! # Single Plugin (plugin.toml)
//!
//! ```toml
//! [plugin]
//! id = "vendor.plugin-name"
//! name = "Human Readable Name"
//! version = "1.0.0"
//! type = "extension"
//!
//! [compatibility]
//! api_version = 1
//! min_host_version = "0.8.0"
//!
//! [binary]
//! name = "my_plugin"
//! ```
//!
//! # Multi-Plugin Package (package.toml)
//!
//! ```toml
//! [package]
//! id = "vendor.theme-pack"
//! name = "Theme Collection"
//! version = "2.0.0"
//!
//! [[plugins]]
//! id = "vendor.theme-dark"
//! name = "Dark Theme"
//! type = "theme"
//! binary = "dark_theme"
//! ```

mod error;
mod package;
mod platform;
mod plugin;

pub use error::*;
pub use package::*;
pub use platform::*;
pub use plugin::*;

use std::path::Path;

/// Unified manifest type that can be either a single plugin or a package.
#[derive(Debug, Clone)]
pub enum Manifest {
    /// A single plugin manifest
    Single(PluginManifest),
    /// A multi-plugin package manifest
    Package(PackageManifest),
}

impl Manifest {
    /// Parse a manifest from a TOML string, auto-detecting the type.
    pub fn from_toml(content: &str) -> Result<Self, ManifestError> {
        // Try to detect the type by checking for [plugin] vs [package]
        if content.contains("[package]") {
            Ok(Manifest::Package(PackageManifest::from_toml(content)?))
        } else if content.contains("[plugin]") {
            Ok(Manifest::Single(PluginManifest::from_toml(content)?))
        } else {
            Err(ManifestError::InvalidFormat(
                "Manifest must contain either [plugin] or [package] section".to_string(),
            ))
        }
    }

    /// Parse a manifest from a file, auto-detecting the type.
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Get all plugin IDs contained in this manifest.
    /// Returns 1 ID for single plugins, N IDs for packages.
    pub fn plugin_ids(&self) -> Vec<&str> {
        match self {
            Manifest::Single(m) => vec![m.plugin.id.as_str()],
            Manifest::Package(m) => m.plugins.iter().map(|p| p.id.as_str()).collect(),
        }
    }

    /// Get the manifest ID (plugin ID or package ID).
    pub fn id(&self) -> &str {
        match self {
            Manifest::Single(m) => &m.plugin.id,
            Manifest::Package(m) => &m.package.id,
        }
    }

    /// Get the manifest version.
    pub fn version(&self) -> &str {
        match self {
            Manifest::Single(m) => &m.plugin.version,
            Manifest::Package(m) => &m.package.version,
        }
    }

    /// Check if this is a package (multi-plugin).
    pub fn is_package(&self) -> bool {
        matches!(self, Manifest::Package(_))
    }

    /// Get CLI configuration if this is a single plugin with CLI support.
    /// Returns None for packages (they can't have CLI commands) or
    /// single plugins without a [cli] section.
    pub fn cli_config(&self) -> Option<&CliConfig> {
        match self {
            Manifest::Single(m) => m.cli.as_ref(),
            Manifest::Package(_) => None,
        }
    }
}
