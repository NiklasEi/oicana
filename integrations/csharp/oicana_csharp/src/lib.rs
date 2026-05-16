//! This crate defines FFI bindings for PDF templating from C#

use std::collections::HashMap;
use std::slice;

use interoptopus::patterns::slice::FFISlice;
use interoptopus::patterns::string::AsciiPointer;
use interoptopus::{ffi_function, ffi_type, function, Inventory, InventoryBuilder};

use oicana_ffi_core as core;

/// Configure automatic cache eviction after each compilation.
///
/// # Parameters
///
/// `max_age` (start value: 10) - Maximum age threshold, or null to disable:
///   - `null` - Disables cache eviction (cache never cleared)
///   - `0` - Clears all cache entries with every eviction
///   - `1` - Keeps only entries used since the last eviction
///   - `n` - Keeps entries used within the last n evictions
#[ffi_function]
#[no_mangle]
pub extern "C" fn configure_automatic_cache_eviction(max_age: i64) {
    let max_age = if max_age < 0 {
        None
    } else {
        Some(max_age as usize)
    };
    core::configure_automatic_cache_eviction(max_age);
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
///
/// # Parameters
///
/// * `max_age` - Maximum age threshold for eviction
///
/// Calls with negative `max_age` are ignored.
#[ffi_function]
#[no_mangle]
pub extern "C" fn evict_cache(max_age: i64) {
    if max_age >= 0 {
        core::evict_cache(max_age as usize);
    }
}

/// Register a template for the given identifier
///
/// After a successful call to this method, use [`unsafe_compile_template()`] for compiling
/// with improved performance.
///
/// # Safety
///
/// The caller is responsible for ensuring that the provided
/// `template`, `input`, and `banner` pointers are valid and non-null, and that
/// the `input` and `banner` data is properly aligned and initialized.
///
/// Additionally, the caller must ensure that no inputs are modified
/// concurrently while this function is executing.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn unsafe_register_template(
    template: AsciiPointer,
    files: Buffer,
    json_inputs: FFISlice<FfiJsonInput>,
    blob_inputs: FFISlice<FfiBlobInput>,
    compilation_options: CompilationOptions,
) -> Buffer {
    let template = match template.as_str() {
        Ok(template) => template,
        Err(error) => return Buffer::from_error(format!("Invalid template ID: {error:?}")),
    };
    let files = unsafe { slice_from_buffer(files) };
    let (json_map, blob_map) = match unsafe { parse_inputs(json_inputs, blob_inputs) } {
        Ok(parsed) => parsed,
        Err(error) => return error,
    };

    Buffer::from_string_result(core::register_template(
        template,
        files,
        json_map,
        blob_map,
        compilation_options.mode.into(),
    ))
}

/// Compile the given template once.
///
/// This method does not do any caching. If you want faster compilations,
/// prepare your templates by registering them with [`unsafe_register_template`]
/// and then calling [`unsafe_compile_template`] with the same identifier.
///
/// # Safety
///
/// The caller is responsible for ensuring that the provided
/// `files`, `json_inputs`, and `blob_inputs` pointers are valid and non-null,
/// and that all data is properly aligned and initialized.
///
/// Additionally, the caller must ensure that no inputs are modified
/// concurrently while this function is executing.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn unsafe_export_template_once(
    files: Buffer,
    json_inputs: FFISlice<FfiJsonInput>,
    blob_inputs: FFISlice<FfiBlobInput>,
    compile_options: CompilationOptions,
    export_options: ExportOptions,
) -> Buffer {
    let files = unsafe { slice_from_buffer(files) };
    let (json_map, blob_map) = match unsafe { parse_inputs(json_inputs, blob_inputs) } {
        Ok(parsed) => parsed,
        Err(error) => return error,
    };

    Buffer::from_bytes_result(core::compile_once(
        files,
        json_map,
        blob_map,
        compile_options.mode.into(),
        export_options.into(),
    ))
}

