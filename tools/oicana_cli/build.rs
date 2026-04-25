//! Build script for the Oicana CLI.
//!
//! Reads the resolved version of the bundled Typst compiler from `Cargo.lock`
//! and exposes it via the `TYPST_VERSION` env var so it can be embedded in the
//! CLI's `--version` output.

use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo");
    let lock_path = find_cargo_lock(Path::new(&manifest_dir))
        .unwrap_or_else(|| panic!("Cargo.lock not found above {manifest_dir}"));
    println!("cargo:rerun-if-changed={}", lock_path.display());

    let lock = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", lock_path.display()));
    let version =
        find_package_version(&lock, "typst").expect("typst package not found in Cargo.lock");
    println!("cargo:rustc-env=TYPST_VERSION={version}");
}

/// Walk up from `start` looking for the nearest `Cargo.lock`.
///
/// Handles both the workspace layout (lockfile at the workspace root) and the
/// standalone layout produced by `cargo publish` / `cargo install` (lockfile
/// at the package root).
fn find_cargo_lock(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join("Cargo.lock");
        if candidate.is_file() {
            return Some(candidate);
        }
        current = dir.parent();
    }
    None
}

fn find_package_version(lock: &str, name: &str) -> Option<String> {
    let target_name = format!("name = \"{name}\"");
    let mut in_package = false;
    let mut name_matches = false;
    for line in lock.lines() {
        let line = line.trim_end();
        if line == "[[package]]" {
            in_package = true;
            name_matches = false;
        } else if in_package && line == target_name {
            name_matches = true;
        } else if name_matches && line.starts_with("version = \"") {
            return line
                .strip_prefix("version = \"")
                .and_then(|rest| rest.strip_suffix('"'))
                .map(str::to_owned);
        }
    }
    None
}
