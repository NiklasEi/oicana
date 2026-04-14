//! Lower level WASM bindings for Oicana.
//!
//! You most likely want to use the npm package `@oicana/browser` instead.

use dashmap::DashMap;
use js_sys::Uint8Array;
use log::{info, warn, Level};
use oicana_export::pdf::export_merged_pdf;
use oicana_export::png::export_merged_png;
use oicana_export::svg::export_merged_svg;
use oicana_files::packed::PackedTemplate;
use oicana_files::TemplateFiles;
use oicana_input::input::blob::{Blob, BlobInput};
use oicana_input::input::json::JsonInput;
use oicana_input::{CompilationConfig, TemplateInputs};
use oicana_world::diagnostics::DiagnosticColor;
use oicana_world::get_current_time;
use oicana_world::manifest::OicanaWorldFiles;
use oicana_world::world::OicanaWorld;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_wasm_bindgen::from_value;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use typst::foundations::Bytes;
use typst::layout::PagedDocument;
use typst::syntax::{FileId, VirtualPath};
use uuid::Uuid;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// Error string when a requested template is not registered yet. Call `[register_template]` before
/// trying to use the template through a different method.
pub const NOT_REGISTERED: &str = "Template is not registered";

/// Global cache age configuration.
///
/// Default is 10, meaning cache entries used during the last 10 eviction cycles are kept.
/// usize::MAX is used internally to represent disabled eviction.
static CACHE_EVICTION_AGE: AtomicUsize = AtomicUsize::new(10);

/// Configure automatic cache eviction after each compilation.
///
/// # Parameters
///
/// `max_age` (start value: 10) - Maximum age threshold, or null to disable:
///   - `null` - Disables cache eviction (cache never cleared)
///   - `0` - Clears all cache entries with every eviction
///   - `1` - Keeps only entries used since the last eviction
///   - `n` - Keeps entries used within the last n evictions
#[wasm_bindgen]
pub fn configure_automatic_cache_eviction(max_age: Option<usize>) {
    CACHE_EVICTION_AGE.store(max_age.unwrap_or(usize::MAX), Ordering::Relaxed);
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
#[wasm_bindgen]
pub fn evict_cache(max_age: usize) {
    oicana_world::evict_cache(max_age);
}

/// Register the given template. This will read the template as a [`PackedTemplate`] and compile it
/// once with the given inputs. The Typst [`typst::World`] will be cached and reused for subsequent
/// calls to the other methods with the same template identifier.
#[wasm_bindgen]
pub fn register_template(
    template: String,
    files: &Uint8Array,
    json_inputs: JsValue,
    blob_inputs: JsValue,
    compilation_mode: JsValue,
) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(Level::Debug);
    let start = get_current_time();

    let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
    let compilation_mode: CompilationMode = from_value(compilation_mode)
        .map_err(|error| format!("Failed to convert to compilation mode: {error:?}"))?;
    inputs.with_config(compilation_mode.into());

    let mut vec = vec![0; files.length() as usize];
    files.copy_to(&mut vec[..]);
    let files = PackedTemplate::new(Cursor::new(vec))
        .map_err(|error| format!("Failed to read template: {error}"))?;
    let manifest = files.manifest().map_err(|error| format!("{error:?}"))?;

    let mut world =
        OicanaWorld::new(files, inputs, manifest).map_err(|error| format!("{error:?}"))?;
    world.color = DiagnosticColor::None;

    let document = world.compile().map_err(|error| error.to_string())?;
    let document_time = get_current_time();
    info!("Done compiling document in {}ms", document_time - start);

    let result_id = new_document_id(&template);

    WORLD_CACHE.insert(template, world);
    DOCUMENT_CACHE.insert(result_id.clone(), document.document);

    let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
    if cache_age != usize::MAX {
        oicana_world::evict_cache(cache_age);
    }

    Ok(result_id)
}

