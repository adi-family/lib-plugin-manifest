//! Extract plugin manifest from Cargo.toml `[package.metadata.plugin]`.

use std::path::Path;

use crate::error::ManifestError;
use crate::plugin::*;

/// Generate a `PluginManifest` from a Cargo.toml with `[package.metadata.plugin]`.
pub fn generate_manifest_from_cargo(cargo_toml_path: &Path) -> Result<PluginManifest, ManifestError> {
    let content = std::fs::read_to_string(cargo_toml_path)?;
    let doc: toml::Value = toml::from_str(&content).map_err(ManifestError::TomlParse)?;

    let package = doc
        .get("package")
        .ok_or_else(|| ManifestError::MissingField("package".into()))?;

    // Resolve version (may be workspace-inherited)
    let version = resolve_version(package, cargo_toml_path)?;
    let description = package
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let author = resolve_author(package);

    let metadata_plugin = package
        .get("metadata")
        .and_then(|m| m.get("plugin"))
        .ok_or_else(|| ManifestError::MissingField("package.metadata.plugin".into()))?;

    // Required plugin fields
    let id = metadata_plugin
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ManifestError::MissingField("package.metadata.plugin.id".into()))?
        .to_string();
    let name = metadata_plugin
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ManifestError::MissingField("package.metadata.plugin.name".into()))?
        .to_string();
    let plugin_type = metadata_plugin
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ManifestError::MissingField("package.metadata.plugin.type".into()))?
        .to_string();

    // Compatibility
    let compatibility = parse_compatibility(metadata_plugin);

    // CLI config
    let cli = parse_cli(metadata_plugin);

    // Provides
    let provides = parse_provides(metadata_plugin);

    // Requires
    let requires = parse_requires(metadata_plugin);

    // Binary
    let binary = parse_binary(metadata_plugin);

    // Tags
    let tags = parse_tags(metadata_plugin);

    // Hive
    let hive = parse_hive(metadata_plugin);

    // Translation
    let translation = parse_translation(metadata_plugin);

    // Language
    let language = parse_language(metadata_plugin);

    // Requirements
    let requirements = parse_requirements(metadata_plugin);

    // Capabilities
    let capabilities = parse_capabilities(metadata_plugin);

    Ok(PluginManifest {
        plugin: PluginMeta {
            id,
            name,
            version,
            plugin_type,
            author,
            description,
            license: None,
            homepage: None,
        },
        compatibility,
        binary,
        signature: None,
        config: ConfigInfo::default(),
        provides,
        requires,
        cli,
        capabilities,
        tags,
        hive,
        translation,
        language,
        requirements,
    })
}

fn resolve_version(package: &toml::Value, cargo_toml_path: &Path) -> Result<String, ManifestError> {
    if let Some(v) = package.get("version") {
        if let Some(s) = v.as_str() {
            return Ok(s.to_string());
        }
        // version = { workspace = true }
        if let Some(table) = v.as_table() {
            if table.get("workspace").and_then(|w| w.as_bool()) == Some(true) {
                return resolve_workspace_version(cargo_toml_path);
            }
        }
    }
    // version.workspace = true (dotted key)
    Err(ManifestError::MissingField("package.version".into()))
}

fn resolve_workspace_version(cargo_toml_path: &Path) -> Result<String, ManifestError> {
    let mut dir = cargo_toml_path
        .parent()
        .ok_or_else(|| ManifestError::InvalidFormat("no parent dir".into()))?;

    loop {
        dir = match dir.parent() {
            Some(p) => p,
            None => break,
        };
        let ws_toml = dir.join("Cargo.toml");
        if !ws_toml.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&ws_toml)?;
        let doc: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(version) = doc
            .get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
        {
            return Ok(version.to_string());
        }
    }

    Err(ManifestError::InvalidFormat(
        "Could not resolve workspace version".into(),
    ))
}

