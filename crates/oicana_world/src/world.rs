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
use typst::{Library, LibraryExt, World};

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
    ///
    /// Note: This does not automatically evict the comemo cache. Use [`evict_cache`] to
    /// manually manage cache eviction if needed.
    pub fn compile(&mut self) -> Result<CompiledDocument, TemplateCompilationFailure> {
        let start = get_current_time();
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

/// Evict cached compilation artifacts from the global comemo cache.
///
/// The comemo cache is global and shared across all `OicanaWorld` instances.
/// This function removes memoized results whose age is larger than or equal to `max_age`.
///
/// # How Cache Aging Works
///
/// - Each cache entry has an age counter
/// - Age increases by 1 during each eviction
/// - Age resets to 0 when the entry produces a cache hit (is used)
/// - Entries with age >= `max_age` are removed
///
/// # Parameters
///
/// * `max_age` - Maximum age threshold:
///   - `0` - Removes all cache entries (full clear)
///   - `n` - Keeps entries used within the last n eviction cycles
///
/// # Example
///
/// ```rust
/// use oicana_world::evict_cache;
///
/// // Clear all cache entries
/// evict_cache(0);
///
/// // Keep entries used in last 30 eviction cycles
/// evict_cache(30);
/// ```
pub fn evict_cache(max_age: usize) {
    comemo::evict(max_age);
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

    fn simple_manifest() -> &'static str {
        r#"
        [package]
        name = "test"
        version = "0.1.0"
        entrypoint = "main.typ"
        
        [tool.oicana]
        manifest_version = 1
        "#
    }

    fn template_with(manifest: &str, main_typ: &str) -> PreloadedTemplate {
        let mut files = HashMap::new();
        files.insert("typst.toml".to_owned(), manifest.to_owned());
        files.insert("main.typ".to_owned(), main_typ.to_owned());
        PreloadedTemplate::new(files)
    }

    #[test]
    fn compiles_simple_template() {
        let files = template_with(
            simple_manifest(),
            "#set page(width: 200pt, height: 100pt)\nHello, World!",
        );
        let manifest = files.manifest().unwrap();
        let mut world = OicanaWorld::new(files, TemplateInputs::new(), manifest).unwrap();

        let result = world.compile();

        assert!(result.is_ok());
        let compiled = result.unwrap();
        assert!(compiled.warnings.is_none());
        assert_eq!(compiled.document.pages.len(), 1);
    }

    #[test]
    fn compiles_multipage_template() {
        let files = template_with(simple_manifest(), "#set page(width: 200pt, height: 100pt)\nPage 1\n#pagebreak()\nPage 2\n#pagebreak()\nPage 3");
        let manifest = files.manifest().unwrap();
        let mut world = OicanaWorld::new(files, TemplateInputs::new(), manifest).unwrap();

        let compiled = world.compile().unwrap();

        assert_eq!(compiled.document.pages.len(), 3);
    }

    #[test]
    fn fails_to_compile_invalid_template() {
        let files = template_with(simple_manifest(), "#invalid_typst_syntax #(");
        let manifest = files.manifest().unwrap();
        let mut world = OicanaWorld::new(files, TemplateInputs::new(), manifest).unwrap();

        let result = world.compile();

        assert!(result.is_err());
    }

    #[test]
    fn compiles_empty_template() {
        let files = template_with(simple_manifest(), "");
        let manifest = files.manifest().unwrap();
        let mut world = OicanaWorld::new(files, TemplateInputs::new(), manifest).unwrap();

        let compiled = world.compile().unwrap();

        assert_eq!(compiled.document.pages.len(), 1);
    }

    #[test]
    fn compilation_with_warnings_includes_warning_message() {
        let files = template_with(
            simple_manifest(),
            "#set text(font: \"NonexistentFont\")\nContent",
        );
        let manifest = files.manifest().unwrap();
        let mut world = OicanaWorld::new(files, TemplateInputs::new(), manifest).unwrap();

        let compiled = world.compile().unwrap();

        assert!(compiled.warnings.is_some());
    }

    #[test]
    fn manifest_returns_correct_template_manifest() {
        let files = template_with(
            r#"
            [package]
            name = "test-template"
            version = "0.1.0"
            entrypoint = "main.typ"

            [tool.oicana]
            manifest_version = 1
            "#,
            "Test",
        );
        let manifest = files.manifest().unwrap();
        let world = OicanaWorld::new(files, TemplateInputs::new(), manifest).unwrap();

        let returned_manifest = world.manifest();

        assert_eq!(returned_manifest.package.name, "test-template");
        assert_eq!(returned_manifest.package.version.to_string(), "0.1.0");
    }
}
