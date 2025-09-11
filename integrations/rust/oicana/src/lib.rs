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
    diagnostics::TemplateDiagnostics,
    manifest::{OicanaWorldFiles, OicanaWorldManifestError},
    world::{OicanaWorld, WorldCreationError},
    CompiledDocument, TemplateCompilationFailure,
};
use thiserror::Error;
use typst::{
    diag::{FileResult, SourceDiagnostic},
    ecow::EcoVec,
    foundations::Bytes,
    syntax::{FileId, Source},
};

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
        self.world.compile()
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