/// Compile the template with the given identifier
///
/// This method requires a previous successful call to [`unsafe_register_template()`].
/// Check if the returned buffer is an error before interpreting the content.
///
/// # Safety
///
/// The caller is responsible for ensuring that the provided
/// `template`, `input`, and `banner` pointers are valid and non-null, and that
/// the `input` and `banner` data is properly aligned and initialized.
///
/// Additionally, the caller must ensure that the blob input buffers are not modified
/// concurrently while this function is executing.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn unsafe_compile_template(
    template: AsciiPointer,
    json_inputs: FFISlice<FfiJsonInput>,
    blob_inputs: FFISlice<FfiBlobInput>,
    compilation_options: CompilationOptions,
) -> Buffer {
    let template = match template.as_str() {
        Ok(template) => template,
        Err(error) => return Buffer::from_error(format!("Invalid template ID: {error:?}")),
    };
    let (json_map, blob_map) = match unsafe { parse_inputs(json_inputs, blob_inputs) } {
        Ok(parsed) => parsed,
        Err(error) => return error,
    };

    Buffer::from_string_result(core::compile_template(
        template,
        json_map,
        blob_map,
        compilation_options.mode.into(),
    ))
}

/// Export the given document
///
/// # Safety
///
/// The caller is responsible for ensuring that the provided
/// `document_id` pointer is valid and non-null
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn unsafe_export_document(
    document_id: AsciiPointer,
    export_options: ExportOptions,
) -> Buffer {
    let document_id = match document_id.as_str() {
        Ok(document_id) => document_id,
        Err(error) => return Buffer::from_error(format!("Invalid document ID: {error:?}")),
    };
    Buffer::from_bytes_result(core::export_document(document_id, export_options.into()))
}

/// Load the inputs of the given template.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn inputs(template: AsciiPointer) -> Buffer {
    let template = match template.as_str() {
        Ok(template) => template,
        Err(error) => return Buffer::from_error(format!("{error:?}")),
    };
    Buffer::from_string_result(core::inputs(template))
}

/// Load the source at the given path in the template.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn get_source(template: AsciiPointer, path: AsciiPointer) -> Buffer {
    let template = match template.as_str() {
        Ok(template) => template,
        Err(error) => return Buffer::from_error(format!("Invalid template ID: {error:?}")),
    };
    let path = match path.as_str() {
        Ok(path) => path,
        Err(error) => return Buffer::from_error(format!("Invalid path: {error:?}")),
    };
    Buffer::from_string_result(core::get_source(template, path))
}

/// Load the file at the given path in the template.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn get_file(template: AsciiPointer, path: AsciiPointer) -> Buffer {
    let template = match template.as_str() {
        Ok(template) => template,
        Err(error) => return Buffer::from_error(format!("Invalid template ID: {error:?}")),
    };
    let path = match path.as_str() {
        Ok(path) => path,
        Err(error) => return Buffer::from_error(format!("Invalid path: {error:?}")),
    };
    Buffer::from_bytes_result(core::get_file(template, path))
}

/// Frees a buffer allocated by `compile_template`.
///
/// # Safety
///
/// This function is unsafe because it assumes the following:
///
/// 1. The [`Buffer::data`] pointer must be non-null and valid. It must point to memory allocated by
///    Rust which was not previously freed.
///
/// 2. No other pointers to the memory should be used after this function has been called.
///
/// 3. This function must be called from a context where it is safe to free memory, ensuring
///    no concurrent accesses.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn unsafe_free_buffer(buffer: Buffer) {
    assert!(
        !buffer.data.is_null(),
        "Buffer::data is null while trying to free the memory"
    );
    unsafe {
        let _boxed_data = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            buffer.data,
            buffer.len as usize,
        ));
    }
}

/// Enable or disable JSON schema validation for the given template.
///
/// When enabled (the default), JSON inputs are validated against their schemas
/// before compilation.
#[ffi_function]
#[no_mangle]
pub extern "C" fn set_validate_inputs(template: AsciiPointer, validate: bool) -> Buffer {
    let template = match template.as_str() {
        Ok(template) => template,
        Err(error) => return Buffer::from_error(format!("{error:?}")),
    };
    Buffer::from_unit_result(core::set_validate_inputs(template, validate))
}

/// Configure Oicana.
#[ffi_function]
#[no_mangle]
pub extern "C" fn configure(config: Config) -> Buffer {
    core::configure_diagnostic_color(config.color.into());
    Buffer::from_ok(Vec::new())
}

