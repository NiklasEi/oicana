//! The Node.js integration of Oicana.
//!
//! You will want to use this through the npm package `@oicana/node`.

#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use dashmap::DashMap;
use napi::bindgen_prelude::{Buffer, Result, Uint8Array};
use napi::Error;
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
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use typst::foundations::Bytes;
use typst::layout::PagedDocument;
use typst::syntax::{FileId, VirtualPath};
use uuid::Uuid;

/// Error string when a requested template is not registered yet. Call `[register_template]` before
/// trying to use the template through a different method.
#[napi]
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
///   - `undefined`/`null` - Disables cache eviction (cache never cleared)
///   - `0` - Clears all cache entries with every eviction
///   - `1` - Keeps only entries used since the last eviction
///   - `n` - Keeps entries used within the last n evictions
#[napi]
pub fn configure_automatic_cache_eviction(max_age: Option<u32>) {
  CACHE_EVICTION_AGE.store(
    max_age.map(|v| v as usize).unwrap_or(usize::MAX),
    Ordering::Relaxed,
  );
}

/// Manually evict the comemo cache with the given age threshold.
///
/// This directly calls the underlying eviction with the specified age,
/// regardless of the configured default age.
#[napi]
pub fn evict_cache(max_age: u32) {
  oicana_world::evict_cache(max_age as usize);
}

