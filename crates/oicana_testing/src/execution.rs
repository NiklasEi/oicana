use std::{
    fs, io,
    path::{Path, PathBuf},
};

use image::{GenericImageView, ImageError};
use log::warn;
use oicana::{Template, TemplateInitializationError};
use oicana_export::png::{export_merged_png, EncodingError};
use oicana_files::native::{package_data_dir, NativeTemplate};
use oicana_template::manifest::TemplateManifest;
use oicana_world::{CompiledDocument, TemplateCompilationFailure};
use thiserror::Error;

use crate::Test;

/// Context for test runners
pub struct TestRunnerContext {
    packages: PathBuf,
}

impl TestRunnerContext {
    /// Create a new test runner
    pub fn new() -> Result<Self, CreateTestRunnerError> {
        let packages = package_data_dir().ok_or(CreateTestRunnerError::NoPackageDirectory)?;

        Ok(TestRunnerContext { packages })
    }

    /// Prepare a runner for the template at the given path
    pub fn get_runner(
        &self,
        path: &Path,
        manifest: &TemplateManifest,
    ) -> Result<TestRunner, TemplateInitializationError> {
        Ok(TestRunner {
            instance: Template::<NativeTemplate>::from(path, &self.packages, manifest.clone())?,
        })
    }
}

/// Execution context for tests of a single template
pub struct TestRunner {
    instance: Template<NativeTemplate>,
}

impl TestRunner {
    /// Run the test case
    pub fn run(&mut self, test: Test) -> Result<Vec<String>, TestExecutionError> {
        let CompiledDocument { document, warnings } = self.instance.compile(test.inputs)?;
        let mut warnings = if let Some(warning) = warnings {
            vec![warning]
        } else {
            vec![]
        };

        let image = export_merged_png(&document, 1.)?;
        match test.snapshot {
            crate::Snapshot::Missing(path) => {
                warnings.push(format!(
                    "Writing snapshot file at {path:?}, because it was missing"
                ));
                fs::write(path, image)?;
            }
            crate::Snapshot::Some(path) => {
                if !compare_images(&path, &image, 1)? {
                    let mut compare_path = path.clone();
                    if let Some(stem) = compare_path.file_stem() {
                        let mut new_name = stem.to_os_string();
                        new_name.push(".compare.png");
                        compare_path.set_file_name(new_name);
                    } else {
                        warn!("Snapshot file had no file stem");
                    }
                    fs::write(compare_path, image)?;
                    return Err(TestExecutionError::SnapshotMismatch);
                }
            }
            _ => (),
        }

        Ok(warnings)
    }
}

/// Compare the image file at `path` with the given data. If there is a pixel value with a higher difference
/// than `tolerance`, return `false`.
pub fn compare_images(path: &Path, data: &[u8], tolerance: u8) -> Result<bool, ImageError> {
    let img1 = image::open(path)?;
    let img2 = image::load_from_memory(data)?;

    if img1.dimensions() != img2.dimensions() {
        return Ok(false);
    }

    let pixels1 = img1.pixels();
    let pixels2 = img2.pixels();

    for ((_, _, p1), (_, _, p2)) in pixels1.zip(pixels2) {
        let diff =
            p1.0.iter()
                .zip(p2.0.iter())
                .map(|(a, b)| (*a as i16 - *b as i16).unsigned_abs() as u8)
                .max()
                .unwrap_or(0);

        if diff > tolerance {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Errors that can be produced when executing tests
#[derive(Debug, Error)]
pub enum CreateTestRunnerError {
    /// Could not get the local Typst package directory
    #[error("Failed to get Typst package directory.")]
    NoPackageDirectory,
}

/// Errors that can be produced when executing tests
#[derive(Debug, Error)]
pub enum TestExecutionError {
    /// A test failed to compile
    #[error("{0}")]
    CompilationError(#[from] TemplateCompilationFailure),
    /// Failed to export png image
    #[error("{0}")]
    ExportError(#[from] EncodingError),
    /// Failed to write or read snapshot image
    #[error("Failed to write or read snapshot image: {0}")]
    Io(#[from] io::Error),
    /// Failed to compare the snapshot
    #[error("Failed to compare the snapshot: {0}")]
    Comparison(#[from] ImageError),
    /// The snapshot does not match the result
    #[error("The snapshot does not match the result.")]
    SnapshotMismatch,
}
