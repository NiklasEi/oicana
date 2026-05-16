//! The PHP integration of Oicana.
//!
//! You will want to use this through the PHP package `oicana/oicana`.

// Allow missing docs for generated PHP bindings
// Documentation is provided in the PHP wrapper package
#![allow(missing_docs)]
// Required by ext_php_rs
#![cfg_attr(windows, feature(abi_vectorcall))]

use std::collections::HashMap;

use ext_php_rs::prelude::*;

use oicana_ffi_core as core;

/// Compilation mode constant for production mode.
///
/// In production mode, all required inputs must be explicitly provided.
pub const COMPILATION_MODE_PRODUCTION: i64 = 0;

/// Compilation mode constant for development mode.
///
/// In development mode, default and development values from the template are used
/// when inputs are not explicitly provided.
pub const COMPILATION_MODE_DEVELOPMENT: i64 = 1;

fn compilation_mode_from_i64(mode: i64) -> core::CompilationMode {
    match mode {
        0 => core::CompilationMode::Production,
        _ => core::CompilationMode::Development,
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
    let max_age = max_age.and_then(|age| usize::try_from(age).ok());
    core::configure_automatic_cache_eviction(max_age);
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
#[php_function]
#[php(name = "OicanaInternal\\evict_cache")]
pub fn evict_cache(max_age: i64) {
    if let Ok(max_age) = usize::try_from(max_age) {
        core::evict_cache(max_age);
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
    core::register_template(
        &template,
        &files,
        json_inputs,
        into_core_blobs(blob_inputs),
        compilation_mode_from_i64(compilation_mode),
    )
    .map_err(into_php_err)
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
    core::compile_template(
        &template,
        json_inputs,
        into_core_blobs(blob_inputs),
        compilation_mode_from_i64(compilation_mode),
    )
    .map_err(into_php_err)
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[php_function]
#[php(name = "OicanaInternal\\inputs")]
pub fn inputs(template: String) -> PhpResult<String> {
    core::inputs(&template).map_err(into_php_err)
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[php_function]
#[php(name = "OicanaInternal\\get_source")]
pub fn get_source(template: String, file: String) -> PhpResult<String> {
    core::get_source(&template, &file).map_err(into_php_err)
}

/// Load the binary file content from the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[php_function]
#[php(name = "OicanaInternal\\get_file")]
pub fn get_file(template: String, file: String) -> PhpResult<Vec<u8>> {
    core::get_file(&template, &file).map_err(into_php_err)
}

/// Export the given document
///
/// Make sure to call `remove_document` with the document_id afterwards, to free the memory.
#[php_function]
#[php(name = "OicanaInternal\\export_document")]
pub fn export_document(document_id: String, export_format: String) -> PhpResult<Vec<u8>> {
    let format = core::parse_export_format(&export_format).map_err(into_php_err)?;
    core::export_document(&document_id, format).map_err(into_php_err)
}

/// Remove the document from the cache.
#[php_function]
#[php(name = "OicanaInternal\\remove_document")]
pub fn remove_document(document_id: String) -> PhpResult<()> {
    core::remove_document(&document_id);
    Ok(())
}

/// Return any compilation warnings produced for the given document, or `null`
/// if there were none. Warnings are cleared when the document is removed.
#[php_function]
#[php(name = "OicanaInternal\\get_warnings")]
pub fn get_warnings(document_id: String) -> Option<String> {
    core::get_warnings(&document_id)
}

/// Enable or disable JSON schema validation for the given template.
///
/// When enabled (the default), JSON inputs are validated against their schemas
/// before compilation.
#[php_function]
#[php(name = "OicanaInternal\\set_validate_inputs")]
pub fn set_validate_inputs(template: String, validate: bool) -> PhpResult<()> {
    core::set_validate_inputs(&template, validate).map_err(into_php_err)
}

/// Remove the world from the cache.
///
/// The template will have to be registered again before it can be compiled again.
#[php_function]
#[php(name = "OicanaInternal\\remove_world")]
pub fn remove_world(template_id: String) -> PhpResult<()> {
    core::remove_world(&template_id);
    Ok(())
}

fn into_core_blobs(
    blobs: HashMap<String, &BlobWithMetadata>,
) -> HashMap<String, core::BlobWithMetadata> {
    blobs
        .into_iter()
        .map(|(key, value)| {
            (
                key,
                core::BlobWithMetadata {
                    bytes: value.bytes.clone(),
                    meta: value.meta.clone(),
                },
            )
        })
        .collect()
}

fn into_php_err(error: core::FfiError) -> PhpException {
    PhpException::default(error.to_string())
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
        .function(wrap_function!(get_warnings))
        .function(wrap_function!(remove_world))
        .function(wrap_function!(set_validate_inputs))
        .class::<BlobWithMetadata>()
}

fn startup_function(_ty: i32, _mod_num: i32) -> i32 {
    0
}
