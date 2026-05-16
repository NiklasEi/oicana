//! Shared core logic for Oicana FFI integrations.
//!
//! Each language integration (node, python, php, java, csharp, browser-wasm) is
//! a thin shim around the functions in this crate. The shim handles marshaling
//! between language-native types and the neutral types exposed here, then calls
//! into the corresponding function in this crate.

use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use thiserror::Error;
use typst::foundations::Bytes;
use typst::layout::PagedDocument;
use typst::syntax::{FileId, VirtualPath};
use uuid::Uuid;

use oicana_export::pdf::export_merged_pdf;
use oicana_export::png::export_merged_png;
use oicana_export::svg::export_merged_svg;
use oicana_files::packed::PackedTemplate;
use oicana_files::TemplateFiles;
use oicana_input::input::blob::{Blob, BlobInput};
use oicana_input::input::json::JsonInput;
use oicana_input::{CompilationConfig, TemplateInputs};
use oicana_world::manifest::OicanaWorldFiles;
use oicana_world::world::OicanaWorld;

/// Diagnostic-output coloring (re-exported from `oicana_world`).
pub use oicana_world::diagnostics::DiagnosticColor;

/// Compilation mode passed in from the calling language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilationMode {
    /// Required inputs that are not provided cause compilation to fail.
    Production,
    /// Required inputs that are not provided fall back to development or default values.
    Development,
}

impl From<CompilationMode> for CompilationConfig {
    fn from(value: CompilationMode) -> Self {
        match value {
            CompilationMode::Production => CompilationConfig::production(),
            CompilationMode::Development => CompilationConfig::development(),
        }
    }
}

/// A blob input value together with its JSON-encoded metadata.
///
/// Integrations should construct one of these per blob input, where `meta`
/// is a JSON string (`"{}"` if no metadata is provided).
pub struct BlobWithMetadata {
    /// Raw bytes of the blob.
    pub bytes: Vec<u8>,
    /// JSON-encoded metadata associated with the blob.
    pub meta: String,
}

/// Export format and parameters for a single document export.
#[derive(Debug, Deserialize)]
#[serde(tag = "format")]
pub enum ExportFormat {
    /// PNG export with configurable resolution.
    #[serde(alias = "png")]
    Png {
        /// Pixels per point used for rendering.
        #[serde(rename = "pixelsPerPt")]
        pixels_per_pt: f32,
    },
    /// PDF export. The exact PDF standard is taken from the template manifest.
    #[serde(alias = "pdf")]
    Pdf,
    /// SVG export.
    #[serde(alias = "svg")]
    Svg,
}

/// All errors that core functions can return.
///
/// Integrations should map this to their language-native error type via
/// `.to_string()` (or via individual variant matching when finer control is desired).
#[derive(Debug, Error)]
pub enum FfiError {
    /// The requested template ID is not present in the world cache.
    #[error("Template '{0}' is not registered")]
    TemplateNotRegistered(String),

    /// The requested document ID is not present in the document cache.
    #[error("Document '{0}' not found")]
    DocumentNotFound(String),

    /// PDF export was requested but the originating template's world is gone.
    #[error("World '{template_id}' for document '{document_id}' not found")]
    WorldForDocumentNotFound {
        /// Template ID that the document was compiled from.
        template_id: String,
        /// Document ID that was requested.
        document_id: String,
    },

    /// The document ID does not match the expected `<uuid>:<template_id>` format.
    #[error("Invalid document ID format: {0}")]
    InvalidDocumentId(String),

    /// The packed template bytes could not be read.
    #[error("Failed to read template: {0}")]
    PackedTemplate(String),

    /// The template manifest could not be parsed.
    #[error("Manifest error: {0}")]
    Manifest(String),

    /// Creating the Typst world for the template failed.
    #[error("World creation error: {0}")]
    WorldCreation(String),

    /// Updating inputs on an existing world failed (typically schema validation).
    #[error("Input validation failed: {0}")]
    InputValidation(String),

    /// Template compilation failed.
    #[error("Compilation failed: {0}")]
    Compilation(String),

