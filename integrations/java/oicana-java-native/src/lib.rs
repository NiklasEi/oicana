//! The Java (JNI) integration of Oicana.
//!
//! You will want to use this through the `com.oicana:oicana` Maven package.

#![deny(clippy::all)]

use dashmap::DashMap;
use jni::errors::Error;
use jni::objects::{JByteArray, JClass, JMap, JObject, JString};
use jni::strings::JNIString;
use jni::sys::jint;
use jni::{jni_sig, Env, EnvUnowned};
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

static WORLD_CACHE: Lazy<DashMap<String, OicanaWorld<PackedTemplate>>> = Lazy::new(DashMap::new);
static DOCUMENT_CACHE: Lazy<DashMap<String, PagedDocument>> = Lazy::new(DashMap::new);

/// Global cache age configuration.
/// Default is 10. usize::MAX means disabled.
static CACHE_EVICTION_AGE: AtomicUsize = AtomicUsize::new(10);

fn new_document_id(template_id: &str) -> String {
    format!("{}:{}", Uuid::new_v4(), template_id)
}

fn template_id_from_document_id(document_id: &str) -> &str {
    &document_id[37..]
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

/// Throw a Java OicanaException, returning Error::JavaException for use with `?`.
fn throw_oicana(env: &mut Env, message: &str) -> Error {
    let _ = env.throw_new(
        JNIString::from("com/oicana/OicanaException"),
        JNIString::from(message),
    );
    Error::JavaException
}

/// Safely cast a JObject to JString (the caller must ensure the object is a java.lang.String).
unsafe fn jobject_as_jstring<'local>(env: &Env<'local>, obj: JObject<'local>) -> JString<'local> {
    unsafe { JString::from_raw(env, obj.into_raw()) }
}

/// Extract a HashMap<String, String> from a Java Map<String, String>.
fn extract_string_map<'local>(
    env: &mut Env<'local>,
    map: JObject<'local>,
) -> Result<HashMap<String, String>, Error> {
    if map.is_null() {
        return Ok(HashMap::new());
    }
    let jmap: JMap = env.cast_local::<JMap>(map)?;
    let mut iter = jmap.iter(env)?;
    let mut result = HashMap::new();
    while let Some(entry) = iter.next(env)? {
        let key = entry.key(env)?;
        let value = entry.value(env)?;
        // Safety: we know this is a Map<String, String> from the Java side
        let key_jstr = unsafe { jobject_as_jstring(env, key) };
        let value_jstr = unsafe { jobject_as_jstring(env, value) };
        let key_str = key_jstr.try_to_string(env)?;
        let value_str = value_jstr.try_to_string(env)?;
        result.insert(key_str, value_str);
    }
    Ok(result)
}

/// A blob with its metadata, extracted from a Java BlobWithMetadata object.
struct NativeBlobWithMetadata {
    bytes: Vec<u8>,
    meta: String,
}

/// Extract a HashMap<String, BlobWithMetadata> from a Java Map<String, BlobWithMetadata>.
fn extract_blob_map<'local>(
    env: &mut Env<'local>,
    map: JObject<'local>,
) -> Result<HashMap<String, NativeBlobWithMetadata>, Error> {
    if map.is_null() {
        return Ok(HashMap::new());
    }
    let jmap: JMap = env.cast_local::<JMap>(map)?;
    let mut iter = jmap.iter(env)?;
    let mut result = HashMap::new();
    while let Some(entry) = iter.next(env)? {
        let key = entry.key(env)?;
        let value = entry.value(env)?;
        // Safety: keys are Strings from the Java side
        let key_str = unsafe { jobject_as_jstring(env, key) }.try_to_string(env)?;

        // Get bytes field
        let bytes_obj = env
            .get_field(&value, JNIString::from("bytes"), jni_sig!("[B"))?
            .l()?;
        // Safety: we know the field type is byte[]
        let bytes_array = unsafe { JByteArray::from_raw(env, bytes_obj.into_raw()) };
        let bytes = env.convert_byte_array(bytes_array)?;

        // Get meta field
        let meta_obj = env
            .get_field(
                &value,
                JNIString::from("meta"),
                jni_sig!("Ljava/lang/String;"),
            )?
            .l()?;
        // Safety: we know the field type is String
        let meta = unsafe { jobject_as_jstring(env, meta_obj) }.try_to_string(env)?;

        result.insert(key_str, NativeBlobWithMetadata { bytes, meta });
    }
    Ok(result)
}

