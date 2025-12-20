//! Platform detection and binary filename utilities.

/// Get the current platform identifier.
///
/// Returns a string like "darwin-aarch64", "linux-x86_64", etc.
pub fn current_platform() -> String {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };

    format!("{}-{}", os, arch)
}

/// Get the library filename for a given binary name on the current platform.
///
/// Adds the appropriate prefix (lib on Unix) and extension (.dylib, .so, .dll).
pub fn library_filename(name: &str) -> String {
    let prefix = if cfg!(target_os = "windows") {
        ""
    } else {
        "lib"
    };

    let ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    };

    format!("{}{}.{}", prefix, name, ext)
}

/// Check if the current platform matches a platform identifier.
pub fn matches_platform(platform: &str) -> bool {
    let current = current_platform();
    platform == current || platform == "all"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        assert!(!platform.is_empty());
        assert!(platform.contains('-'));
    }

    #[test]
    fn test_library_filename() {
        let name = library_filename("my_plugin");
        assert!(name.contains("my_plugin"));
    }

    #[test]
    fn test_matches_platform() {
        assert!(matches_platform(&current_platform()));
        assert!(matches_platform("all"));
        assert!(!matches_platform("nonexistent-platform"));
    }
}