    /// Encoding the compiled document to the requested format failed.
    #[error("Failed to encode {format}: {error}")]
    Export {
        /// Human-readable name of the target format ("PDF", "PNG", "SVG").
        format: &'static str,
        /// Underlying encoder error.
        error: String,
    },

    /// Loading the source for a file path failed.
    #[error("Failed to load source '{path}': {error}")]
    SourceLoad {
        /// Requested path inside the template.
        path: String,
        /// Underlying file-system error.
        error: String,
    },

    /// Loading the bytes for a file path failed.
    #[error("Failed to load file '{path}': {error}")]
    FileLoad {
        /// Requested path inside the template.
        path: String,
        /// Underlying file-system error.
        error: String,
    },

    /// Serializing the template's input definitions to JSON failed.
    #[error("Failed to serialize inputs: {0}")]
    InputsSerialization(String),

    /// The blob metadata for a key was not valid JSON or did not match the metadata schema.
    #[error("Failed to parse blob metadata for '{key}': {error}")]
    BlobMetadata {
        /// Blob input key whose metadata failed to parse.
        key: String,
        /// Underlying serde error.
        error: String,
    },

    /// The `export_format` JSON could not be parsed into an [`ExportFormat`].
    #[error("Failed to parse export format: {0}")]
    ExportFormatParse(String),
}

static WORLD_CACHE: Lazy<DashMap<String, OicanaWorld<PackedTemplate>>> = Lazy::new(DashMap::new);
static DOCUMENT_CACHE: Lazy<DashMap<String, PagedDocument>> = Lazy::new(DashMap::new);
static WARNINGS_CACHE: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);

/// Cache age threshold for automatic eviction. `usize::MAX` means disabled.
/// Default keeps entries used within the last 10 evictions.
static CACHE_EVICTION_AGE: AtomicUsize = AtomicUsize::new(10);

/// Global diagnostic-color setting applied to every world created via
/// [`register_template`]. Defaults to `None`.
/// Encoded as 0 = `None`, 1 = `Ansi`.
static DIAGNOSTIC_COLOR: AtomicU8 = AtomicU8::new(0);

fn current_diagnostic_color() -> DiagnosticColor {
    match DIAGNOSTIC_COLOR.load(Ordering::Relaxed) {
        1 => DiagnosticColor::Ansi,
        _ => DiagnosticColor::None,
    }
}

/// Configure the diagnostic-output coloring used for compilation diagnostics.
///
/// The setting is applied immediately to every cached world and used for
/// every world created after this call.
pub fn configure_diagnostic_color(color: DiagnosticColor) {
    DIAGNOSTIC_COLOR.store(
        match color {
            DiagnosticColor::None => 0,
            DiagnosticColor::Ansi => 1,
        },
        Ordering::Relaxed,
    );
    for mut world in WORLD_CACHE.iter_mut() {
        world.color = current_diagnostic_color();
    }
}

/// Parse an `export_format` JSON string into an [`ExportFormat`].
///
/// Integrations that receive the export format as a JSON string (csharp, node,
/// php, python, java) can use this; integrations that deserialize directly
/// from a language-native value (browser-wasm via serde-wasm-bindgen) can
/// build [`ExportFormat`] themselves and skip this.
pub fn parse_export_format(json: &str) -> Result<ExportFormat, FfiError> {
    serde_json::from_str(json).map_err(|error| FfiError::ExportFormatParse(error.to_string()))
}

