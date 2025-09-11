use crate::diagnostics::{DiagnosticColor, TemplateDiagnostics};
use crate::fonts::{FontCollection, FontSlot};
use crate::{get_current_time, CompiledDocument, TemplateCompilationFailure};

use chrono::{DateTime, Datelike, Local};
use log::info;
use oicana_files::TemplateFiles;
use oicana_input::TemplateInputs;
use oicana_template::manifest::ManifestValidationError;
use oicana_template::manifest::TemplateManifest;
use std::sync::OnceLock;
use thiserror::Error;
use typst::diag::{FileError, FileResult, Warned};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

/// A world that provides access to fonts and template files.
#[derive(Debug)]
pub struct OicanaWorld<Files: TemplateFiles> {
    main: FileId,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
    now: OnceLock<DateTime<Local>>,
    manifest: TemplateManifest,
    /// Color mode for diagnostic logs
    pub color: DiagnosticColor,
    /// Files access to the template.
    pub files: Files,
}

impl<Files: TemplateFiles> OicanaWorld<Files> {
    /// Create a new Typst World.
    ///
    /// This will collect embedded fonts from Typst and fonts included in the template files.
    pub fn new(
        files: Files,
        inputs: TemplateInputs,
        manifest: TemplateManifest,
    ) -> Result<Self, WorldCreationError> {
        let library = Library::builder().with_inputs(inputs.to_dict()).build();

        let main_path = VirtualPath::new(manifest.package.entrypoint.as_str());
        let main = FileId::new(None, main_path);
        files.source(main)?;

        let mut searcher = FontCollection::new();
        searcher.collect(&files);

        Ok(Self {
            main,
            library: LazyHash::new(library),
            book: LazyHash::new(searcher.book),
            fonts: searcher.fonts,
            now: OnceLock::new(),
            manifest,
            color: DiagnosticColor::Ansi,
            files,
        })
    }

    /// Update the inputs of the World.
    ///
    /// These inputs will stay in the World cache until overwritten again.
    pub fn update_inputs(&mut self, inputs: TemplateInputs) {
        self.library = LazyHash::new(Library::builder().with_inputs(inputs.to_dict()).build());
    }

    /// Compile the template world.
    pub fn compile(&mut self) -> Result<CompiledDocument, TemplateCompilationFailure> {
        let start = get_current_time();
        // We take a small performance hit here
        // to prevent https://github.com/typst/typst/issues/6832
        comemo::evict(0);
        let Warned { output, warnings } = typst::compile(self);
        info!("Compiled Document in {}ms", get_current_time() - start);
        let warnings = if warnings.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&self.format_diagnostics(warnings)).into())
        };

        match output {
            Ok(document) => Ok(CompiledDocument { document, warnings }),
            Err(diagnostics) => Err(TemplateCompilationFailure {
                error: String::from_utf8_lossy(&self.format_diagnostics(diagnostics)).into(),
                warnings,
            }),
        }
    }

    /// Manifest of the Oicana template
    pub fn manifest(&self) -> &TemplateManifest {
        &self.manifest
    }
}

/// An error that occurs during world construction.
#[derive(Error, Debug)]
pub enum WorldCreationError {
    /// Error while accessing a file in the template
    #[error("Failed to access a required file")]
    FileError(#[from] FileError),
    /// Error in the template manifest
    #[error("There was an issue with the package manifest")]
    ManifestError(#[from] ManifestValidationError),
}

impl<Files: TemplateFiles> World for OicanaWorld<Files> {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = self.now.get_or_init(Local::now);

        let naive = match offset {
            None => now.naive_local(),
            Some(o) => now.naive_utc() + chrono::Duration::try_hours(o)?,
        };

        Datetime::from_ymd(
            naive.year(),
            naive.month().try_into().ok()?,
            naive.day().try_into().ok()?,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::{OicanaWorldFiles, OicanaWorldManifestError};
    use crate::world::{OicanaWorld, WorldCreationError};
    use oicana_files::preloaded::PreloadedTemplate;
    use oicana_input::TemplateInputs;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use typst::diag::FileError;

    #[test]
    fn can_build_world_with_minimal_template() {
        let mut files = HashMap::new();
        files.insert("main.typ".to_owned(), "Test".to_owned());
        files.insert(
            "typst.toml".to_owned(),
            r#"
        [package]
        entrypoint = "main.typ"
        name = "test"
        version = "0.1.0"

        [tool.oicana]
        manifest_version = 1
        "#
            .to_owned(),
        );
        let files = PreloadedTemplate::new(files);
        let manifest = files.manifest().expect("should be able to parse manifest");

        OicanaWorld::new(files, TemplateInputs::new(), manifest).expect("Failed to create world");
    }

    #[test]
    fn fails_to_build_world_without_typst_toml_file() {
        let mut files = HashMap::new();
        files.insert("some_wrong_file_name.typ".to_owned(), "Test".to_owned());
        let files = PreloadedTemplate::new(files);

        let Err(OicanaWorldManifestError::NoManifest(file_error)) = files.manifest() else {
            panic!("Created a world without main template file or with wrong error")
        };

        assert_eq!(
            file_error,
            FileError::NotFound(PathBuf::from("/typst.toml"))
        )
    }

    #[test]
    fn fails_to_parse_invalid_package_meta() {
        let mut files = HashMap::new();
        files.insert(
            "typst.toml".to_owned(),
            r#"
        [package]
        entrypoint = "not_main.typ"

        [tool.oicana]
        manifest_version = 1
        "#
            .to_owned(),
        );
        files.insert("main.typ".to_owned(), "Test".to_owned());
        let files = PreloadedTemplate::new(files);

        let Err(OicanaWorldManifestError::InvalidManifest(file_error)) = files.manifest() else {
            panic!("Created a world with invalid package meta or got unexpected error")
        };

        assert_eq!(file_error.message(), "missing field `name`")
    }

    #[test]
    fn fails_to_build_world_without_entry_file() {
        let mut files = HashMap::new();
        files.insert(
            "typst.toml".to_owned(),
            r#"
        [package]
        entrypoint = "not_main.typ"
        name = "test"
        version = "0.1.0"

        [tool.oicana]
        manifest_version = 1
        "#
            .to_owned(),
        );
        files.insert("main.typ".to_owned(), "Test".to_owned());
        let files = PreloadedTemplate::new(files);
        let manifest = files.manifest().expect("should be able to parse manifest");

        let Err(WorldCreationError::FileError(file_error)) =
            OicanaWorld::new(files, TemplateInputs::new(), manifest)
        else {
            panic!("Created a world without main template file or with wrong error")
        };

        assert_eq!(
            file_error,
            FileError::NotFound(PathBuf::from("/not_main.typ"))
        )
    }
}
