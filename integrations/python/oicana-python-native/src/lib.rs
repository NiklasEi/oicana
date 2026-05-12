//! The Python integration of Oicana.
//!
//! You will want to use this through the Python package `oicana`.

use std::collections::HashMap;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use oicana_ffi_core as core;

/// Compilation mode enum
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilationMode {
    Production,
    Development,
}

impl From<CompilationMode> for core::CompilationMode {
    fn from(mode: CompilationMode) -> Self {
        match mode {
            CompilationMode::Production => core::CompilationMode::Production,
            CompilationMode::Development => core::CompilationMode::Development,
        }
    }
}

/// Blob input with metadata
#[pyclass]
pub struct BlobWithMetadata {
    #[pyo3(get)]
    pub bytes: Py<PyBytes>,
    #[pyo3(get)]
    pub meta: String, // JSON string
}

#[pymethods]
impl BlobWithMetadata {
    #[new]
    fn new(bytes: Py<PyBytes>, meta: String) -> Self {
        Self { bytes, meta }
    }
}

/// Configure automatic cache eviction after each compilation.
///
/// # Parameters
///
/// `max_age` (start value: 10) - Maximum age threshold, or null to disable:
///   - `null` - Disables cache eviction (cache never cleared)
///   - `0` - Clears all cache entries with every eviction
///   - `1` - Keeps only entries used since the last eviction
///   - `n` - Keeps entries used within the last n evictions
#[pyfunction]
#[pyo3(signature = (max_age=None))]
fn configure_automatic_cache_eviction(max_age: Option<usize>) {
    core::configure_automatic_cache_eviction(max_age);
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
#[pyfunction]
fn evict_cache(max_age: usize) {
    core::evict_cache(max_age);
}

/// Register the given template. This will read the template files as a PackedTemplate and
/// compile it once with the given inputs. The Typst World will be cached and reused for
/// subsequent calls to the other methods with the same template identifier.
#[pyfunction]
fn register_template(
    py: Python<'_>,
    template: String,
    files: &Bound<'_, PyBytes>,
    json_inputs: HashMap<String, String>,
    blob_inputs: &Bound<'_, PyDict>,
    compilation_mode: CompilationMode,
) -> PyResult<String> {
    let blobs = into_core_blobs(py, blob_inputs)?;
    core::register_template(
        &template,
        files.as_bytes(),
        json_inputs,
        blobs,
        compilation_mode.into(),
    )
    .map_err(into_py_err)
}

/// Compile the identified template with the given inputs.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn compile_template(
    py: Python<'_>,
    template: String,
    json_inputs: HashMap<String, String>,
    blob_inputs: &Bound<'_, PyDict>,
    compilation_mode: CompilationMode,
) -> PyResult<String> {
    let blobs = into_core_blobs(py, blob_inputs)?;
    core::compile_template(&template, json_inputs, blobs, compilation_mode.into())
        .map_err(into_py_err)
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn inputs(template: String) -> PyResult<String> {
    core::inputs(&template).map_err(into_py_err)
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn get_source(template: String, file: String) -> PyResult<String> {
    core::get_source(&template, &file).map_err(into_py_err)
}

/// Load the binary file content from the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn get_file(py: Python<'_>, template: String, file: String) -> PyResult<Bound<'_, PyBytes>> {
    let bytes = core::get_file(&template, &file).map_err(into_py_err)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Export the given document
///
/// Make sure to call `remove_document` with the document_id afterwards, to free the memory.
#[pyfunction]
fn export_document(
    py: Python<'_>,
    document_id: String,
    export_format: String,
) -> PyResult<Bound<'_, PyBytes>> {
    let format = core::parse_export_format(&export_format).map_err(into_py_err)?;
    let bytes = core::export_document(&document_id, format).map_err(into_py_err)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Remove the document from the cache.
#[pyfunction]
fn remove_document(document_id: String) -> PyResult<()> {
    core::remove_document(&document_id);
    Ok(())
}

/// Enable or disable JSON schema validation for the given template.
///
/// When enabled (the default), JSON inputs are validated against their schemas
/// before compilation.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn set_validate_inputs(template: String, validate: bool) -> PyResult<()> {
    core::set_validate_inputs(&template, validate).map_err(into_py_err)
}

/// Remove the world from the cache.
///
/// The template will have to be registered again before it can be compiled again.
#[pyfunction]
fn remove_world(template_id: String) -> PyResult<()> {
    core::remove_world(&template_id);
    Ok(())
}

fn into_core_blobs(
    py: Python<'_>,
    blob_inputs: &Bound<'_, PyDict>,
) -> PyResult<HashMap<String, core::BlobWithMetadata>> {
    let mut blobs = HashMap::with_capacity(blob_inputs.len());
    for (key, value) in blob_inputs.iter() {
        let key: String = key.extract()?;
        let blob: Py<BlobWithMetadata> = value.extract()?;
        let blob = blob.bind(py).borrow();
        let bytes = blob.bytes.bind(py).as_bytes().to_vec();
        blobs.insert(
            key,
            core::BlobWithMetadata {
                bytes,
                meta: blob.meta.clone(),
            },
        );
    }
    Ok(blobs)
}

fn into_py_err(error: core::FfiError) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

#[pymodule]
fn oicana_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(register_template, m)?)?;
    m.add_function(wrap_pyfunction!(compile_template, m)?)?;
    m.add_function(wrap_pyfunction!(export_document, m)?)?;
    m.add_function(wrap_pyfunction!(inputs, m)?)?;
    m.add_function(wrap_pyfunction!(get_source, m)?)?;
    m.add_function(wrap_pyfunction!(get_file, m)?)?;
    m.add_function(wrap_pyfunction!(remove_document, m)?)?;
    m.add_function(wrap_pyfunction!(remove_world, m)?)?;
    m.add_function(wrap_pyfunction!(set_validate_inputs, m)?)?;
    m.add_function(wrap_pyfunction!(configure_automatic_cache_eviction, m)?)?;
    m.add_function(wrap_pyfunction!(evict_cache, m)?)?;
    m.add_class::<CompilationMode>()?;
    m.add_class::<BlobWithMetadata>()?;
    Ok(())
}