/// Register a template under the given identifier.
///
/// Reads `files` as a [`PackedTemplate`], builds a Typst world, compiles
/// once with the given inputs as a warm-up, and stores both the world and
/// the resulting document. Returns the document ID for the warm-up result.
///
/// Subsequent calls with the same `template_id` can use
/// [`compile_template`] without re-reading the files.
pub fn register_template(
    template_id: &str,
    files: &[u8],
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, BlobWithMetadata>,
    mode: CompilationMode,
) -> Result<String, FfiError> {
    let packed = PackedTemplate::new(Cursor::new(files))
        .map_err(|error| FfiError::PackedTemplate(error.to_string()))?;
    let manifest = packed
        .manifest()
        .map_err(|error| FfiError::Manifest(error.to_string()))?;

    let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
    inputs.with_config(mode.into());

    let mut world = OicanaWorld::new(packed, inputs, manifest)
        .map_err(|error| FfiError::WorldCreation(error.to_string()))?;
    world.color = current_diagnostic_color();

    let document = world
        .compile()
        .map_err(|error| FfiError::Compilation(error.to_string()))?;

    let result_id = new_document_id(template_id);
    WORLD_CACHE.insert(template_id.to_owned(), world);
    store_warnings(&result_id, document.warnings);
    DOCUMENT_CACHE.insert(result_id.clone(), document.document);

    auto_evict();

    Ok(result_id)
}

/// Compile the previously-registered template with the given inputs.
///
/// Requires a prior successful call to [`register_template`] with the
/// same `template_id`. Returns the document ID of the new compilation.
pub fn compile_template(
    template_id: &str,
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, BlobWithMetadata>,
    mode: CompilationMode,
) -> Result<String, FfiError> {
    let Some(mut world) = WORLD_CACHE.get_mut(template_id) else {
        return Err(FfiError::TemplateNotRegistered(template_id.to_owned()));
    };

    let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
    inputs.with_config(mode.into());
    world
        .update_inputs(inputs)
        .map_err(|error| FfiError::InputValidation(error.to_string()))?;

    let document = world
        .compile()
        .map_err(|error| FfiError::Compilation(error.to_string()))?;

    let result_id = new_document_id(template_id);
    store_warnings(&result_id, document.warnings);
    DOCUMENT_CACHE.insert(result_id.clone(), document.document);

    auto_evict();

    Ok(result_id)
}

/// Compile a template once and immediately export it, without caching the world.
///
/// Useful for one-off compilations where the template will not be reused.
/// For repeated compilations of the same template, use [`register_template`]
/// + [`compile_template`] + [`export_document`] instead.
pub fn compile_once(
    files: &[u8],
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, BlobWithMetadata>,
    mode: CompilationMode,
    format: ExportFormat,
) -> Result<Vec<u8>, FfiError> {
    let packed = PackedTemplate::new(Cursor::new(files))
        .map_err(|error| FfiError::PackedTemplate(error.to_string()))?;
    let manifest = packed
        .manifest()
        .map_err(|error| FfiError::Manifest(error.to_string()))?;

    let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
    inputs.with_config(mode.into());

    let mut world = OicanaWorld::new(packed, inputs, manifest)
        .map_err(|error| FfiError::WorldCreation(error.to_string()))?;
    world.color = current_diagnostic_color();

    let document = world
        .compile()
        .map_err(|error| FfiError::Compilation(error.to_string()))?;

    auto_evict();

    encode_document(&document.document, &world, format)
}

/// Export a previously-compiled document.
///
/// PDF export needs access to the originating world (for embedded fonts and
/// PDF standards configuration), so the world for the document's template
/// must still be present in the cache.
pub fn export_document(document_id: &str, format: ExportFormat) -> Result<Vec<u8>, FfiError> {
    let Some(document) = DOCUMENT_CACHE.get(document_id) else {
        return Err(FfiError::DocumentNotFound(document_id.to_owned()));
    };

    match format {
        ExportFormat::Png { pixels_per_pt } => {
            export_merged_png(&document, pixels_per_pt).map_err(|error| FfiError::Export {
                format: "PNG",
                error: format!("{error:?}"),
            })
        }
        ExportFormat::Pdf => {
            let template_id = template_id_from_document_id(document_id)?;
            let Some(world) = WORLD_CACHE.get(template_id) else {
                return Err(FfiError::WorldForDocumentNotFound {
                    template_id: template_id.to_owned(),
                    document_id: document_id.to_owned(),
                });
            };
            export_merged_pdf(&document, &*world, world.manifest().pdf_standards()).map_err(
                |error| FfiError::Export {
                    format: "PDF",
                    error: format!("{error:?}"),
                },
            )
        }
        ExportFormat::Svg => Ok(export_merged_svg(&document)),
    }
}