/// Remove the document from the cache.
#[ffi_function]
#[no_mangle]
pub extern "C" fn remove_document(document_id: AsciiPointer) -> Buffer {
    let document_id = match document_id.as_str() {
        Ok(document_id) => document_id,
        Err(error) => return Buffer::from_error(format!("{error:?}")),
    };
    core::remove_document(document_id);
    Buffer::from_ok(Vec::new())
}

/// Return any compilation warnings produced for the given document.
///
/// On success the buffer contains either the warnings text (UTF-8) or is
/// empty when there were no warnings. Warnings are cleared together with
/// the document by [`remove_document`].
#[ffi_function]
#[no_mangle]
pub extern "C" fn get_warnings(document_id: AsciiPointer) -> Buffer {
    let document_id = match document_id.as_str() {
        Ok(document_id) => document_id,
        Err(error) => return Buffer::from_error(format!("{error:?}")),
    };
    match core::get_warnings(document_id) {
        Some(warnings) => Buffer::from_ok_string(warnings),
        None => Buffer::from_ok(Vec::new()),
    }
}

/// Clear the specified template from the internal cache.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn remove_world(template_id: AsciiPointer) -> Buffer {
    let template_id = match template_id.as_str() {
        Ok(template_id) => template_id,
        Err(error) => return Buffer::from_error(format!("{error:?}")),
    };
    core::remove_world(template_id);
    Buffer::from_ok(Vec::new())
}

/// Access to a piece of Rust memory.
///
/// If [`Self::error`] is `true`, [`Self::data`] will point to a UTF-8 encoded error message.
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Buffer {
    /// Pointer to the beginning of the buffer data.
    pub data: *mut u8,
    /// Whether this buffer is an error.
    pub error: bool,
    /// Length of the buffer data.
    pub len: u32,
}

impl Buffer {
    fn from_error(error_string: String) -> Self {
        Buffer {
            error: true,
            ..Buffer::from_ok_string(error_string)
        }
    }

    fn from_ok_string(string: String) -> Self {
        Buffer::from_ok(string.into_bytes())
    }

    fn from_ok(value: Vec<u8>) -> Self {
        let mut buf = value.into_boxed_slice();
        let len = buf.len() as u32;
        let data = buf.as_mut_ptr();
        std::mem::forget(buf);

        Buffer {
            data,
            len,
            error: false,
        }
    }

    fn from_bytes_result(result: Result<Vec<u8>, core::FfiError>) -> Self {
        match result {
            Ok(bytes) => Buffer::from_ok(bytes),
            Err(error) => Buffer::from_error(error.to_string()),
        }
    }

    fn from_string_result(result: Result<String, core::FfiError>) -> Self {
        match result {
            Ok(string) => Buffer::from_ok_string(string),
            Err(error) => Buffer::from_error(error.to_string()),
        }
    }

    fn from_unit_result(result: Result<(), core::FfiError>) -> Self {
        match result {
            Ok(()) => Buffer::from_ok(Vec::new()),
            Err(error) => Buffer::from_error(error.to_string()),
        }
    }
}

/// A collection of string pairs representing JSON inputs
#[ffi_type]
#[repr(C)]
#[derive(Debug)]
pub struct FfiJsonInput<'a> {
    /// String containing the json payload of this input.
    pub data: AsciiPointer<'a>,
    /// Identifier of the input definition this input value belongs to.
    pub key: AsciiPointer<'a>,
}

/// A collection of string keys with Buffers representing blob inputs
#[ffi_type]
#[repr(C)]
#[derive(Debug)]
pub struct FfiBlobInput<'a> {
    /// Buffer containing the main data of the blob input.
    pub data: Buffer,
    /// Identifier of the input definition this input value belongs to.
    pub key: AsciiPointer<'a>,
    /// Metadata of the blob input as json.
    pub meta: AsciiPointer<'a>,
}

/// Formats that an Oicana template can be compiled into.
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum CompilationTarget {
    /// Render the template to a PDF file.
    ///
    /// The exported standard can be configured in the template manifest
    /// via [tool.oicana.export.pdf] section. Defaults to PDF/A-3b.
    Pdf,
    /// Render the template into a png image.
    ///
    /// The image is not optimized for file size to speed up compilation.
    Png,
    /// Render the template as SVG file.
    Svg,
}

