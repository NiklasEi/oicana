//! Build script for the Oicana CLI.
//!
//! Resolves the version of the bundled Typst compiler via `cargo_metadata`
//! and exposes it as the `TYPST_VERSION` env var so it can be embedded in the
//! CLI's `--version` output.

use cargo_metadata::MetadataCommand;

fn main() {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("failed to run `cargo metadata`");

    let lock_path = metadata.workspace_root.join("Cargo.lock");
    println!("cargo:rerun-if-changed={lock_path}");

    let typst = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name.as_str() == "typst")
        .expect("typst package not found in cargo metadata");
    println!("cargo:rustc-env=TYPST_VERSION={}", typst.version);
}
