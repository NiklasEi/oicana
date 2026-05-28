//! Lower level WASM bindings for Oicana.
//!
//! You most likely want to use the npm package `@oicana/browser` instead.

use std::collections::HashMap;

use js_sys::Uint8Array;
use log::{trace, warn, Level};
use serde::Deserialize;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use oicana_world::get_current_time;

/// Error string when a requested template is not registered yet. Call `[register_template]` before
/// trying to use the template through a different method.
pub const NOT_REGISTERED: &str = "Template is not registered";

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
    oicana_ffi_core::configure_automatic_cache_eviction(max_age);
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
#[wasm_bindgen]
pub fn evict_cache(max_age: usize) {
    oicana_ffi_core::evict_cache(max_age);
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
    init_logging();
    let start = get_current_time();

    let json_map = decode_json_inputs(json_inputs)?;
    let blob_map = decode_blob_inputs(blob_inputs)?;
    let compilation_mode: CompilationMode = from_value(compilation_mode)
        .map_err(|error| format!("Failed to convert to compilation mode: {error:?}"))?;

    let mut bytes = vec![0; files.length() as usize];
    files.copy_to(&mut bytes[..]);

    let result_id = oicana_ffi_core::register_template(
        &template,
        &bytes,
        json_map,
        blob_map,
        compilation_mode.into(),
    )
    .map_err(|error| error.to_string())?;

    log_warnings(&result_id);
    trace!(
        "Done compiling document in {}ms",
        get_current_time() - start
    );
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
    init_logging();
    let start = get_current_time();

    let json_map = decode_json_inputs(json_inputs)?;
    let blob_map = decode_blob_inputs(blob_inputs)?;
    let compilation_mode: CompilationMode = from_value(compilation_mode)
        .map_err(|error| format!("Failed to convert to compilation mode: {error:?}"))?;

    let result_id =
        oicana_ffi_core::compile_template(&template, json_map, blob_map, compilation_mode.into())
            .map_err(|error| error.to_string())?;

    log_warnings(&result_id);
    trace!(
        "Done preparing document in {}ms",
        get_current_time() - start
    );
    Ok(result_id)
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn inputs(template: String) -> Result<String, String> {
    init_logging();
    oicana_ffi_core::inputs(&template).map_err(|error| error.to_string())
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn get_source(template: String, file: String) -> Result<String, String> {
    oicana_ffi_core::get_source(&template, &file).map_err(|error| error.to_string())
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[wasm_bindgen]
pub fn get_file(template: String, file: String) -> Result<Uint8Array, String> {
    let bytes = oicana_ffi_core::get_file(&template, &file).map_err(|error| error.to_string())?;
    Ok(bytes_to_js_array(&bytes))
}

/// Remove the document from the cache.
#[wasm_bindgen]
pub fn remove_document(document_id: String) -> Result<(), String> {
    oicana_ffi_core::remove_document(&document_id);
    Ok(())
}

/// Return any compilation warnings produced for the given document, or
/// `undefined` if there were none. Warnings are cleared when the document
/// is removed.
#[wasm_bindgen]
pub fn get_warnings(document_id: String) -> Option<String> {
    oicana_ffi_core::get_warnings(&document_id)
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
    oicana_ffi_core::set_validate_inputs(&template, validate).map_err(|error| error.to_string())
}

/// Remove the world from the cache.
///
/// The template will have to be registered again before it can be compiled again.
#[wasm_bindgen]
pub fn remove_world(template_id: String) -> Result<(), String> {
    oicana_ffi_core::remove_world(&template_id);
    Ok(())
}

/// Export the given document
///
/// Make sure to call `removeDocument` with the documentId afterwards, to free the memory.
#[wasm_bindgen]
pub fn export_document(document_id: String, export_format: JsValue) -> Result<Uint8Array, String> {
    let format: oicana_ffi_core::ExportFormat = from_value(export_format)
        .map_err(|error| format!("Failed to convert to export format: {error:?}"))?;
    let start = get_current_time();
    let bytes = oicana_ffi_core::export_document(&document_id, format)
        .map_err(|error| error.to_string())?;
    trace!("Exported document in {}ms", get_current_time() - start);
    Ok(bytes_to_js_array(&bytes))
}

fn init_logging() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(Level::Trace);
}

fn log_warnings(document_id: &str) {
    if let Some(warnings) = oicana_ffi_core::get_warnings(document_id) {
        warn!("{warnings}");
    }
}

fn bytes_to_js_array(bytes: &[u8]) -> Uint8Array {
    let uint8_array = Uint8Array::new_with_length(bytes.len() as u32);
    uint8_array.copy_from(bytes);
    uint8_array
}

fn decode_json_inputs(value: JsValue) -> Result<HashMap<String, String>, String> {
    from_value(value).map_err(|error| {
        format!("Failed to deserialize HashMap<String, String> from JavaScript value: {error:?}")
    })
}

fn decode_blob_inputs(
    value: JsValue,
) -> Result<HashMap<String, oicana_ffi_core::BlobWithMetadata>, String> {
    let blobs: HashMap<String, BlobWithMetadata> = from_value(value).map_err(|error| {
        format!("Failed to deserialize HashMap<String, BlobWithMetadata> from JavaScript value: {error:?}")
    })?;
    blobs
        .into_iter()
        .map(|(key, blob)| {
            let meta = serde_json::to_string(&blob.meta)
                .map_err(|error| format!("Failed to encode metadata for '{key}': {error:?}"))?;
            Ok((
                key,
                oicana_ffi_core::BlobWithMetadata {
                    bytes: blob.bytes,
                    meta,
                },
            ))
        })
        .collect()
}

#[derive(Deserialize)]
enum CompilationMode {
    #[serde(alias = "production")]
    Production,
    #[serde(alias = "development")]
    Development,
}

impl From<CompilationMode> for oicana_ffi_core::CompilationMode {
    fn from(value: CompilationMode) -> Self {
        match value {
            CompilationMode::Development => oicana_ffi_core::CompilationMode::Development,
            CompilationMode::Production => oicana_ffi_core::CompilationMode::Production,
        }
    }
}

#[derive(Deserialize)]
struct BlobWithMetadata {
    bytes: Vec<u8>,
    meta: serde_json::Value,
}
