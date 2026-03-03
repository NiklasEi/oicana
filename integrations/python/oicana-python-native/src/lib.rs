//! The Python integration of Oicana.
//!
//! You will want to use this through the Python package `oicana`.

use dashmap::DashMap;
use once_cell::sync::Lazy;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use oicana_export::pdf::export_merged_pdf;
use oicana_export::png::export_merged_png;
use oicana_export::svg::export_merged_svg;
use oicana_files::packed::PackedTemplate;
use oicana_files::TemplateFiles;
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
/// `max_age` (start value: 10) - Maximum age threshold, or null to disable:
///   - `null` - Disables cache eviction (cache never cleared)
///   - `0` - Clears all cache entries with every eviction
///   - `1` - Keeps only entries used since the last eviction
///   - `n` - Keeps entries used within the last n evictions
#[pyfunction]
#[pyo3(signature = (max_age=None))]
fn configure_automatic_cache_eviction(max_age: Option<usize>) {
    CACHE_EVICTION_AGE.store(max_age.unwrap_or(usize::MAX), Ordering::Relaxed);
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
#[pyfunction]
fn evict_cache(max_age: usize) {
    oicana_world::evict_cache(max_age);
}

/// Compilation mode enum
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilationMode {
    Production,
    Development,
}

impl From<CompilationMode> for CompilationConfig {
    fn from(mode: CompilationMode) -> Self {
        match mode {
            CompilationMode::Production => CompilationConfig::production(),
            CompilationMode::Development => CompilationConfig::development(),
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
    let files_data = files.as_bytes();
    let packed = PackedTemplate::new(Cursor::new(files_data))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let manifest = packed
        .manifest()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let mut inputs = prepare_inputs(py, json_inputs, blob_inputs)?;
    inputs.with_config(compilation_mode.into());

    let mut zip_world = OicanaWorld::new(packed, inputs, manifest)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    zip_world.color = DiagnosticColor::None;

    let document = zip_world
        .compile()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

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
#[pyfunction]
fn compile_template(
    py: Python<'_>,
    template: String,
    json_inputs: HashMap<String, String>,
    blob_inputs: &Bound<'_, PyDict>,
    compilation_mode: CompilationMode,
) -> PyResult<String> {
    let Some(mut world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PyRuntimeError::new_err("Template was not registered"));
    };

    let mut inputs = prepare_inputs(py, json_inputs, blob_inputs)?;
    inputs.with_config(compilation_mode.into());
    world.update_inputs(inputs);

    let document = world
        .compile()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

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

fn template_id_from_document_id(document_id: &str) -> &str {
    &document_id[37..]
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn inputs(template: String) -> PyResult<String> {
    let Some(world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PyRuntimeError::new_err("Template was not registered"));
    };
    let oicana_config = &world.manifest().tool.oicana;

    serde_json::ser::to_string(&oicana_config).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn get_source(template: String, file: String) -> PyResult<String> {
    let Some(world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PyRuntimeError::new_err("Template was not registered"));
    };
    world
        .files
        .source(FileId::new(None, VirtualPath::new(file)))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        .map(|source| source.text().to_string())
}

/// Load the binary file content from the template.
///
/// Calling this method requires a previous call to `register_template` with the same template
/// identifier.
#[pyfunction]
fn get_file(py: Python<'_>, template: String, file: String) -> PyResult<Bound<'_, PyBytes>> {
    let Some(world) = WORLD_CACHE.get_mut(&template) else {
        return Err(PyRuntimeError::new_err("Template was not registered"));
    };
    let bytes = world
        .files
        .file(FileId::new(None, VirtualPath::new(file)))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

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
    let Some(document) = DOCUMENT_CACHE.get(&document_id) else {
        return Err(PyRuntimeError::new_err("Document not found!"));
    };

    let export_format: ExportFormat =
        serde_json::from_str(&export_format).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let bytes = match export_format {
        ExportFormat::Png { pixels_per_pt } => export_merged_png(&document, pixels_per_pt)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to encode PNG: {e:?}")))?,
        ExportFormat::Pdf => {
            let template_id = template_id_from_document_id(&document_id);
            let Some(world) = WORLD_CACHE.get(template_id) else {
                return Err(PyRuntimeError::new_err(format!(
                    "World '{template_id}' for the given document '{document_id}' not found!"
                )));
            };

            export_merged_pdf(
                &document,
                &*world,
                &world.manifest().tool.oicana.export.pdf.standards,
            )
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to encode PDF: {e:?}")))?
        }
        ExportFormat::Svg => export_merged_svg(&document),
    };

    Ok(PyBytes::new(py, &bytes))
}

/// Remove the document from the cache.
#[pyfunction]
fn remove_document(document_id: String) -> PyResult<()> {
    DOCUMENT_CACHE.remove(&document_id);
    Ok(())
}

/// Remove the world from the cache.
///
/// The template will have to be registered again before it can be compiled again.
#[pyfunction]
fn remove_world(template_id: String) -> PyResult<()> {
    WORLD_CACHE.remove(&template_id);
    Ok(())
}

fn prepare_inputs(
    py: Python<'_>,
    json_inputs: HashMap<String, String>,
    blob_inputs: &Bound<'_, PyDict>,
) -> PyResult<TemplateInputs> {
    let mut inputs = TemplateInputs::new();

    for (key, value) in json_inputs {
        inputs.with_input(JsonInput::new(key, value));
    }

    for (key, value) in blob_inputs.iter() {
        let key_str: String = key.extract()?;
        let blob_with_meta: Py<BlobWithMetadata> = value.extract()?;
        let blob_ref = blob_with_meta.bind(py).borrow();

        let bytes_ref = blob_ref.bytes.bind(py);
        let bytes_vec = bytes_ref.as_bytes().to_vec();
        let mut blob = Blob::from(Bytes::new(bytes_vec));

        blob.metadata =
            Deserialize::deserialize(serde_json::Value::from_str(&blob_ref.meta).map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to parse metadata JSON: {e:?}"))
            })?)
            .map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to deserialize metadata: {e:?}"))
            })?;

        inputs.with_input(BlobInput::new(key_str, blob));
    }

    Ok(inputs)
}

#[derive(Deserialize)]
#[serde(tag = "format")]
enum ExportFormat {
    #[serde(alias = "png")]
    Png {
        #[serde(rename = "pixelsPerPt")]
        pixels_per_pt: f32,
    },
    #[serde(alias = "pdf")]
    Pdf,
    #[serde(alias = "svg")]
    Svg,
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
    m.add_function(wrap_pyfunction!(configure_automatic_cache_eviction, m)?)?;
    m.add_function(wrap_pyfunction!(evict_cache, m)?)?;
    m.add_class::<CompilationMode>()?;
    m.add_class::<BlobWithMetadata>()?;
    Ok(())
}
