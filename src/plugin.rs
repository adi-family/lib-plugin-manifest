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

    /// Tags for categorization
    #[serde(default)]
    pub tags: Option<TagsInfo>,

    /// Hive plugin metadata (for hive-plugin type)
    #[serde(default)]
    pub hive: Option<HiveInfo>,

    /// Translation plugin metadata (for translation type)
    #[serde(default)]
    pub translation: Option<TranslationInfo>,

    /// Language analyzer metadata (for lang type)
    #[serde(default)]
    pub language: Option<LanguageInfo>,

    /// Platform requirements
    #[serde(default)]
    pub requirements: Option<RequirementsInfo>,
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

/// Tags for plugin categorization and discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsInfo {
    /// Category tags (e.g., ["tasks", "workflow"])
    #[serde(default)]
    pub categories: Vec<String>,

    /// Platform tags (e.g., ["darwin-aarch64"])
    #[serde(default)]
    pub platforms: Vec<String>,
}

/// Hive plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveInfo {
    /// Plugin category within hive (e.g., "runner", "proxy", "health")
    pub category: String,

    /// Plugin name within category (e.g., "docker", "cors")
    pub name: String,
}

/// Translation plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationInfo {
    /// Plugin ID this translates (e.g., "adi.workflow")
    pub translates: String,

    /// Language code (e.g., "en-US")
    pub language: String,

    /// Human-readable language name (e.g., "English (United States)")
    pub language_name: String,

    /// Translation namespace (e.g., "workflow")
    pub namespace: String,
}

/// Language analyzer plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Language identifier (e.g., "rust", "python")
    pub id: String,

    /// File extensions (e.g., ["rs"], ["py", "pyi"])
    pub extensions: Vec<String>,
}

/// Platform requirements for the plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementsInfo {
    /// Required OS (e.g., "darwin", "linux")
    #[serde(default)]
    pub os: Option<String>,

    /// Required architecture (e.g., "aarch64")
    #[serde(default)]
    pub arch: Option<String>,

    /// Human-readable notes about requirements
    #[serde(default)]
    pub notes: Option<String>,
}

impl PluginManifest {
    /// Serialize to TOML string.
    pub fn to_toml(&self) -> Result<String, ManifestError> {
        toml::to_string_pretty(self).map_err(|e| {
            ManifestError::InvalidFormat(format!("Failed to serialize manifest: {e}"))
        })
    }
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
name = "tasks_plugin"
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
name = "embed_plugin"
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert!(manifest.cli.is_none());
    }

    #[test]
    fn test_parse_hive_plugin() {
        let toml = r#"
[plugin]
id = "hive.runner.docker"
name = "Docker Runner"
version = "0.1.0"
type = "hive-plugin"
author = "ADI Team"
description = "Docker container runner"

[hive]
category = "runner"
name = "docker"

[tags]
categories = ["hive", "runner", "docker"]

[binary]
name = "plugin"
"#;
        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.plugin.id, "hive.runner.docker");
        let hive = manifest.hive.unwrap();
        assert_eq!(hive.category, "runner");
        assert_eq!(hive.name, "docker");
        let tags = manifest.tags.unwrap();
        assert_eq!(tags.categories, vec!["hive", "runner", "docker"]);
    }

    #[test]
    fn test_parse_translation_plugin() {
        let toml = r#"
[plugin]
id = "adi.workflow.en-US"
name = "ADI Workflow - English"
version = "1.0.0"
type = "translation"

[translation]
translates = "adi.workflow"
language = "en-US"
language_name = "English (United States)"
namespace = "workflow"

[binary]
name = "plugin"
"#;
        let manifest = PluginManifest::from_toml(toml).unwrap();
        let tr = manifest.translation.unwrap();
        assert_eq!(tr.translates, "adi.workflow");
        assert_eq!(tr.language, "en-US");
        assert_eq!(tr.namespace, "workflow");
    }

    #[test]
    fn test_parse_language_plugin() {
        let toml = r#"
[plugin]
id = "adi.lang.rust"
name = "Rust Language Support"
version = "3.0.0"
type = "lang"

[language]
id = "rust"
extensions = ["rs"]

[binary]
name = "plugin"
"#;
        let manifest = PluginManifest::from_toml(toml).unwrap();
        let lang = manifest.language.unwrap();
        assert_eq!(lang.id, "rust");
        assert_eq!(lang.extensions, vec!["rs"]);
    }

    #[test]
    fn test_to_toml_roundtrip() {
        let toml_input = r#"
[plugin]
id = "adi.tasks"
name = "ADI Tasks"
version = "0.8.8"
type = "core"
author = "ADI Team"
description = "Task management"

[cli]
command = "tasks"
description = "Task management"
aliases = ["t"]

[[provides]]
id = "adi.tasks.cli"
version = "1.0.0"
description = "CLI commands"

[binary]
name = "plugin"

[tags]
categories = ["tasks", "workflow"]
"#;
        let manifest = PluginManifest::from_toml(toml_input).unwrap();
        let serialized = manifest.to_toml().unwrap();
        let reparsed = PluginManifest::from_toml(&serialized).unwrap();
        assert_eq!(reparsed.plugin.id, "adi.tasks");
        assert_eq!(reparsed.plugin.version, "0.8.8");
        assert!(reparsed.cli.is_some());
        assert_eq!(reparsed.provides.len(), 1);
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
name = "tasks_plugin"
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
