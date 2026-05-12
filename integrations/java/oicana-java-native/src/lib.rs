//! The Java (JNI) integration of Oicana.
//!
//! You will want to use this through the `com.oicana:oicana` Maven package.

#![deny(clippy::all)]

use std::collections::HashMap;

use jni::errors::Error;
use jni::objects::{JByteArray, JClass, JMap, JObject, JString};
use jni::strings::JNIString;
use jni::sys::jint;
use jni::{jni_sig, Env, EnvUnowned};

use oicana_ffi_core as core;

/// Throw a Java OicanaException, returning Error::JavaException for use with `?`.
fn throw_oicana(env: &mut Env, message: &str) -> Error {
    let _ = env.throw_new(
        JNIString::from("com/oicana/OicanaException"),
        JNIString::from(message),
    );
    Error::JavaException
}

fn throw_ffi(env: &mut Env, error: core::FfiError) -> Error {
    throw_oicana(env, &error.to_string())
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

/// Extract a HashMap<String, core::BlobWithMetadata> from a Java
/// Map<String, BlobWithMetadata>.
fn extract_blob_map<'local>(
    env: &mut Env<'local>,
    map: JObject<'local>,
) -> Result<HashMap<String, core::BlobWithMetadata>, Error> {
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

        result.insert(key_str, core::BlobWithMetadata { bytes, meta });
    }
    Ok(result)
}

fn compilation_mode_from_jint(mode: jint) -> core::CompilationMode {
    match mode {
        1 => core::CompilationMode::Development,
        _ => core::CompilationMode::Production,
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

            let result_id = core::register_template(
                &template_id,
                &file_bytes,
                json_map,
                blob_map,
                compilation_mode_from_jint(compilation_mode),
            )
            .map_err(|e| throw_ffi(env, e))?;

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
            let json_map = extract_string_map(env, json_inputs)?;
            let blob_map = extract_blob_map(env, blob_inputs)?;

            let result_id = core::compile_template(
                &template_id,
                json_map,
                blob_map,
                compilation_mode_from_jint(compilation_mode),
            )
            .map_err(|e| throw_ffi(env, e))?;

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

            let format =
                core::parse_export_format(&export_format_str).map_err(|e| throw_ffi(env, e))?;
            let bytes =
                core::export_document(&document_id, format).map_err(|e| throw_ffi(env, e))?;

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
            core::remove_document(&id);
            Ok(())
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// Return any compilation warnings produced for the given document, or `null`
/// if there were none. Warnings are cleared when the document is removed.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_getWarnings<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    document_id: JString<'local>,
) -> JString<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JString<'_>> {
            let id = document_id.try_to_string(env)?;
            match core::get_warnings(&id) {
                Some(warnings) => Ok(JString::from_str(env, &warnings)?),
                None => Ok(JString::default()),
            }
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
            core::remove_world(&id);
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
            let json = core::inputs(&template_id).map_err(|e| throw_ffi(env, e))?;
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
            let source =
                core::get_source(&template_id, &file_path).map_err(|e| throw_ffi(env, e))?;
            Ok(JString::from_str(env, &source)?)
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
            let bytes = core::get_file(&template_id, &file_path).map_err(|e| throw_ffi(env, e))?;
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
            core::set_validate_inputs(&template_id, validate != 0)
                .map_err(|e| throw_ffi(env, e))?;
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
    let max_age = if max_age < 0 {
        None
    } else {
        Some(max_age as usize)
    };
    core::configure_automatic_cache_eviction(max_age);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_oicana_OicanaNative_evictCache<'local>(
    _unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    max_age: jint,
) {
    if max_age >= 0 {
        core::evict_cache(max_age as usize);
    }
}