/// Compile the identified template with the given inputs.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn compile_template(
    template: String,
    json_inputs: JsValue,
    blob_inputs: JsValue,
    compilation_mode: JsValue,
) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(Level::Debug);
    let start = get_current_time();

    let Some(mut world) = WORLD_CACHE.get_mut(&template) else {
        return Err(NOT_REGISTERED.to_owned());
    };
    let compilation_mode: CompilationMode = from_value(compilation_mode)
        .map_err(|error| format!("Failed to convert to compilation mode: {error:?}"))?;
    let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
    inputs.with_config(compilation_mode.into());
    world
        .update_inputs(inputs)
        .map_err(|error| error.to_string())?;

    let document = world.compile().map_err(|error| error.to_string())?;
    let document_time = get_current_time();
    if let Some(warnings) = document.warnings {
        warn!("{warnings}");
    }
    info!("Done preparing document in {}ms", document_time - start);

    let result_id = new_document_id(&template);
    DOCUMENT_CACHE.insert(result_id.clone(), document.document);

    let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
    if cache_age != usize::MAX {
        oicana_world::evict_cache(cache_age);
    }

    Ok(result_id)
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn inputs(template: String) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(Level::Debug);

    let Some(world) = WORLD_CACHE.get(&template) else {
        return Err(NOT_REGISTERED.to_owned());
    };

    let manifest_result = world.files.manifest();

    let manifest = manifest_result.map_err(|error| format!("{error:?}"))?;
    let template_def = manifest.tool.oicana;

    serde_json::ser::to_string(&template_def).map_err(|error| format!("{error:?}"))
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn get_source(template: String, file: String) -> Result<String, String> {
    let Some(world) = WORLD_CACHE.get(&template) else {
        return Err(NOT_REGISTERED.to_owned());
    };
    world
        .files
        .source(FileId::new(None, VirtualPath::new(file)))
        .map_err(|error| format!("{error}"))
        .map(|source| source.text().to_string())
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn get_file(template: String, file: String) -> Result<Uint8Array, String> {
    let Some(world) = WORLD_CACHE.get(&template) else {
        return Err(NOT_REGISTERED.to_owned());
    };
    let bytes = world
        .files
        .file(FileId::new(None, VirtualPath::new(file)))
        .map_err(|error| format!("{error}"))?;
    let array = Uint8Array::new_with_length(bytes.len() as u32);
    array.copy_from(&bytes);
    Ok(array)
}

fn prepare_inputs(json_inputs: JsValue, blobs: JsValue) -> Result<TemplateInputs, String> {
    let start = get_current_time();
    let mut inputs = TemplateInputs::new();
    add_blobs(&mut inputs, blobs)?;
    let blob_time = get_current_time();
    info!("Created blob map in {}ms", blob_time - start);
    add_json_inputs(&mut inputs, json_inputs)?;
    let inputs_time = get_current_time();
    info!("Created json input map in {}ms", inputs_time - blob_time);

    Ok(inputs)
}

/// Remove the document from the cache.
#[wasm_bindgen]
pub fn remove_document(document_id: String) -> Result<(), String> {
    DOCUMENT_CACHE.remove(&document_id);
    Ok(())
}

/// Enable or disable JSON schema validation for the given template.
///
/// When enabled (the default), JSON inputs are validated against their schemas
/// before compilation.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn set_validate_inputs(template: String, validate: bool) -> Result<(), String> {
    let Some(mut world) = WORLD_CACHE.get_mut(&template) else {
        return Err(NOT_REGISTERED.to_owned());
    };
    world.validate_inputs = validate;
    Ok(())
}

/// Remove the world from the cache.
///
/// The template will have to be registered again before it can be compiled again.
#[wasm_bindgen]
pub fn remove_world(template_id: String) -> Result<(), String> {
    WORLD_CACHE.remove(&template_id);
    Ok(())
}

fn new_document_id(template_id: &str) -> String {
    format!("{}:{}", Uuid::new_v4(), template_id)
}

fn template_id_from_document_id(document_id: &str) -> Result<&str, String> {
    if document_id.len() <= 37 {
        return Err(format!(
            "Invalid document ID format (length {}): {}",
            document_id.len(),
            document_id
        ));
    }
    if let Some(colon_idx) = document_id.find(':') {
        if colon_idx == 36 {
            return Ok(&document_id[37..]);
        }
    }
    Err(format!(
        "Invalid document ID format (no colon at position 36): {}",
        document_id
    ))
}

