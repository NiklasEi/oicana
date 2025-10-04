//! The Node.js integration of Oicana.
//!
//! You will want to use this through the npm package `@oicana/node`.

#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::{Buffer, Result, Uint8Array};
use napi::Error;
use oicana_files::packed::PackedTemplate;
use oicana_files::TemplateFiles;
use oicana_input::input::blob::{Blob, BlobInput};
use oicana_input::input::json::JsonInput;
use oicana_input::{CompilationConfig, TemplateInputs};
use oicana_world::manifest::OicanaWorldFiles;
use oicana_world::world::OicanaWorld;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Mutex;
use typst::foundations::{Bytes, Smart, Value};
use typst::layout::{Abs, PagedDocument};
use typst::syntax::{FileId, VirtualPath};
use typst::utils::once_cell::sync::OnceCell;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};

/// Error string when a requested template is not registered yet. Call `[register_template]` before
/// trying to use the template through a different method.
#[napi]
pub const NOT_REGISTERED: &str = "Template is not registered";

/// Register the given template. This will read the template files as a [`PackedTemplate`] and
/// compile it once with the given inputs. The Typst [`typst::World`] will be cached and reused for
/// subsequent calls to the other methods with the same template identifier.
#[napi]
pub fn register_template(
  template: String,
  files: Uint8Array,
  json_inputs: HashMap<String, String>,
  blob_inputs: HashMap<String, BlobWithMetadata>,
  export_format: ExportFormat,
) -> Result<Buffer> {
  let files = PackedTemplate::new(Cursor::new(files));
  let manifest = files
    .manifest()
    .map_err(|error| Error::from_reason(error.to_string()))?;
  println!("inserting new world for template '{template}'");

  let inputs = prepare_inputs(json_inputs, blob_inputs);
  let mut zip_world = OicanaWorld::new(files, inputs, manifest)
    .map_err(|error| Error::from_reason(error.to_string()))?;

  let document = zip_world
    .compile()
    .map_err(|error| Error::from_reason(error.to_string()))?;
  world().lock().unwrap().insert(template, zip_world);

  export(&document.document, export_format)
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
  export_format: ExportFormat,
) -> Result<Buffer> {
  let mut cache_lock = world().lock().unwrap();
  let Some(world) = cache_lock.get_mut(&template) else {
    return Err(Error::from_reason("Template was not registered"));
  };
  let mut inputs = prepare_inputs(json_inputs, blob_inputs);
  inputs.with_config(CompilationConfig::production());
  world.update_inputs(inputs);

  let document = world
    .compile()
    .map_err(|error| Error::from_reason(error.to_string()))?;

  export(&document.document, export_format)
}

/// Load all input definitions for the given template.
///
/// Calling this method requires a previous call to [`register_template`] with the same template
/// identifier.
#[napi]
pub fn inputs(template: String) -> Result<String> {
  let mut cache_lock = world().lock().unwrap();
  let Some(world) = cache_lock.get_mut(&template) else {
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
  let mut cache_lock = world().lock().unwrap();
  let Some(world) = cache_lock.get_mut(&template) else {
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
  let mut cache_lock = world().lock().unwrap();
  let Some(world) = cache_lock.get_mut(&template) else {
    return Err(Error::from_reason("Template was not registered"));
  };
  let bytes = world
    .files
    .file(FileId::new(None, VirtualPath::new(file)))
    .map_err(|error| Error::from_reason(error.to_string()))?;
  Ok(bytes.to_vec().into()) // This is currently copying!
}

fn export(document: &PagedDocument, export_format: ExportFormat) -> Result<Buffer> {
  match export_format {
    ExportFormat::Png => Ok(create_image(document, 1.)?.into()),
    ExportFormat::Pdf => {
      let options = PdfOptions {
        ident: Smart::Auto,
        timestamp: None,
        page_ranges: None,
        standards: PdfStandards::new(&[PdfStandard::A_3b]).expect("Invalid PDF standards"),
      };
      let pdf = typst_pdf::pdf(document, &options)
        .map_err(|error| Error::from_reason(format!("{error:?}")))?;

      Ok(pdf.into())
    }
  }
}

fn prepare_inputs(
  json_inputs: HashMap<String, String>,
  blob_inputs: HashMap<String, BlobWithMetadata>,
) -> TemplateInputs {
  let mut inputs = TemplateInputs::new();
  add_json_inputs(&mut inputs, json_inputs);
  add_blob_inputs(&mut inputs, blob_inputs);
  inputs
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
) {
  for (key, value) in blob_inputs.drain() {
    let mut blob = Blob::from(Bytes::new(value.bytes.to_vec()));
    blob.metadata.extend(
      value
        .meta
        .into_iter()
        .map(|(meta_key, meta_value)| (meta_key.into(), Value::Str(meta_value.into()))),
    );
    inputs.with_input(BlobInput::new(key, blob));
  }
}

fn create_image(document: &PagedDocument, pixels_per_pt: f32) -> Result<Vec<u8>> {
  typst_render::render_merged(document, pixels_per_pt, Abs::pt(15.), None)
    .encode_png()
    .map_err(|error| Error::from_reason(format!("{error:?}")))
}

/// The supported export formats of a template.
#[napi]
pub enum ExportFormat {
  /// Render the template into a png image.
  ///
  /// The image is not optimized for file size to speed up compiling.
  Png,
  /// Render the template to a PDF file.
  ///
  /// The currently exported standard is PDF/A-3b
  Pdf,
}

/// A blob with its metadata.
#[napi(object)]
pub struct BlobWithMetadata {
  /// The byte content of the blob.
  pub bytes: Uint8Array,
  /// Metadata of the blob.
  pub meta: HashMap<String, String>,
}

fn world() -> &'static Mutex<HashMap<String, OicanaWorld<PackedTemplate>>> {
  static ZIPPED_WORLD: OnceCell<Mutex<HashMap<String, OicanaWorld<PackedTemplate>>>> =
    OnceCell::new();
  ZIPPED_WORLD.get_or_init(|| Mutex::new(HashMap::new()))
}
