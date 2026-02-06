//! Generate plugin.toml from Cargo.toml `[package.metadata.plugin]`.
//!
//! Usage: manifest-gen --cargo-toml <path> [--output <path>]

use lib_plugin_manifest::cargo_extract::generate_manifest_from_cargo;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut cargo_toml_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cargo-toml" => {
                i += 1;
                cargo_toml_path = Some(PathBuf::from(&args[i]));
            }
            "--output" | "-o" => {
                i += 1;
                output_path = Some(PathBuf::from(&args[i]));
            }
            "--help" | "-h" => {
                eprintln!("Usage: manifest-gen --cargo-toml <path> [--output <path>]");
                eprintln!();
                eprintln!("Generate plugin.toml from Cargo.toml [package.metadata.plugin].");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  --cargo-toml <path>  Path to Cargo.toml (required)");
                eprintln!("  --output, -o <path>  Output path (default: stdout)");
                std::process::exit(0);
            }
            other => {
                // Positional: treat first positional as cargo-toml path
                if cargo_toml_path.is_none() {
                    cargo_toml_path = Some(PathBuf::from(other));
                } else {
                    eprintln!("Unknown argument: {other}");
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }

    let cargo_toml_path = match cargo_toml_path {
        Some(p) => p,
        None => {
            eprintln!("Error: --cargo-toml <path> is required");
            std::process::exit(1);
        }
    };

    if !cargo_toml_path.exists() {
        eprintln!("Error: file not found: {}", cargo_toml_path.display());
        std::process::exit(1);
    }

    let manifest = match generate_manifest_from_cargo(&cargo_toml_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let toml_str = match manifest.to_toml() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error serializing manifest: {e}");
            std::process::exit(1);
        }
    };

    match output_path {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, &toml_str) {
                eprintln!("Error writing to {}: {e}", path.display());
                std::process::exit(1);
            }
        }
        None => print!("{toml_str}"),
    }
}
