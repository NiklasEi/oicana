//! # Oicana
//!
//! _Dynamic PDF Generation based on Typst_
//!
//! With this library, you can compile Oicana templates from Rust code.

use std::io::{Read, Seek};

use oicana_files::{packed::PackedTemplate, TemplateFiles};
use oicana_input::TemplateInputs;
use oicana_template::manifest::TemplateManifest;
use oicana_world::{
    diagnostics::{DiagnosticColor, TemplateDiagnostics},
    manifest::{OicanaWorldFiles, OicanaWorldManifestError},
    world::{OicanaWorld, WorldCreationError},
    CompiledDocument, TemplateCompilationFailure,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use thiserror::Error;
use typst::{
    diag::{FileResult, SourceDiagnostic},
    ecow::EcoVec,
    foundations::Bytes,
    syntax::{FileId, Source},
};

/// Global cache age configuration.
///
/// Default is 10, meaning cache entries used during the last 10 eviction cycles are kept.
/// usize::MAX is used internally to represent disabled eviction.
static CACHE_EVICTION_AGE: AtomicUsize = AtomicUsize::new(10);

/// Set the global cache age for comemo cache eviction.
///
/// Pass `None` to disable cache eviction completely.
/// Pass `Some(n)` to set the maximum age threshold.
///
/// # How Cache Aging Works
///
/// - Each cache entry has an age counter
/// - Age increases by 1 during each eviction call
/// - Age resets to 0 when the entry is used (cache hit)
/// - Entries with age >= `max_age` are removed
///
/// # Parameters
///
/// * `max_age` - Maximum age threshold, or None to disable:
///   - `None` - Disables cache eviction (cache never cleared)
///   - `Some(0)` - Clears all cache after every compilation
///   - `Some(1)` - Keeps only entries used since the last eviction
///   - `Some(n)` - Keeps entries used within the last n compilations
///
/// Default: 10
///
/// # Example
///
/// ```
/// use oicana::set_cache_eviction_age;
///
/// // Clear cache after every compilation
/// set_cache_eviction_age(Some(0));
///
/// // Keep cache entries from last 50 compilations
/// set_cache_eviction_age(Some(50));
///
/// // Disable eviction completely
/// set_cache_eviction_age(None);
/// ```
pub fn set_cache_eviction_age(max_age: Option<usize>) {
    CACHE_EVICTION_AGE.store(max_age.unwrap_or(usize::MAX), Ordering::Relaxed);
}

/// Support for native Oicana templates.
/// Native templates are not packed. They are a Typst project in a native file system.
#[cfg(feature = "native")]
pub mod native;

/// A prepared Oicana Template
pub struct Template<F: TemplateFiles> {
    world: OicanaWorld<F>,
}

impl Template<PackedTemplate> {
    /// Initialize the given template
    pub fn init<R: Read + Seek>(template: R) -> Result<Self, TemplateInitializationError> {
        let files = PackedTemplate::new(template);
        let manifest = files.manifest()?;

        let world = OicanaWorld::new(files, TemplateInputs::new(), manifest)?;

        Ok(Template { world })
    }
}

impl<Files: TemplateFiles> Template<Files> {
    /// Compile the template with given inputs
    pub fn compile(
        &mut self,
        inputs: TemplateInputs,
    ) -> Result<CompiledDocument, TemplateCompilationFailure> {
        self.world.update_inputs(inputs);
        let result = self.world.compile();
        let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
        if cache_age != usize::MAX {
            oicana_world::evict_cache(cache_age);
        }
        result
    }

    /// Get the manifest of the template
    pub fn manifest(&self) -> &TemplateManifest {
        self.world.manifest()
    }

    /// Return the source of a file in the template project
    pub fn source(&self, id: FileId) -> FileResult<Source> {
        self.world.files.source(id)
    }

    /// Return a file in the template project as bytes
    pub fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.world.files.file(id)
    }

    /// Configure the coloring of diagnostic output from this template
    pub fn set_diagnostic_color(&mut self, color: DiagnosticColor) {
        self.world.color = color;
    }
}

impl<Files: TemplateFiles> TemplateDiagnostics for Template<Files> {
    fn format_diagnostics(&self, diagnostics: EcoVec<SourceDiagnostic>) -> Vec<u8> {
        self.world.format_diagnostics(diagnostics)
    }
}

/// An error occurred while initiating the template
#[derive(Error, Debug)]
pub enum TemplateInitializationError {
    /// An error concerning the template manifest
    #[error("Manifest error: {0}")]
    ManifestError(#[from] OicanaWorldManifestError),

    /// Error while creating the template world
    #[error("Issue while creating template World: {0}")]
    WorldCreationError(#[from] WorldCreationError),

    /// The data directory for Typst packages could not be found
    #[error("Failed to find the data directory for Typst packages on the System")]
    PackageDirectoryNotFound,
}
