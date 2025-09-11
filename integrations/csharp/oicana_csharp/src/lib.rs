//! This crate defines FFI bindings for PDF templating from C#

use dashmap::DashMap;
use interoptopus::patterns::slice::FFISlice;
use interoptopus::patterns::string::AsciiPointer;
use interoptopus::{ffi_function, ffi_type, function, Inventory, InventoryBuilder};
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
use oicana_world::{CompiledDocument, TemplateCompilationFailure};
use once_cell::sync::Lazy;
use serde_json::Error;
use std::io::Cursor;
use std::slice;
use std::sync::{Arc, Mutex};
use typst::foundations::Bytes;
use typst::syntax::{FileId, VirtualPath};

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
    let template = template.as_str().unwrap().to_owned();

    match unsafe { prepare_world(files, json_inputs, blob_inputs, compilation_options.mode) } {
        Ok(world) => {
            WORLD_CACHE.insert(template.clone(), world);
            let world = WORLD_CACHE.get_mut(&template);
            let mut world = world.unwrap();
            let document_result = world.value_mut().compile();
            Buffer::from_document_result(document_result, &world, compilation_options.target)
        }
        Err(error) => error,
    }
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
pub unsafe extern "C" fn unsafe_compile_template_once(
    files: Buffer,
    json_inputs: FFISlice<FfiJsonInput>,
    blob_inputs: FFISlice<FfiBlobInput>,
    compilation_options: CompilationOptions,
) -> Buffer {
    match unsafe { prepare_world(files, json_inputs, blob_inputs, compilation_options.mode) } {
        Ok(mut world) => {
            let document_result = world.compile();
            Buffer::from_document_result(document_result, &world, compilation_options.target)
        }
        Err(error) => error,
    }
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
    let template = template.as_str().unwrap().to_owned();
    let world = WORLD_CACHE.get_mut(&template);
    let inputs = unsafe { prepare_inputs(json_inputs, blob_inputs, compilation_options.mode) };
    let inputs = match inputs {
        Err(error) => {
            return Buffer::from_error(format!("The inputs could not be prepared: {error:?}"))
        }
        Ok(inputs) => inputs,
    };

    let Some(mut world) = world else {
        return Buffer::from_error(format!("The template '{template}' is not registered"));
    };

    let world = world.value_mut();
    world.update_inputs(inputs);
    let document_result = world.compile();
    Buffer::from_document_result(document_result, world, compilation_options.target)
}

/// Load the inputs of the given template.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn inputs(template: AsciiPointer) -> Buffer {
    let template = template.as_str().unwrap();
    let world = WORLD_CACHE.get_mut(template);
    let Some(world) = world else {
        return Buffer::from_error(format!("The template '{template}' is not registered"));
    };

    let manifest = &world.manifest().tool.oicana;
    let inputs = match serde_json::ser::to_string(manifest).map_err(|error| format!("{error:?}")) {
        Ok(inputs) => inputs,
        Err(error) => return Buffer::from_error(error),
    };

    Buffer::from_ok(inputs.into_bytes())
}

/// Load the source at the given path in the template.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn get_source(template: AsciiPointer, path: AsciiPointer) -> Buffer {
    let template = template.as_str().unwrap();
    let world = WORLD_CACHE.get_mut(template);
    let Some(world) = world else {
        return Buffer::from_error(format!("The template '{template}' is not registered"));
    };

    let path = path.as_str().unwrap();
    let source = match world
        .files
        .source(FileId::new(None, VirtualPath::new(path)))
    {
        Ok(source) => source,
        Err(error) => return Buffer::from_error(error.to_string()),
    };

    Buffer::from_ok(source.text().to_owned().into_bytes())
}

/// Load the file at the given path in the template.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn get_file(template: AsciiPointer, path: AsciiPointer) -> Buffer {
    let template = template.as_str().unwrap();
    let world = WORLD_CACHE.get_mut(template);
    let Some(world) = world else {
        return Buffer::from_error(format!("The template '{template}' is not registered"));
    };

    let path = path.as_str().unwrap();
    let file = match world.files.file(FileId::new(None, VirtualPath::new(path))) {
        Ok(file) => file,
        Err(error) => return Buffer::from_error(error.to_string()),
    };

    Buffer::from_ok(file.to_vec())
}

/// Clear the specified template from the internal cache.
///
/// This method requires a previous successful call to [`unsafe_register_template`].
/// Check if the returned buffer is an error before interpreting the content.
#[ffi_function]
#[no_mangle]
pub extern "C" fn unregister_template(template: AsciiPointer) {
    WORLD_CACHE.remove(template.as_str().unwrap());
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
        let _boxed_data =
            Box::from_raw(slice::from_raw_parts_mut(buffer.data, buffer.len as usize));
    }
}

/// Configure Oicana.
#[ffi_function]
#[no_mangle]
pub extern "C" fn configure(config: Config) -> Buffer {
    for mut world in WORLD_CACHE.iter_mut() {
        world.color = config.color.into();
    }

    match CONFIGURATION.lock() {
        Ok(mut cfg) => {
            cfg.color = config.color;
            Buffer::from_ok(Vec::new())
        }
        Err(error) => Buffer::from_error(error.to_string()),
    }
}