/// Export the given document
///
/// Make sure to call `removeDocument` with the documentId afterwards, to free the memory.
#[wasm_bindgen]
pub fn export_document(document_id: String, export_format: JsValue) -> Result<Uint8Array, String> {
    let Some(document) = DOCUMENT_CACHE.get(&document_id) else {
        return Err("Document not found!".to_owned());
    };
    let export_format: ExportFormat = from_value(export_format)
        .map_err(|error| format!("Failed to convert to export format: {error:?}"))?;
    match export_format {
        ExportFormat::Png { pixels_per_pt } => {
            let start_time = get_current_time();
            let pix_map_result = export_merged_png(&document, pixels_per_pt);
            info!("Rendered image in {}ms", get_current_time() - start_time);
            pix_map_result
                .map_err(|error| format!("Failed to encode PNG: {error:?}"))
                .map(|pix_map| bytes_to_js_array(&pix_map))
        }
        ExportFormat::Pdf => {
            let template_id = template_id_from_document_id(&document_id)?;
            let Some(world) = WORLD_CACHE.get(template_id) else {
                return Err(format!(
                    "World '{template_id}' for the given document '{document_id}' not found!"
                ));
            };

            export_merged_pdf(&document, &*world, world.manifest().pdf_standards())
                .map(|pdf| bytes_to_js_array(&pdf))
        }
        ExportFormat::Svg => {
            let svg = export_merged_svg(&document);

            Ok(bytes_to_js_array(&svg))
        }
    }
}

fn bytes_to_js_array(bytes: &[u8]) -> Uint8Array {
    let uint8_array = Uint8Array::new_with_length(bytes.len() as u32);
    uint8_array.copy_from(bytes);

    uint8_array
}

#[derive(Deserialize)]
enum CompilationMode {
    #[serde(alias = "production")]
    Production,
    #[serde(alias = "development")]
    Development,
}

impl From<CompilationMode> for oicana_input::CompilationConfig {
    fn from(value: CompilationMode) -> Self {
        match value {
            CompilationMode::Development => CompilationConfig::development(),
            CompilationMode::Production => CompilationConfig::production(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "format")]
enum ExportFormat {
    #[serde(alias = "png")]
    Png { pixels_per_pt: f32 },
    #[serde(alias = "pdf")]
    Pdf,
    #[serde(alias = "svg")]
    Svg,
}

#[derive(Deserialize)]
struct BlobWithMetadata {
    bytes: Vec<u8>,
    meta: serde_json::Value,
}

fn add_blobs(inputs: &mut TemplateInputs, blobs: JsValue) -> Result<(), String> {
    let blobs: HashMap<String, BlobWithMetadata> = from_value(blobs)
        .map_err(|error| format!("Failed to deserialize HashMap<String, BlobWithMetadata> from JavaScript value: {error:?}"))?;
    for (key, value) in blobs {
        let mut blob = Blob::from(Bytes::new(value.bytes.to_vec()));
        blob.metadata = Deserialize::deserialize(value.meta)
            .map_err(|error| format!("Failed to deserialize from JSON value: {error:?}"))?;
        inputs.with_input(BlobInput::new(key, blob));
    }

    Ok(())
}

fn add_json_inputs(inputs: &mut TemplateInputs, json_inputs: JsValue) -> Result<(), String> {
    let json_inputs: HashMap<String, String> = from_value(json_inputs).map_err(|error| {
        format!("Failed to deserialize HashMap<String, String> from JavaScript value: {error:?}")
    })?;
    json_inputs
        .into_iter()
        .map(|(key, value)| JsonInput::new(key, value))
        .for_each(|input| {
            inputs.with_input(input);
        });

    Ok(())
}

static WORLD_CACHE: Lazy<DashMap<String, OicanaWorld<PackedTemplate>>> = Lazy::new(DashMap::new);

static DOCUMENT_CACHE: Lazy<DashMap<String, PagedDocument>> = Lazy::new(DashMap::new);
