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

    /// CLI command configuration (optional)
    /// When present, registers the plugin as a top-level CLI command
    #[serde(default)]
    pub cli: Option<CliConfig>,

    /// Capabilities this plugin provides (for cocoon routing)
    /// Auto-discovered capabilities for hybrid cloud routing
    #[serde(default)]
    pub capabilities: Vec<CapabilityDeclaration>,
}

/// CLI command configuration for plugins that provide top-level commands.
///
/// When a plugin has a `[cli]` section, it will be registered as a
/// direct subcommand of the `adi` CLI (e.g., `adi tasks`, `adi lint`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// The command name (e.g., "tasks", "lint")
    /// Must be lowercase alphanumeric with hyphens
    pub command: String,

    /// Human-readable description for --help output
    pub description: String,

    /// Optional short aliases (e.g., ["t"] for "tasks")
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Enable dynamic shell completions for this command.
    /// When true, the shell will call `adi <command> --completions <position> [args...]`
    /// to get completion suggestions. The plugin should output tab-separated
    /// completion\tdescription pairs, one per line.
    #[serde(default)]
    pub dynamic_completions: bool,
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

/// Capability declaration for hybrid cloud routing.
///
/// Capabilities are advertised to the signaling server, allowing cocoons
/// to discover and request services from each other (e.g., embeddings, LLM chat).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDeclaration {
    /// Protocol/capability name (e.g., "tasks", "embeddings", "llm.chat")
    pub protocol: String,

    /// Semantic version (e.g., "1.0.0", "2.3.1")
    pub version: String,

    /// Human-readable description (optional)
    #[serde(default)]
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cocoon_manifest() {
        let toml = r#"[plugin]
id = "adi.cocoon"
name = "Cocoon"
version = "0.1.2"
type = "core"
author = "ADI Team"
description = "Remote containerized worker with PTY support and signaling server connectivity"
min_host_version = "0.8.0"

[cli]
command = "cocoon"
description = "Containerized worker for remote command execution"
aliases = []

[[provides]]
id = "adi.cocoon.cli"
version = "1.0.0"
description = "CLI commands for cocoon management"

[binary]
name = "libcocoon"

[tags]
categories = ["remote", "execution", "terminal", "pty"]
"#;
        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.plugin.id, "adi.cocoon");
        assert_eq!(manifest.plugin.plugin_type, "core");
    }

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

    #[test]
    fn test_cli_config() {
        let toml = r#"
[plugin]
id = "adi.tasks"
name = "ADI Tasks"
version = "1.0.0"
type = "core"

[cli]
command = "tasks"
description = "Task management with dependency tracking"
aliases = ["t"]

[binary]
name = "adi_tasks_plugin"
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert!(manifest.cli.is_some());
        let cli = manifest.cli.unwrap();
        assert_eq!(cli.command, "tasks");
        assert_eq!(cli.description, "Task management with dependency tracking");
        assert_eq!(cli.aliases, vec!["t"]);
    }

    #[test]
    fn test_no_cli_config() {
        let toml = r#"
[plugin]
id = "adi.embed"
name = "ADI Embed"
version = "1.0.0"
type = "core"

[binary]
name = "adi_embed_plugin"
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert!(manifest.cli.is_none());
    }

    #[test]
    fn test_capabilities() {
        let toml = r#"
[plugin]
id = "adi.tasks"
name = "ADI Tasks"
version = "1.0.0"
type = "core"

[[capabilities]]
protocol = "tasks"
version = "1.0.0"
description = "Task management API"

[[capabilities]]
protocol = "tasks.execute"
version = "1.0.0"
description = "Task execution capability"

[binary]
name = "adi_tasks_plugin"
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.capabilities.len(), 2);
        assert_eq!(manifest.capabilities[0].protocol, "tasks");
        assert_eq!(manifest.capabilities[0].version, "1.0.0");
        assert_eq!(manifest.capabilities[0].description, "Task management API");
        assert_eq!(manifest.capabilities[1].protocol, "tasks.execute");
        assert_eq!(manifest.capabilities[1].version, "1.0.0");
    }
}
