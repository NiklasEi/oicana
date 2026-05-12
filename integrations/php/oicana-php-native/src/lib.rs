//! The PHP integration of Oicana.
//!
//! You will want to use this through the PHP package `oicana/oicana`.

// Allow missing docs for generated PHP bindings
// Documentation is provided in the PHP wrapper package
#![allow(missing_docs)]
// Required by ext_php_rs
#![cfg_attr(windows, feature(abi_vectorcall))]

use dashmap::DashMap;
use ext_php_rs::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use oicana_export::pdf::export_merged_pdf;
use oicana_export::png::export_merged_png;
use oicana_export::svg::export_merged_svg;
use oicana_files::TemplateFiles;
use oicana_files::packed::PackedTemplate;
use oicana_input::input::blob::{Blob, BlobInput};
use oicana_input::input::json::JsonInput;
use oicana_input::{CompilationConfig, TemplateInputs};
use oicana_world::diagnostics::DiagnosticColor;
use oicana_world::manifest::OicanaWorldFiles;
use oicana_world::world::OicanaWorld;
use typst::foundations::Bytes;
use typst::layout::PagedDocument;
use typst::syntax::{FileId, VirtualPath};

static WORLD_CACHE: Lazy<DashMap<String, OicanaWorld<PackedTemplate>>> = Lazy::new(DashMap::new);
static DOCUMENT_CACHE: Lazy<DashMap<String, PagedDocument>> = Lazy::new(DashMap::new);

/// Global cache age configuration.
///
/// Default is 10, meaning cache entries used during the last 10 eviction cycles are kept.
/// usize::MAX is used internally to represent disabled eviction.
static CACHE_EVICTION_AGE: AtomicUsize = AtomicUsize::new(10);

/// Configure automatic cache eviction after each compilation.
///
/// # Parameters
///
/// `max_age` - Maximum age threshold, or null to disable:
///   - `null` - Disables cache eviction (cache never cleared)
///   - `0` - Clears all cache entries with every eviction
///   - `1` - Keeps only entries used since the last eviction
///   - `n` - Keeps entries used within the last n evictions
#[php_function]
#[php(name = "OicanaInternal\\configure_automatic_cache_eviction")]
pub fn configure_automatic_cache_eviction(max_age: Option<i64>) {
    let age = match max_age {
        Some(age) if age >= 0 => age as usize,
        _ => usize::MAX,
    };
    CACHE_EVICTION_AGE.store(age, Ordering::Relaxed);
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
#[php_function]
#[php(name = "OicanaInternal\\evict_cache")]
pub fn evict_cache(max_age: i64) {
    if max_age >= 0 {
        oicana_world::evict_cache(max_age as usize);
    }
}

/// Compilation mode constant for production mode.
///
/// In production mode, all required inputs must be explicitly provided.
pub const COMPILATION_MODE_PRODUCTION: i64 = 0;

/// Compilation mode constant for development mode.
///
/// In development mode, default and development values from the template are used
/// when inputs are not explicitly provided.
pub const COMPILATION_MODE_DEVELOPMENT: i64 = 1;

fn compilation_mode_from_i64(mode: i64) -> CompilationConfig {
    match mode {
        0 => CompilationConfig::production(),
        _ => CompilationConfig::development(),
    }
}

/// Blob input with associated metadata.
///
/// This class is used to pass binary data (images, fonts, etc.) along with
/// JSON-encoded metadata to template inputs.
#[php_class]
#[php(name = "OicanaInternal\\BlobWithMetadata")]
#[derive(Debug, Clone)]
pub struct BlobWithMetadata {
    /// The raw binary data of the blob.
    #[php(prop)]
    pub bytes: Vec<u8>,
    /// JSON-encoded metadata associated with the blob.
    #[php(prop)]
    pub meta: String,
}

#[php_impl]
impl BlobWithMetadata {
    /// Creates a new BlobWithMetadata instance.
    pub fn __construct(bytes: Vec<u8>, meta: String) -> Self {
        Self { bytes, meta }
    }
}

/// Register the given template. This will read the template files as a PackedTemplate and
/// compile it once with the given inputs. The Typst World will be cached and reused for
/// subsequent calls to the other methods with the same template identifier.
#[php_function]
#[php(name = "OicanaInternal\\register_template")]
pub fn register_template(
    template: String,
    files: Vec<u8>,
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, &BlobWithMetadata>,
    compilation_mode: i64,
) -> PhpResult<String> {
    let packed = PackedTemplate::new(Cursor::new(&files))
        .map_err(|e| PhpException::default(e.to_string()))?;

    let manifest = packed
        .manifest()
        .map_err(|e| PhpException::default(e.to_string()))?;

    let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
    inputs.with_config(compilation_mode_from_i64(compilation_mode));

    let mut zip_world = OicanaWorld::new(packed, inputs, manifest)
        .map_err(|e| PhpException::default(e.to_string()))?;
    zip_world.color = DiagnosticColor::None;

    let document = zip_world
        .compile()
        .map_err(|e| PhpException::default(e.to_string()))?;

    let result_id = new_document_id(&template);

    WORLD_CACHE.insert(template, zip_world);
    DOCUMENT_CACHE.insert(result_id.clone(), document.document);

    let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
    if cache_age != usize::MAX {
        oicana_world::evict_cache(cache_age);
    }

    Ok(result_id)
}

/// Compile the identified template with the given inputs.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[php_function]
#[php(name = "OicanaInternal\\compile_template")]
pub fn compile_template(
    template: String,
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, &BlobWithMetadata>,
    compilation_mode: i64,
) -> PhpResult<String> {
    let Some(mut world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PhpException::default(
            "Template was not registered".to_string(),
        ));
    };

    let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
    inputs.with_config(compilation_mode_from_i64(compilation_mode));
    world
        .update_inputs(inputs)
        .map_err(|e| PhpException::default(e.to_string()))?;

    let document = world
        .compile()
        .map_err(|e| PhpException::default(e.to_string()))?;

    let result_id = new_document_id(&template);
    DOCUMENT_CACHE.insert(result_id.clone(), document.document);

    let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
    if cache_age != usize::MAX {
        oicana_world::evict_cache(cache_age);
    }

    Ok(result_id)
}