/// Return the template's input definitions serialized as a JSON string.
pub fn inputs(template_id: &str) -> Result<String, FfiError> {
    let Some(world) = WORLD_CACHE.get(template_id) else {
        return Err(FfiError::TemplateNotRegistered(template_id.to_owned()));
    };
    serde_json::to_string(&world.manifest().tool.oicana)
        .map_err(|error| FfiError::InputsSerialization(error.to_string()))
}

/// Return the source text of a file inside the template.
pub fn get_source(template_id: &str, path: &str) -> Result<String, FfiError> {
    let Some(world) = WORLD_CACHE.get(template_id) else {
        return Err(FfiError::TemplateNotRegistered(template_id.to_owned()));
    };
    world
        .files
        .source(FileId::new(None, VirtualPath::new(path)))
        .map(|source| source.text().to_string())
        .map_err(|error| FfiError::SourceLoad {
            path: path.to_owned(),
            error: error.to_string(),
        })
}

/// Return the raw bytes of a file inside the template.
pub fn get_file(template_id: &str, path: &str) -> Result<Vec<u8>, FfiError> {
    let Some(world) = WORLD_CACHE.get(template_id) else {
        return Err(FfiError::TemplateNotRegistered(template_id.to_owned()));
    };
    world
        .files
        .file(FileId::new(None, VirtualPath::new(path)))
        .map(|bytes| bytes.to_vec())
        .map_err(|error| FfiError::FileLoad {
            path: path.to_owned(),
            error: error.to_string(),
        })
}

/// Enable or disable JSON schema validation of inputs for a registered template.
///
/// When enabled (the default), JSON inputs are validated against their schemas
/// before compilation.
pub fn set_validate_inputs(template_id: &str, validate: bool) -> Result<(), FfiError> {
    let Some(mut world) = WORLD_CACHE.get_mut(template_id) else {
        return Err(FfiError::TemplateNotRegistered(template_id.to_owned()));
    };
    world.validate_inputs = validate;
    Ok(())
}

/// Drop the cached document (and any warnings stored alongside it), freeing
/// its memory.
pub fn remove_document(document_id: &str) {
    DOCUMENT_CACHE.remove(document_id);
    WARNINGS_CACHE.remove(document_id);
}

/// Return any compilation warnings produced for the given document.
///
/// Warnings are stored when [`register_template`] or [`compile_template`]
/// successfully produce a document. They are cleared together with the
/// document by [`remove_document`].
pub fn get_warnings(document_id: &str) -> Option<String> {
    WARNINGS_CACHE.get(document_id).map(|entry| entry.clone())
}

fn store_warnings(document_id: &str, warnings: Option<String>) {
    if let Some(warnings) = warnings {
        WARNINGS_CACHE.insert(document_id.to_owned(), warnings);
    }
}

/// Drop the cached world for a template. Subsequent compilations require
/// a fresh [`register_template`] call.
pub fn remove_world(template_id: &str) {
    WORLD_CACHE.remove(template_id);
}

/// Configure automatic comemo cache eviction after each compilation.
///
/// `max_age` semantics:
///   - `None` – disable automatic eviction (the cache is never cleared).
///   - `Some(0)` – clear every cache entry on each eviction.
///   - `Some(1)` – keep only entries used since the last eviction.
///   - `Some(n)` – keep entries used within the last `n` evictions.
///
/// The default is `Some(10)`.
pub fn configure_automatic_cache_eviction(max_age: Option<usize>) {
    CACHE_EVICTION_AGE.store(max_age.unwrap_or(usize::MAX), Ordering::Relaxed);
}

/// Manually evict the comemo cache with the given age threshold, regardless
/// of the automatic eviction setting.
pub fn evict_cache(max_age: usize) {
    oicana_world::evict_cache(max_age);
}

fn auto_evict() {
    let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
    if cache_age != usize::MAX {
        oicana_world::evict_cache(cache_age);
    }
}

