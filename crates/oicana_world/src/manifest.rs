use oicana_files::TemplateFiles;
use oicana_template::manifest::TemplateManifest;
use thiserror::Error;
use typst::diag::FileError;
use typst::syntax::{FileId, VirtualPath};

/// Files that represent an Oicana World
pub trait OicanaWorldFiles<Files: TemplateFiles> {
    /// Get the manifest for this world.
    fn manifest(&self) -> Result<TemplateManifest, OicanaWorldManifestError>;
}

impl<Files: TemplateFiles> OicanaWorldFiles<Files> for Files {
    fn manifest(&self) -> Result<TemplateManifest, OicanaWorldManifestError> {
        let manifest = self.source(FileId::new(None, VirtualPath::new("/typst.toml")))?;
        TemplateManifest::from_toml(manifest.text()).map_err(Into::into)
    }
}

/// Errors when reading the manifest from the template files.
#[derive(Error, Debug)]
pub enum OicanaWorldManifestError {
    /// Error from attempting to parse the manifest file.
    #[error("Failed to parse manifest file `typst.toml`: {0}")]
    InvalidManifest(#[from] toml::de::Error),
    /// The manifest file could not be found.
    #[error("Failed to find the manifest file: {0}")]
    NoManifest(#[from] FileError),
}