/// The mode of compilation
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompilationMode {
    /// Use development values for inputs if an input is not explicitly set.
    /// If there is no development value, fall back to the default value.
    Development,
    /// If an input is not set, use the default value if available.
    Production,
}

impl From<CompilationMode> for core::CompilationMode {
    fn from(mode: CompilationMode) -> Self {
        match mode {
            CompilationMode::Development => core::CompilationMode::Development,
            CompilationMode::Production => core::CompilationMode::Production,
        }
    }
}

/// Options for compiling the template
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CompilationOptions {
    /// The mode of compilation
    pub mode: CompilationMode,
}

/// Options for exporting the template
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ExportOptions {
    /// Formats that an Oicana template can be compiled into.
    pub target: CompilationTarget,
    /// Pixels per pt
    /// Only used for PNG export
    pub px_per_pt: f32,
}

impl From<ExportOptions> for core::ExportFormat {
    fn from(opts: ExportOptions) -> Self {
        match opts.target {
            CompilationTarget::Pdf => core::ExportFormat::Pdf,
            CompilationTarget::Png => core::ExportFormat::Png {
                pixels_per_pt: opts.px_per_pt,
            },
            CompilationTarget::Svg => core::ExportFormat::Svg,
        }
    }
}

/// Formats that the compiled documents can be rendered into.
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum DiagnosticColor {
    /// No colors in diagnostic output
    None,
    /// ANSI codes for colors in diagnostic output
    Ansi,
}

impl From<DiagnosticColor> for core::DiagnosticColor {
    fn from(value: DiagnosticColor) -> Self {
        match value {
            DiagnosticColor::Ansi => core::DiagnosticColor::Ansi,
            DiagnosticColor::None => core::DiagnosticColor::None,
        }
    }
}

/// Oicana Configuration.
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Config {
    /// Coloring for diagnostics like warnings and errors
    pub color: DiagnosticColor,
}

unsafe fn slice_from_buffer<'a>(buffer: Buffer) -> &'a [u8] {
    unsafe { slice::from_raw_parts::<u8>(buffer.data, buffer.len as usize) }
}

type ParsedInputs = (
    HashMap<String, String>,
    HashMap<String, core::BlobWithMetadata>,
);

unsafe fn parse_inputs(
    json_inputs: FFISlice<FfiJsonInput>,
    blob_inputs: FFISlice<FfiBlobInput>,
) -> Result<ParsedInputs, Buffer> {
    let mut json_map = HashMap::new();
    for input in json_inputs.iter() {
        let key = input
            .key
            .as_str()
            .map_err(|error| Buffer::from_error(format!("Invalid JSON input key: {error:?}")))?;
        let data = input
            .data
            .as_str()
            .map_err(|error| Buffer::from_error(format!("Invalid JSON input data: {error:?}")))?;
        json_map.insert(key.to_owned(), data.to_owned());
    }

    let mut blob_map = HashMap::new();
    for input in blob_inputs.iter() {
        let key = input
            .key
            .as_str()
            .map_err(|error| Buffer::from_error(format!("Invalid blob input key: {error:?}")))?;
        let meta = input
            .meta
            .as_str()
            .map_err(|error| Buffer::from_error(format!("Invalid blob input meta: {error:?}")))?;
        let bytes = unsafe { slice_from_buffer(input.data) }.to_vec();
        blob_map.insert(
            key.to_owned(),
            core::BlobWithMetadata {
                bytes,
                meta: meta.to_owned(),
            },
        );
    }

    Ok((json_map, blob_map))
}

/// List methods for auto generated bindings
pub fn my_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(function!(unsafe_compile_template))
        .register(function!(unsafe_export_template_once))
        .register(function!(unsafe_register_template))
        .register(function!(unsafe_export_document))
        .register(function!(inputs))
        .register(function!(get_source))
        .register(function!(get_file))
        .register(function!(unsafe_free_buffer))
        .register(function!(remove_world))
        .register(function!(remove_document))
        .register(function!(get_warnings))
        .register(function!(configure))
        .register(function!(set_validate_inputs))
        .register(function!(configure_automatic_cache_eviction))
        .register(function!(evict_cache))
        .inventory()
}