/// Register the given template. This will read the template files as a [`PackedTemplate`] and
/// compile it once with the given inputs. The Typst [`typst::World`] will be cached and reused for
/// subsequent calls to the other methods with the same template identifier.
#[napi]
pub fn register_template(
  template: String,
  files: Uint8Array,
  json_inputs: HashMap<String, String>,
  blob_inputs: HashMap<String, BlobWithMetadata>,
  compilation_mode: CompilationMode,
) -> Result<String> {
  let files = PackedTemplate::new(Cursor::new(files))
    .map_err(|error| Error::from_reason(error.to_string()))?;
  let manifest = files
    .manifest()
    .map_err(|error| Error::from_reason(error.to_string()))?;

  let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
  inputs.with_config(compilation_mode.into());
  let mut zip_world = OicanaWorld::new(files, inputs, manifest)
    .map_err(|error| Error::from_reason(error.to_string()))?;
  zip_world.color = DiagnosticColor::None;

  let document = zip_world
    .compile()
    .map_err(|error| Error::from_reason(error.to_string()))?;

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
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[napi]
pub fn compile_template(
  template: String,
  json_inputs: HashMap<String, String>,
  blob_inputs: HashMap<String, BlobWithMetadata>,
  compilation_mode: CompilationMode,
) -> Result<String> {
  let Some(mut world) = WORLD_CACHE.get_mut(&template) else {
    return Err(Error::from_reason("Template was not registered"));
  };
  let mut inputs = prepare_inputs(json_inputs, blob_inputs)?;
  inputs.with_config(compilation_mode.into());
  world
    .update_inputs(inputs)
    .map_err(|error| Error::from_reason(error.to_string()))?;

  let document = world
    .compile()
    .map_err(|error| Error::from_reason(error.to_string()))?;

  let result_id = new_document_id(&template);
  DOCUMENT_CACHE.insert(result_id.clone(), document.document);

  let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
  if cache_age != usize::MAX {
    oicana_world::evict_cache(cache_age);
  }

  Ok(result_id)
}

fn new_document_id(template_id: &str) -> String {
  format!("{}:{}", Uuid::new_v4(), template_id)
}

fn template_id_from_document_id(document_id: &str) -> &str {
  &document_id[37..]
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[napi]
pub fn inputs(template: String) -> Result<String> {
  let Some(world) = WORLD_CACHE.get_mut(&template) else {
    return Err(Error::from_reason("Template was not registered"));
  };
  let oicana_config = &world.manifest().tool.oicana;

  serde_json::ser::to_string(&oicana_config).map_err(|error| Error::from_reason(error.to_string()))
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[napi]
pub fn get_source(template: String, file: String) -> Result<String> {
  let Some(world) = WORLD_CACHE.get_mut(&template) else {
    return Err(Error::from_reason("Template was not registered"));
  };
  world
    .files
    .source(FileId::new(None, VirtualPath::new(file)))
    .map_err(|error| Error::from_reason(error.to_string()))
    .map(|source| source.text().to_string())
}

/// Load the source of the given file in the template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[napi]
pub fn get_file(template: String, file: String) -> Result<Buffer> {
  let Some(world) = WORLD_CACHE.get_mut(&template) else {
    return Err(Error::from_reason("Template was not registered"));
  };
  let bytes = world
    .files
    .file(FileId::new(None, VirtualPath::new(file)))
    .map_err(|error| Error::from_reason(error.to_string()))?;
  Ok(bytes.to_vec().into()) // This is currently copying, although we own bytes here.
}

/// Export the given document
///
/// Make sure to call `removeDocument` with the documentId afterwards, to free the memory.
#[napi]
pub fn export_document(document_id: String, export_format: String) -> Result<Buffer> {
  let Some(document) = DOCUMENT_CACHE.get(&document_id) else {
    return Err(Error::from_reason("Document not found!"));
  };
  let export_format =
    serde_json::from_str(&export_format).map_err(|error| Error::from_reason(error.to_string()))?;
  match export_format {
    ExportFormat::Png { pixels_per_pt } => {
      let pix_map_result = export_merged_png(&document, pixels_per_pt);
      pix_map_result
        .map_err(|error| Error::from_reason(format!("Failed to encode PNG: {error:?}")))
        .map(|pix_map| pix_map.into())
    }
    ExportFormat::Pdf => {
      let template_id = template_id_from_document_id(&document_id);
      let Some(world) = WORLD_CACHE.get(template_id) else {
        return Err(Error::from_reason(format!(
          "World '{template_id}' for the given document '{document_id}' not found!"
        )));
      };

      export_merged_pdf(
        &document,
        &*world,
        &world.manifest().tool.oicana.export.pdf.standards,
      )
      .map_err(|error| Error::from_reason(format!("Failed to encode PDF: {error:?}")))
      .map(|pdf| pdf.into())
    }
    ExportFormat::Svg => {
      let svg = export_merged_svg(&document);

      Ok(svg.into())
    }
  }
}

/// Remove the document from the cache.
#[napi]
pub fn remove_document(document_id: String) -> Result<()> {
  DOCUMENT_CACHE.remove(&document_id);
  Ok(())
}

/// Remove the world from the cache.
///
/// The template will have to be registered again before it can be compiled again.
#[napi]
pub fn remove_world(template_id: String) -> Result<()> {
  WORLD_CACHE.remove(&template_id);
  Ok(())
}

fn prepare_inputs(
  json_inputs: HashMap<String, String>,
  blob_inputs: HashMap<String, BlobWithMetadata>,
) -> Result<TemplateInputs> {
  let mut inputs = TemplateInputs::new();
  add_json_inputs(&mut inputs, json_inputs);
  add_blob_inputs(&mut inputs, blob_inputs)?;
  Ok(inputs)
}

fn add_json_inputs(inputs: &mut TemplateInputs, mut json_inputs: HashMap<String, String>) {
  json_inputs
    .drain()
    .map(|(key, value)| JsonInput::new(key, value))
    .for_each(|input| {
      inputs.with_input(input);
    });
}

fn add_blob_inputs(
  inputs: &mut TemplateInputs,
  mut blob_inputs: HashMap<String, BlobWithMetadata>,
) -> Result<()> {
  for (key, value) in blob_inputs.drain() {
    let mut blob = Blob::from(Bytes::new(value.bytes.to_vec()));
    let json_value = serde_json::Value::from_str(&value.meta).map_err(|error| {
      Error::from_reason(format!(
        "Failed to parse metadata JSON for '{key}': {error:?}"
      ))
    })?;
    blob.metadata = Deserialize::deserialize(json_value).map_err(|error| {
      Error::from_reason(format!(
        "Failed to deserialize metadata for '{key}': {error:?}"
      ))
    })?;
    inputs.with_input(BlobInput::new(key, blob));
  }
  Ok(())
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

#[napi]
pub enum CompilationMode {
  Production,
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

/// A blob with its metadata.
#[napi(object)]
pub struct BlobWithMetadata {
  /// The byte content of the blob.
  pub bytes: Uint8Array,
  /// Metadata of the blob.
  pub meta: String,
}

static WORLD_CACHE: Lazy<DashMap<String, OicanaWorld<PackedTemplate>>> = Lazy::new(DashMap::new);

static DOCUMENT_CACHE: Lazy<DashMap<String, PagedDocument>> = Lazy::new(DashMap::new);