fn new_document_id(template_id: &str) -> String {
    format!("{}:{}", uuid::Uuid::new_v4(), template_id)
}

fn template_id_from_document_id(document_id: &str) -> Result<&str, String> {
    if document_id.len() <= 37 {
        return Err(format!(
            "Invalid document ID format (length {}): {document_id}",
            document_id.len()
        ));
    }
    if let Some(colon_idx) = document_id.find(':') {
        if colon_idx == 36 {
            return Ok(&document_id[37..]);
        }
    }
    Err(format!(
        "Invalid document ID format (no colon at position 36): {document_id}"
    ))
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[php_function]
#[php(name = "OicanaInternal\\inputs")]
pub fn inputs(template: String) -> PhpResult<String> {
    let Some(world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PhpException::default(
            "Template was not registered".to_string(),
        ));
    };
    let oicana_config = &world.manifest().tool.oicana;

    serde_json::ser::to_string(&oicana_config).map_err(|e| PhpException::default(e.to_string()))
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[php_function]
#[php(name = "OicanaInternal\\get_source")]
pub fn get_source(template: String, file: String) -> PhpResult<String> {
    let Some(world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PhpException::default(
            "Template was not registered".to_string(),
        ));
    };
    world
        .files
        .source(FileId::new(None, VirtualPath::new(file)))
        .map_err(|e| PhpException::default(e.to_string()))
        .map(|source| source.text().to_string())
}

/// Load the binary file content from the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[php_function]
#[php(name = "OicanaInternal\\get_file")]
pub fn get_file(template: String, file: String) -> PhpResult<Vec<u8>> {
    let Some(world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PhpException::default(
            "Template was not registered".to_string(),
        ));
    };
    let bytes = world
        .files
        .file(FileId::new(None, VirtualPath::new(file)))
        .map_err(|e| PhpException::default(e.to_string()))?;

    Ok(bytes.to_vec())
}

