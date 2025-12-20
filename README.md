# lib-plugin-manifest

Plugin manifest parsing for the universal Rust plugin system.

## Overview

Parses `plugin.toml` (single plugins) and `package.toml` (multi-plugin packages).

## Single Plugin (plugin.toml)

```toml
[plugin]
id = "vendor.my-plugin"
name = "My Plugin"
version = "1.0.0"
type = "extension"
author = "Your Name"
description = "What this plugin does"

[compatibility]
api_version = 1
min_host_version = "0.8.0"
platforms = ["darwin-aarch64", "linux-x86_64"]

[binary]
name = "my_plugin"
[binary.checksums]
darwin-aarch64 = "sha256:..."

[config.defaults]
option1 = "value"
```

## Multi-Plugin Package (package.toml)

```toml
[package]
id = "vendor.theme-pack"
name = "Theme Collection"
version = "2.0.0"
author = "Vendor Team"

[compatibility]
api_version = 1

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
depends_on = ["vendor.theme-dark"]
```

## Usage

```rust
use lib_plugin_manifest::{Manifest, PluginManifest, PackageManifest};

// Auto-detect type
let manifest = Manifest::from_file("plugin.toml")?;
println!("Plugin IDs: {:?}", manifest.plugin_ids());

// Parse specific type
let plugin = PluginManifest::from_file("plugin.toml")?;
let package = PackageManifest::from_file("package.toml")?;

// Get install order for package (respects dependencies)
let order = package.install_order()?;
```

## License

MIT