fn new_document_id(template_id: &str) -> String {
    format!("{}:{}", Uuid::new_v4(), template_id)
}

/// UUID v4 string representations are 36 chars, so a document ID is at least
/// 38 chars (`<uuid>:<at-least-one-char>`).
const DOCUMENT_ID_UUID_LEN: usize = 36;

fn template_id_from_document_id(document_id: &str) -> Result<&str, FfiError> {
    let bytes = document_id.as_bytes();
    if bytes.len() <= DOCUMENT_ID_UUID_LEN + 1 || bytes[DOCUMENT_ID_UUID_LEN] != b':' {
        return Err(FfiError::InvalidDocumentId(document_id.to_owned()));
    }
    Ok(&document_id[DOCUMENT_ID_UUID_LEN + 1..])
}

fn prepare_inputs(
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, BlobWithMetadata>,
) -> Result<TemplateInputs, FfiError> {
    let mut inputs = TemplateInputs::new();

    for (key, value) in json_inputs {
        inputs.with_input(JsonInput::new(key, value));
    }

    for (key, value) in blob_inputs {
        let mut blob = Blob::from(Bytes::new(value.bytes));
        let parsed =
            serde_json::Value::from_str(&value.meta).map_err(|error| FfiError::BlobMetadata {
                key: key.clone(),
                error: error.to_string(),
            })?;
        blob.metadata =
            Deserialize::deserialize(parsed).map_err(|error| FfiError::BlobMetadata {
                key: key.clone(),
                error: error.to_string(),
            })?;
        inputs.with_input(BlobInput::new(key, blob));
    }

    Ok(inputs)
}

fn encode_document(
    document: &PagedDocument,
    world: &OicanaWorld<PackedTemplate>,
    format: ExportFormat,
) -> Result<Vec<u8>, FfiError> {
    match format {
        ExportFormat::Png { pixels_per_pt } => {
            export_merged_png(document, pixels_per_pt).map_err(|error| FfiError::Export {
                format: "PNG",
                error: format!("{error:?}"),
            })
        }
        ExportFormat::Pdf => export_merged_pdf(document, world, world.manifest().pdf_standards())
            .map_err(|error| FfiError::Export {
                format: "PDF",
                error: format!("{error:?}"),
            }),
        ExportFormat::Svg => Ok(export_merged_svg(document)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_export_format_pdf() {
        let format = parse_export_format(r#"{"format":"pdf"}"#).unwrap();
        assert!(matches!(format, ExportFormat::Pdf));
    }

    #[test]
    fn parses_export_format_png_with_resolution() {
        let format = parse_export_format(r#"{"format":"png","pixelsPerPt":2.0}"#).unwrap();
        match format {
            ExportFormat::Png { pixels_per_pt } => assert_eq!(pixels_per_pt, 2.0),
            other => panic!("expected PNG, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_export_format() {
        let err = parse_export_format(r#"{"format":"docx"}"#).unwrap_err();
        assert!(matches!(err, FfiError::ExportFormatParse(_)));
    }

    #[test]
    fn template_id_round_trips_through_document_id() {
        let doc_id = new_document_id("some-template");
        assert_eq!(
            template_id_from_document_id(&doc_id).unwrap(),
            "some-template"
        );
    }

    #[test]
    fn rejects_document_id_without_separator() {
        let err = template_id_from_document_id("not-a-document-id").unwrap_err();
        assert!(matches!(err, FfiError::InvalidDocumentId(_)));
    }

    #[test]
    fn rejects_document_id_without_template_id_suffix() {
        let only_uuid = format!("{}", Uuid::new_v4());
        let err = template_id_from_document_id(&only_uuid).unwrap_err();
        assert!(matches!(err, FfiError::InvalidDocumentId(_)));

        let trailing_colon = format!("{}:", Uuid::new_v4());
        let err = template_id_from_document_id(&trailing_colon).unwrap_err();
        assert!(matches!(err, FfiError::InvalidDocumentId(_)));
    }
}