/// Export the given document
///
/// Make sure to call `remove_document` with the document_id afterwards, to free the memory.
#[php_function]
#[php(name = "OicanaInternal\\export_document")]
pub fn export_document(document_id: String, export_format: String) -> PhpResult<Vec<u8>> {
    let Some(document) = DOCUMENT_CACHE.get(&document_id) else {
        return Err(PhpException::default("Document not found!".to_string()));
    };

    let export_format: ExportFormat =
        serde_json::from_str(&export_format).map_err(|e| PhpException::default(e.to_string()))?;

    let bytes = match export_format {
        ExportFormat::Png { pixels_per_pt } => export_merged_png(&document, pixels_per_pt)
            .map_err(|e| PhpException::default(format!("Failed to encode PNG: {e:?}")))?,
        ExportFormat::Pdf => {
            let template_id =
                template_id_from_document_id(&document_id).map_err(PhpException::default)?;
            let Some(world) = WORLD_CACHE.get(template_id) else {
                return Err(PhpException::default(format!(
                    "World '{template_id}' for the given document '{document_id}' not found!"
                )));
            };

            export_merged_pdf(&document, &*world, world.manifest().pdf_standards())
                .map_err(|e| PhpException::default(format!("Failed to encode PDF: {e:?}")))?
        }
        ExportFormat::Svg => export_merged_svg(&document),
    };

    Ok(bytes)
}

/// Remove the document from the cache.
#[php_function]
#[php(name = "OicanaInternal\\remove_document")]
pub fn remove_document(document_id: String) -> PhpResult<()> {
    DOCUMENT_CACHE.remove(&document_id);
    Ok(())
}

/// Enable or disable JSON schema validation for the given template.
///
/// When enabled (the default), JSON inputs are validated against their schemas
/// before compilation.
#[php_function]
#[php(name = "OicanaInternal\\set_validate_inputs")]
pub fn set_validate_inputs(template: String, validate: bool) -> PhpResult<()> {
    let Some(mut world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PhpException::default(
            "Template was not registered".to_string(),
        ));
    };
    world.validate_inputs = validate;
    Ok(())
}

/// Remove the world from the cache.
///
/// The template will have to be registered again before it can be compiled again.
#[php_function]
#[php(name = "OicanaInternal\\remove_world")]
pub fn remove_world(template_id: String) -> PhpResult<()> {
    WORLD_CACHE.remove(&template_id);
    Ok(())
}

fn prepare_inputs(
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, &BlobWithMetadata>,
) -> PhpResult<TemplateInputs> {
    let mut inputs = TemplateInputs::new();

    for (key, value) in json_inputs {
        inputs.with_input(JsonInput::new(key, value));
    }

    for (key, value) in blob_inputs {
        let bytes_vec = value.bytes.clone();
        let mut blob = Blob::from(Bytes::new(bytes_vec));

        blob.metadata =
            Deserialize::deserialize(serde_json::Value::from_str(&value.meta).map_err(|e| {
                PhpException::default(format!("Failed to parse metadata JSON: {e:?}"))
            })?)
            .map_err(|e| PhpException::default(format!("Failed to deserialize metadata: {e:?}")))?;

        inputs.with_input(BlobInput::new(key, blob));
    }

    Ok(inputs)
}

/// Export format configuration for document rendering.
#[derive(Deserialize)]
#[serde(tag = "format")]
enum ExportFormat {
    /// PNG export with configurable resolution.
    #[serde(alias = "png")]
    Png {
        /// Pixels per point for PNG rendering resolution.
        #[serde(rename = "pixelsPerPt")]
        pixels_per_pt: f32,
    },
    /// PDF export format.
    #[serde(alias = "pdf")]
    Pdf,
    /// SVG export format.
    #[serde(alias = "svg")]
    Svg,
}

/// Registers the PHP module with ext-php-rs.
#[php_module]
#[php(startup = startup_function)]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .name("oicana")
        .function(wrap_function!(configure_automatic_cache_eviction))
        .function(wrap_function!(evict_cache))
        .function(wrap_function!(register_template))
        .function(wrap_function!(compile_template))
        .function(wrap_function!(inputs))
        .function(wrap_function!(get_source))
        .function(wrap_function!(get_file))
        .function(wrap_function!(export_document))
        .function(wrap_function!(remove_document))
        .function(wrap_function!(remove_world))
        .function(wrap_function!(set_validate_inputs))
        .class::<BlobWithMetadata>()
}

fn startup_function(_ty: i32, _mod_num: i32) -> i32 {
    0
}