fn resolve_author(package: &toml::Value) -> String {
    package
        .get("authors")
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn parse_compatibility(meta: &toml::Value) -> CompatibilityInfo {
    let compat = match meta.get("compatibility") {
        Some(c) => c,
        None => return CompatibilityInfo::default(),
    };

    CompatibilityInfo {
        api_version: compat
            .get("api_version")
            .and_then(|v| v.as_integer())
            .unwrap_or(2) as u32,
        min_host_version: compat
            .get("min_host_version")
            .and_then(|v| v.as_str())
            .map(String::from),
        max_host_version: compat
            .get("max_host_version")
            .and_then(|v| v.as_str())
            .map(String::from),
        platforms: compat
            .get("platforms")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        depends_on: compat
            .get("depends_on")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    }
}

fn parse_cli(meta: &toml::Value) -> Option<CliConfig> {
    let cli = meta.get("cli")?;
    Some(CliConfig {
        command: cli.get("command")?.as_str()?.to_string(),
        description: cli
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        aliases: cli
            .get("aliases")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        dynamic_completions: cli
            .get("dynamic_completions")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    })
}

fn parse_provides(meta: &toml::Value) -> Vec<ServiceDeclaration> {
    meta.get("provides")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(ServiceDeclaration {
                        id: item.get("id")?.as_str()?.to_string(),
                        version: item
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("1.0.0")
                            .to_string(),
                        description: item
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_requires(meta: &toml::Value) -> Vec<ServiceRequirement> {
    meta.get("requires")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(ServiceRequirement {
                        id: item.get("id")?.as_str()?.to_string(),
                        min_version: item
                            .get("min_version")
                            .or_else(|| item.get("version"))
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        optional: item
                            .get("optional")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_binary(meta: &toml::Value) -> BinaryInfo {
    match meta.get("binary") {
        Some(b) => BinaryInfo {
            name: b
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("plugin")
                .to_string(),
            checksums: Default::default(),
        },
        None => BinaryInfo::default(),
    }
}

fn parse_tags(meta: &toml::Value) -> Option<TagsInfo> {
    let tags = meta.get("tags")?;
    Some(TagsInfo {
        categories: tags
            .get("categories")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        platforms: tags
            .get("platforms")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    })
}

fn parse_hive(meta: &toml::Value) -> Option<HiveInfo> {
    let hive = meta.get("hive")?;
    Some(HiveInfo {
        category: hive.get("category")?.as_str()?.to_string(),
        name: hive.get("name")?.as_str()?.to_string(),
    })
}

fn parse_translation(meta: &toml::Value) -> Option<TranslationInfo> {
    let tr = meta.get("translation")?;
    Some(TranslationInfo {
        translates: tr.get("translates")?.as_str()?.to_string(),
        language: tr.get("language")?.as_str()?.to_string(),
        language_name: tr
            .get("language_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        namespace: tr
            .get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn parse_language(meta: &toml::Value) -> Option<LanguageInfo> {
    let lang = meta.get("language")?;
    Some(LanguageInfo {
        id: lang.get("id")?.as_str()?.to_string(),
        extensions: lang
            .get("extensions")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    })
}

fn parse_requirements(meta: &toml::Value) -> Option<RequirementsInfo> {
    let req = meta.get("requirements")?;
    Some(RequirementsInfo {
        os: req.get("os").and_then(|v| v.as_str()).map(String::from),
        arch: req.get("arch").and_then(|v| v.as_str()).map(String::from),
        notes: req.get("notes").and_then(|v| v.as_str()).map(String::from),
    })
}

fn parse_capabilities(meta: &toml::Value) -> Vec<CapabilityDeclaration> {
    meta.get("capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(CapabilityDeclaration {
                        protocol: item.get("protocol")?.as_str()?.to_string(),
                        version: item
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("1.0.0")
                            .to_string(),
                        description: item
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_from_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "adi-tasks-plugin"
version = "0.8.8"
edition = "2021"
description = "Task management with dependency tracking"
authors = ["ADI Team"]

[package.metadata.plugin]
id = "adi.tasks"
name = "ADI Tasks"
type = "core"

[package.metadata.plugin.compatibility]
api_version = 3
min_host_version = "0.9.0"

[package.metadata.plugin.cli]
command = "tasks"
description = "Task management"
aliases = ["t"]

[[package.metadata.plugin.provides]]
id = "adi.tasks.cli"
version = "1.0.0"
description = "CLI commands"

[package.metadata.plugin.binary]
name = "plugin"

[package.metadata.plugin.tags]
categories = ["tasks", "workflow"]
"#,
        )
        .unwrap();

        let manifest = generate_manifest_from_cargo(&cargo_toml).unwrap();
        assert_eq!(manifest.plugin.id, "adi.tasks");
        assert_eq!(manifest.plugin.name, "ADI Tasks");
        assert_eq!(manifest.plugin.version, "0.8.8");
        assert_eq!(manifest.plugin.plugin_type, "core");
        assert_eq!(manifest.plugin.author, "ADI Team");
        assert_eq!(manifest.plugin.description, "Task management with dependency tracking");
        assert_eq!(manifest.compatibility.api_version, 3);
        assert_eq!(
            manifest.compatibility.min_host_version,
            Some("0.9.0".into())
        );
        assert!(manifest.cli.is_some());
        let cli = manifest.cli.unwrap();
        assert_eq!(cli.command, "tasks");
        assert_eq!(cli.aliases, vec!["t"]);
        assert_eq!(manifest.provides.len(), 1);
        assert_eq!(manifest.provides[0].id, "adi.tasks.cli");
        let tags = manifest.tags.unwrap();
        assert_eq!(tags.categories, vec!["tasks", "workflow"]);
    }

    #[test]
    fn test_workspace_version_resolution() {
        let dir = tempfile::tempdir().unwrap();

        // Create workspace root
        let ws_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &ws_toml,
            r#"
[workspace]
members = ["plugins/test"]

[workspace.package]
version = "1.2.3"
"#,
        )
        .unwrap();

        // Create nested crate
        let plugin_dir = dir.path().join("plugins").join("test");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        let cargo_toml = plugin_dir.join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-plugin"
version.workspace = true
description = "Test"
authors = ["Test"]

[package.metadata.plugin]
id = "test.plugin"
name = "Test Plugin"
type = "core"
"#,
        )
        .unwrap();

        let manifest = generate_manifest_from_cargo(&cargo_toml).unwrap();
        assert_eq!(manifest.plugin.version, "1.2.3");
    }

    #[test]
    fn test_hive_plugin_extraction() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "hive-runner-docker"
version = "0.1.0"
description = "Docker runner"
authors = ["ADI Team"]

[package.metadata.plugin]
id = "hive.runner.docker"
name = "Docker Runner"
type = "hive-plugin"

[package.metadata.plugin.hive]
category = "runner"
name = "docker"

[package.metadata.plugin.tags]
categories = ["hive", "runner"]
"#,
        )
        .unwrap();

        let manifest = generate_manifest_from_cargo(&cargo_toml).unwrap();
        assert_eq!(manifest.plugin.id, "hive.runner.docker");
        let hive = manifest.hive.unwrap();
        assert_eq!(hive.category, "runner");
        assert_eq!(hive.name, "docker");
    }

    #[test]
    fn test_translation_plugin_extraction() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "adi-workflow-lang-en"
version = "1.0.0"
description = "English translations"
authors = ["ADI Team"]

[package.metadata.plugin]
id = "adi.workflow.en-US"
name = "ADI Workflow - English"
type = "translation"

[package.metadata.plugin.translation]
translates = "adi.workflow"
language = "en-US"
language_name = "English (United States)"
namespace = "workflow"
"#,
        )
        .unwrap();

        let manifest = generate_manifest_from_cargo(&cargo_toml).unwrap();
        let tr = manifest.translation.unwrap();
        assert_eq!(tr.translates, "adi.workflow");
        assert_eq!(tr.language, "en-US");
    }

    #[test]
    fn test_language_plugin_extraction() {
        let dir = tempfile::tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "adi-lang-rust"
version = "3.0.0"
description = "Rust language support"
authors = ["ADI Team"]

[package.metadata.plugin]
id = "adi.lang.rust"
name = "Rust Language Support"
type = "lang"

[package.metadata.plugin.language]
id = "rust"
extensions = ["rs"]

[package.metadata.plugin.compatibility]
api_version = 3
min_host_version = "0.9.0"
"#,
        )
        .unwrap();

        let manifest = generate_manifest_from_cargo(&cargo_toml).unwrap();
        let lang = manifest.language.unwrap();
        assert_eq!(lang.id, "rust");
        assert_eq!(lang.extensions, vec!["rs"]);
    }
}