unsafe fn prepare_world(
    files: Buffer,
    json_inputs: FFISlice<FfiJsonInput>,
    blob_inputs: FFISlice<FfiBlobInput>,
    compilation_mode: CompilationMode,
) -> Result<OicanaWorld<PackedTemplate>, Buffer> {
    let files = unsafe {
        PackedTemplate::new(Cursor::new(slice::from_raw_parts::<u8>(
            files.data,
            files.len as usize,
        )))
    };
    let manifest = match files.manifest() {
        Ok(manifest) => manifest,
        Err(error) => return Err(Buffer::from_error(format!("{error}"))),
    };
    let inputs = unsafe { prepare_inputs(json_inputs, blob_inputs, compilation_mode) };
    let inputs = match inputs {
        Err(error) => {
            return Err(Buffer::from_error(format!(
                "The inputs could not be prepared: {error:?}"
            )))
        }
        Ok(inputs) => inputs,
    };

    let color = get_diagnostic_color();

    OicanaWorld::new(files, inputs, manifest)
        .map_err(|error| Buffer::from_error(format!("{error}")))
        .map(|mut world| {
            world.color = color;
            world
        })
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
        let mut buf = error_string.into_bytes().into_boxed_slice();
        let len = buf.len() as u32;
        let data = buf.as_mut_ptr();
        std::mem::forget(buf);

        Buffer {
            data,
            len,
            error: true,
        }
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

    fn from_document_result(
        document_result: Result<CompiledDocument, TemplateCompilationFailure>,
        world: &OicanaWorld<PackedTemplate>,
        format: CompilationTarget,
    ) -> Self {
        match document_result {
            Ok(compilation_result) => match format {
                CompilationTarget::Pdf => {
                    match export_merged_pdf(&compilation_result.document, world) {
                        Err(error) => Buffer::from_error(format!(
                            "Error encoding compilation result as PDF: {error:?}"
                        )),
                        Ok(pdf) => Buffer::from_ok(pdf),
                    }
                }
                CompilationTarget::Png => {
                    match export_merged_png(&compilation_result.document, 1.0) {
                        Err(error) => Buffer::from_error(format!(
                            "Error encoding compilation result as PNG: {error:?}"
                        )),
                        Ok(png) => Buffer::from_ok(png),
                    }
                }
                CompilationTarget::Svg => {
                    Buffer::from_ok(export_merged_svg(&compilation_result.document))
                }
            },
            Err(error) => Buffer::from_error(format!("{error:?}")),
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
    /// The exported standard is PDF/A-3b
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

/// Options for compiling the template
#[ffi_type]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CompilationOptions {
    /// Formats that an Oicana template can be compiled into.
    pub target: CompilationTarget,
    /// The mode of compilation
    pub mode: CompilationMode,
    /// Pixels per pt
    /// Only used for PNG export
    pub px_per_pt: f32,
}

unsafe fn prepare_inputs(
    json_inputs: FFISlice<FfiJsonInput>,
    blob_inputs: FFISlice<FfiBlobInput>,
    compilation_mode: CompilationMode,
) -> Result<TemplateInputs, Error> {
    let mut inputs = TemplateInputs::new();
    for blob_input in blob_inputs.iter() {
        let mut blob = Blob::from(Bytes::new(unsafe {
            slice::from_raw_parts::<u8>(blob_input.data.data, blob_input.data.len as usize)
        }));
        blob.metadata = serde_json::from_str(blob_input.meta.as_str().unwrap())?;
        inputs.with_input(BlobInput {
            key: blob_input.key.as_str().unwrap().into(),
            value: blob,
        });
    }
    for json_input in json_inputs.iter() {
        inputs.with_input(JsonInput {
            key: json_input.key.as_str().unwrap().into(),
            value: json_input.data.as_str().unwrap().to_owned(),
        });
    }

    match compilation_mode {
        CompilationMode::Development => inputs.with_config(CompilationConfig::development()),
        CompilationMode::Production => inputs.with_config(CompilationConfig::production()),
    };

    Ok(inputs)
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

impl From<DiagnosticColor> for oicana_world::diagnostics::DiagnosticColor {
    fn from(value: DiagnosticColor) -> Self {
        match value {
            DiagnosticColor::Ansi => oicana_world::diagnostics::DiagnosticColor::Ansi,
            DiagnosticColor::None => oicana_world::diagnostics::DiagnosticColor::None,
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

/// Get the currently configured diagnostic color.
fn get_diagnostic_color() -> oicana_world::diagnostics::DiagnosticColor {
    match CONFIGURATION.lock() {
        Ok(config) => config.color.into(),
        Err(_) => oicana_world::diagnostics::DiagnosticColor::None,
    }
}

static CONFIGURATION: Lazy<Arc<Mutex<Config>>> = Lazy::new(|| {
    Arc::new(Mutex::new(Config {
        color: DiagnosticColor::None,
    }))
});

static WORLD_CACHE: Lazy<DashMap<String, OicanaWorld<PackedTemplate>>> = Lazy::new(DashMap::new);

/// List methods for auto generated bindings
pub fn my_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(function!(unsafe_compile_template))
        .register(function!(unsafe_compile_template_once))
        .register(function!(unsafe_register_template))
        .register(function!(inputs))
        .register(function!(get_source))
        .register(function!(get_file))
        .register(function!(unsafe_free_buffer))
        .register(function!(unregister_template))
        .register(function!(configure))
        .inventory()
}
