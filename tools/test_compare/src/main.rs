//! Tool to compare output images from Oicana's e2e tests.
//!
//! The truth are always the checked in results of the template tests in `e2e_test_template`.
//! This tool will gather the result images and compare them to the images produced
//! by tests in integrations. They should always match the reference images, or the integration is broken.

use anyhow::{bail, Context, Error};
use clap::Parser;
use log::{error, info};
use oicana_template::validate_native_template;
use oicana_testing::execution::compare_images;
use std::{ffi::OsStr, fs::read, path::PathBuf};
use walkdir::WalkDir;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let manifest = validate_native_template(&cli.e2e_test_template)?;
    let tests_dir = cli.e2e_test_template.join(manifest.tool.oicana.tests);
    info!("Comparing test results from the e2e test template at {tests_dir:?}");
    let source_files: Vec<_> = WalkDir::new(&tests_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let Some(file_name) = e.path().file_name() else {
                return false;
            };
            file_name.to_string_lossy().starts_with("e2e.")
                && e.path().extension() == Some(OsStr::new("png"))
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    let mut failures = Vec::new();
    if source_files.is_empty() {
        bail!(
            "Didn't find any snapshots in {:?}",
            tests_dir.canonicalize()?
        );
    }

    for source_file in source_files {
        let file_stem = source_file.file_stem().unwrap();
        let file_name = source_file.file_name().unwrap().to_string_lossy();
        let file_name = file_name.trim_start_matches("e2e.");
        let test_name = file_name.trim_end_matches(".png");
        let compare_path = &cli.target_dir.join(file_name);

        let to_compare = read(compare_path).context(format!(
            "Failed to load snapshot {compare_path:?} for comparison"
        ))?;
        if compare_images(&source_file, &to_compare, 1)? {
            info!("The test {test_name:?} resulted in the same image.");
        } else {
            failures.push(file_stem.to_string_lossy().to_string());
            error!("The test {test_name:?} produced different results between {source_file:?} and {compare_path:?}");
        }
    }
    if !failures.is_empty() {
        bail!("The following e2e tests created different outputs: {failures:?}");
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "test_compare", version, author)]
#[command(name = "test_compare")]
#[command(about = "Compare test output from Oicanas e2e tests")]
struct Cli {
    #[arg(
        required = true,
        help = "Test directory of the Oicana e2e test template."
    )]
    e2e_test_template: PathBuf,

    #[arg(
        required = true,
        help = "Output directory of the tests to compare to the e2e template."
    )]
    target_dir: PathBuf,
}