fn prepare_inputs(
    json_inputs: HashMap<String, String>,
    blob_inputs: HashMap<String, NativeBlobWithMetadata>,
) -> Result<TemplateInputs, String> {
    let mut inputs = TemplateInputs::new();
    for (key, value) in json_inputs {
        inputs.with_input(JsonInput::new(key, value));
    }
    for (key, value) in blob_inputs {
        let mut blob = Blob::from(Bytes::new(value.bytes));
        let json_value = serde_json::Value::from_str(&value.meta).map_err(|e| e.to_string())?;
        blob.metadata = Deserialize::deserialize(json_value).map_err(|e| e.to_string())?;
        inputs.with_input(BlobInput::new(key, blob));
    }
    Ok(inputs)
}

fn compilation_config_from_mode(mode: jint) -> CompilationConfig {
    match mode {
        1 => CompilationConfig::development(),
        _ => CompilationConfig::production(),
    }
}

fn evict_if_configured() {
    let cache_age = CACHE_EVICTION_AGE.load(Ordering::Relaxed);
    if cache_age != usize::MAX {
        oicana_world::evict_cache(cache_age);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_registerTemplate<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    template_id: JString<'local>,
    files: JByteArray<'local>,
    json_inputs: JObject<'local>,
    blob_inputs: JObject<'local>,
    compilation_mode: jint,
) -> JString<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JString<'_>> {
            let template_id = template_id.try_to_string(env)?;
            let file_bytes = env.convert_byte_array(files)?;
            let json_map = extract_string_map(env, json_inputs)?;
            let blob_map = extract_blob_map(env, blob_inputs)?;

            let packed = PackedTemplate::new(Cursor::new(file_bytes))
                .map_err(|e| throw_oicana(env, &e.to_string()))?;
            let manifest = packed
                .manifest()
                .map_err(|e| throw_oicana(env, &e.to_string()))?;

            let mut inputs =
                prepare_inputs(json_map, blob_map).map_err(|e| throw_oicana(env, &e))?;
            inputs.with_config(compilation_config_from_mode(compilation_mode));

            let mut world = OicanaWorld::new(packed, inputs, manifest)
                .map_err(|e| throw_oicana(env, &e.to_string()))?;
            world.color = DiagnosticColor::None;

            let document = world
                .compile()
                .map_err(|e| throw_oicana(env, &e.to_string()))?;
            let result_id = new_document_id(&template_id);

            WORLD_CACHE.insert(template_id, world);
            DOCUMENT_CACHE.insert(result_id.clone(), document.document);

            evict_if_configured();

            Ok(JString::from_str(env, &result_id)?)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_compileTemplate<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    template_id: JString<'local>,
    json_inputs: JObject<'local>,
    blob_inputs: JObject<'local>,
    compilation_mode: jint,
) -> JString<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JString<'_>> {
            let template_id = template_id.try_to_string(env)?;

            let Some(mut world) = WORLD_CACHE.get_mut(&template_id) else {
                return Err(throw_oicana(env, "Template was not registered"));
            };

            let json_map = extract_string_map(env, json_inputs)?;
            let blob_map = extract_blob_map(env, blob_inputs)?;

            let mut inputs =
                prepare_inputs(json_map, blob_map).map_err(|e| throw_oicana(env, &e))?;
            inputs.with_config(compilation_config_from_mode(compilation_mode));
            world
                .update_inputs(inputs)
                .map_err(|e| throw_oicana(env, &e.to_string()))?;

            let document = world
                .compile()
                .map_err(|e| throw_oicana(env, &e.to_string()))?;
            let result_id = new_document_id(&template_id);
            DOCUMENT_CACHE.insert(result_id.clone(), document.document);

            evict_if_configured();

            Ok(JString::from_str(env, &result_id)?)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_exportDocument<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    document_id: JString<'local>,
    export_format: JString<'local>,
) -> JByteArray<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JByteArray<'_>> {
            let document_id = document_id.try_to_string(env)?;
            let export_format_str = export_format.try_to_string(env)?;

            let Some(document) = DOCUMENT_CACHE.get(&document_id) else {
                return Err(throw_oicana(env, "Document not found"));
            };

            let format: ExportFormat = serde_json::from_str(&export_format_str)
                .map_err(|e| throw_oicana(env, &e.to_string()))?;

            let bytes = match format {
                ExportFormat::Png { pixels_per_pt } => export_merged_png(&document, pixels_per_pt)
                    .map_err(|e| throw_oicana(env, &format!("Failed to encode PNG: {e:?}")))?,
                ExportFormat::Pdf => {
                    let template_id = template_id_from_document_id(&document_id);
                    let Some(world) = WORLD_CACHE.get(template_id) else {
                        return Err(throw_oicana(
                            env,
                            &format!(
                                "World '{template_id}' for document '{document_id}' not found"
                            ),
                        ));
                    };
                    export_merged_pdf(
                        &document,
                        &*world,
                        &world.manifest().tool.oicana.export.pdf.standards,
                    )
                    .map_err(|e| throw_oicana(env, &format!("Failed to encode PDF: {e:?}")))?
                }
                ExportFormat::Svg => export_merged_svg(&document),
            };

            Ok(env.byte_array_from_slice(&bytes)?)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_removeDocument<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    document_id: JString<'local>,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<()> {
            let id = document_id.try_to_string(env)?;
            DOCUMENT_CACHE.remove(&id);
            Ok(())
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_removeWorld<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    template_id: JString<'local>,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<()> {
            let id = template_id.try_to_string(env)?;
            WORLD_CACHE.remove(&id);
            Ok(())
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_inputs<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    template_id: JString<'local>,
) -> JString<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JString<'_>> {
            let template_id = template_id.try_to_string(env)?;

            let Some(world) = WORLD_CACHE.get(&template_id) else {
                return Err(throw_oicana(env, "Template was not registered"));
            };
            let oicana_config = &world.manifest().tool.oicana;
            let json = serde_json::ser::to_string(&oicana_config)
                .map_err(|e| throw_oicana(env, &e.to_string()))?;

            Ok(JString::from_str(env, &json)?)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_getSource<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    template_id: JString<'local>,
    file: JString<'local>,
) -> JString<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JString<'_>> {
            let template_id = template_id.try_to_string(env)?;
            let file_path = file.try_to_string(env)?;

            let Some(world) = WORLD_CACHE.get(&template_id) else {
                return Err(throw_oicana(env, "Template was not registered"));
            };
            let source = world
                .files
                .source(FileId::new(None, VirtualPath::new(&file_path)))
                .map_err(|e| throw_oicana(env, &e.to_string()))?;

            Ok(JString::from_str(env, source.text())?)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_getFile<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    template_id: JString<'local>,
    file: JString<'local>,
) -> JByteArray<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JByteArray<'_>> {
            let template_id = template_id.try_to_string(env)?;
            let file_path = file.try_to_string(env)?;

            let Some(world) = WORLD_CACHE.get(&template_id) else {
                return Err(throw_oicana(env, "Template was not registered"));
            };
            let bytes = world
                .files
                .file(FileId::new(None, VirtualPath::new(&file_path)))
                .map_err(|e| throw_oicana(env, &e.to_string()))?;

            Ok(env.byte_array_from_slice(&bytes)?)
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_setValidateInputs<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    template_id: JString<'local>,
    validate: u8,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<()> {
            let template_id = template_id.try_to_string(env)?;
            let Some(mut world) = WORLD_CACHE.get_mut(&template_id) else {
                return Err(throw_oicana(env, "Template was not registered"));
            };
            world.validate_inputs = validate != 0;
            Ok(())
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_configureAutomaticCacheEviction<'local>(
    _unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    max_age: jint,
) {
    if max_age < 0 {
        CACHE_EVICTION_AGE.store(usize::MAX, Ordering::Relaxed);
    } else {
        CACHE_EVICTION_AGE.store(max_age as usize, Ordering::Relaxed);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_evictCache<'local>(
    _unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    max_age: jint,
) {
    oicana_world::evict_cache(max_age as usize);
}
