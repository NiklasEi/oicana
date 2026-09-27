//! Template checks for validation.

use oicana::export::pdf::{validate_pdf_standards, validate_pdf_tagging};
use oicana::input::InputDefinition;
use oicana::template::manifest::TemplateManifest;
use std::collections::HashMap;
use std::path::Path;

/// Which JSON inputs get their schema checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaSelection {
    /// Only inputs with `validate = true`.
    Validated,
    /// All inputs with a schema.
    All,
}

/// Compiled schemas by input key, together with the problems found while checking them.
#[derive(Default)]
pub struct SchemaCheck {
    pub validators: HashMap<String, jsonschema::Validator>,
    pub errors: Vec<String>,
}

/// Check that the entrypoint exists and is part of the packed template.
pub fn check_entrypoint(template_path: &Path, manifest: &TemplateManifest) -> Result<(), String> {
    let entrypoint = manifest.package.entrypoint.as_str();
    if !template_path.join(entrypoint).exists() {
        return Err(format!("entrypoint file '{entrypoint}' does not exist"));
    }
    if pack_exclusion(manifest)(entrypoint) {
        return Err(format!(
            "entrypoint file '{entrypoint}' is excluded from the packed template"
        ));
    }
    Ok(())
}

/// Compile the schemas of the selected JSON inputs.
///
/// Schemas of validated inputs must also be part of the packed template.
pub fn check_schemas(
    template_path: &Path,
    manifest: &TemplateManifest,
    selection: SchemaSelection,
) -> SchemaCheck {
    let mut check = SchemaCheck::default();
    let is_excluded = pack_exclusion(manifest);

    for input in &manifest.tool.oicana.inputs {
        let InputDefinition::Json(json_def) = input else {
            continue;
        };
        let Some(schema_path) = &json_def.schema else {
            continue;
        };
        if selection == SchemaSelection::Validated && !json_def.validate {
            continue;
        }

        match compile_schema(template_path, &json_def.key, schema_path) {
            Ok(validator) => {
                check.validators.insert(json_def.key.clone(), validator);
            }
            Err(error) => {
                check.errors.push(error);
                continue;
            }
        }
        if json_def.validate && is_excluded(schema_path) {
            check.errors.push(format!(
                "Input '{}': schema file '{schema_path}' is excluded from the packed template",
                json_def.key
            ));
        }
    }

    check
}

/// Check that the PDF export configuration is valid.
pub fn check_pdf_export(manifest: &TemplateManifest) -> Vec<String> {
    let pdf = &manifest.tool.oicana.export.pdf;
    [
        validate_pdf_standards(&pdf.standards),
        validate_pdf_tagging(&pdf.standards, pdf.tagged),
    ]
    .into_iter()
    .filter_map(Result::err)
    .collect()
}

/// Check that the fallback value files of all inputs exist and match their compiled schema.
pub fn check_fallback_values(
    template_path: &Path,
    inputs: &[InputDefinition],
    validators: &HashMap<String, jsonschema::Validator>,
) -> Vec<String> {
    let mut errors = Vec::new();

    for input in inputs {
        match input {
            InputDefinition::Json(json_def) => {
                let validator = validators.get(&json_def.key).zip(json_def.schema.as_ref());

                for (label, file_path) in [
                    ("default", &json_def.default),
                    ("development", &json_def.development),
                ] {
                    let Some(file_path) = file_path else {
                        continue;
                    };

                    let content = match std::fs::read(template_path.join(file_path)) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            errors.push(format!(
                                "Input '{}': failed to read {label} value file '{file_path}': {e}",
                                json_def.key,
                            ));
                            continue;
                        }
                    };

                    let Some((validator, schema_path)) = validator else {
                        continue;
                    };

                    let parsed: serde_json::Value = match serde_json::from_slice(&content) {
                        Ok(v) => v,
                        Err(e) => {
                            errors.push(format!(
                                "Input '{}': {label} value file '{file_path}' is not valid JSON: {e}",
                                json_def.key,
                            ));
                            continue;
                        }
                    };

                    if !validator.is_valid(&parsed) {
                        let validation_errors: Vec<String> = validator
                            .iter_errors(&parsed)
                            .map(|error| {
                                let path = error.instance_path().to_string();
                                if path.is_empty() {
                                    error.to_string()
                                } else {
                                    format!("  at {path}: {error}")
                                }
                            })
                            .collect();

                        errors.push(format!(
                            "Input '{}': {label} value file '{file_path}' does not match schema '{schema_path}':\n{}",
                            json_def.key,
                            validation_errors.join("\n"),
                        ));
                    }
                }
            }
            InputDefinition::Blob(blob_def) => {
                for (label, fallback) in [
                    ("default", &blob_def.default),
                    ("development", &blob_def.development),
                ] {
                    let Some(fallback) = fallback else {
                        continue;
                    };

                    let file_path = &fallback.file;
                    if let Err(e) = std::fs::metadata(template_path.join(file_path)) {
                        errors.push(format!(
                            "Input '{}': failed to read {label} value file '{file_path}': {e}",
                            blob_def.key,
                        ));
                    }
                }
            }
        }
    }

    errors
}

/// Collect required inputs without a `default` or `development` value.
pub fn missing_fallbacks(inputs: &[InputDefinition]) -> Vec<String> {
    inputs
        .iter()
        .filter(|input| input.required() && !has_fallback(input))
        .map(|input| {
            format!(
                "Input '{}' is required but has no default or development value. \
                 Compiling without a supplied value for it will fail, including the warm-up \
                 compilation that most integrations run when registering the template. \
                 Set a 'development' value that applies outside of production mode only, \
                 or a 'default' which also applies in production.",
                input.key(),
            )
        })
        .collect()
}

/// Whether the input has a `default` or `development` value configured.
fn has_fallback(input: &InputDefinition) -> bool {
    match input {
        InputDefinition::Json(def) => def.default.is_some() || def.development.is_some(),
        InputDefinition::Blob(def) => def.default.is_some() || def.development.is_some(),
    }
}

/// Build a predicate telling whether a template file is left out of the packed template.
fn pack_exclusion(manifest: &TemplateManifest) -> impl Fn(&str) -> bool {
    let exclude_matcher = manifest.build_exclude_matcher();
    move |file| {
        exclude_matcher
            .matched_path_or_any_parents(file.trim_start_matches('/'), false)
            .is_ignore()
    }
}

/// Read and compile the JSON schema of an input.
fn compile_schema(
    template_path: &Path,
    key: &str,
    schema_path: &str,
) -> Result<jsonschema::Validator, String> {
    let schema_bytes = std::fs::read(template_path.join(schema_path))
        .map_err(|e| format!("Input '{key}': failed to read schema file '{schema_path}': {e}"))?;

    let schema_value: serde_json::Value = serde_json::from_slice(&schema_bytes)
        .map_err(|e| format!("Input '{key}': failed to parse schema file '{schema_path}': {e}"))?;

    jsonschema::validator_for(&schema_value)
        .map_err(|e| format!("Input '{key}': failed to compile schema '{schema_path}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oicana::input::{BlobInputDefinition, FallbackBlobInput, JsonInputDefinition};
    use std::fs::{create_dir_all, write, File};
    use tempfile::tempdir;

    const SCHEMA: &str = r#"{ "type": "object" }"#;

    fn manifest(extra: &str) -> TemplateManifest {
        TemplateManifest::from_toml(&format!(
            r#"[package]
name = "test"
version = "0.1.0"
entrypoint = "main.typ"

[tool.oicana]
manifest_version = 1
{extra}"#
        ))
        .unwrap()
    }

    fn schema_input(schema: &str, validate: bool) -> String {
        format!(
            "\n[[tool.oicana.inputs]]\ntype = \"json\"\nkey = \"data\"\nschema = \"{schema}\"\nvalidate = {validate}\n"
        )
    }

    fn template_dir() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        write(dir.path().join("main.typ"), "= Hello").unwrap();
        dir
    }

    #[test]
    fn existing_entrypoint_passes() {
        let dir = template_dir();

        assert_eq!(check_entrypoint(dir.path(), &manifest("")), Ok(()));
    }

    #[test]
    fn missing_entrypoint_is_an_error() {
        let dir = tempdir().unwrap();

        assert_eq!(
            check_entrypoint(dir.path(), &manifest("")),
            Err("entrypoint file 'main.typ' does not exist".to_string())
        );
    }

    #[test]
    fn excluded_entrypoint_is_an_error() {
        let dir = template_dir();
        let manifest = TemplateManifest::from_toml(
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nentrypoint = \"main.typ\"\nexclude = [\"main.typ\"]\n\n[tool.oicana]\nmanifest_version = 1\n",
        )
        .unwrap();

        assert_eq!(
            check_entrypoint(dir.path(), &manifest),
            Err("entrypoint file 'main.typ' is excluded from the packed template".to_string())
        );
    }

    #[test]
    fn valid_schema_is_compiled() {
        let dir = template_dir();
        write(dir.path().join("data.schema.json"), SCHEMA).unwrap();

        let check = check_schemas(
            dir.path(),
            &manifest(&schema_input("data.schema.json", true)),
            SchemaSelection::Validated,
        );
        assert!(check.errors.is_empty(), "got: {:?}", check.errors);
        assert!(check.validators.contains_key("data"));
    }

    #[test]
    fn missing_schema_is_an_error() {
        let dir = template_dir();

        let errors = check_schemas(
            dir.path(),
            &manifest(&schema_input("nope.json", true)),
            SchemaSelection::Validated,
        )
        .errors;
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].contains("failed to read schema file 'nope.json'"),
            "got: {errors:?}"
        );
    }

    #[test]
    fn invalid_schema_is_an_error() {
        let dir = template_dir();
        write(dir.path().join("data.schema.json"), r#"{ "type": 5 }"#).unwrap();

        let errors = check_schemas(
            dir.path(),
            &manifest(&schema_input("data.schema.json", true)),
            SchemaSelection::Validated,
        )
        .errors;
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].contains("failed to compile schema"),
            "got: {errors:?}"
        );
    }

    #[test]
    fn schema_in_the_default_excluded_tests_directory_is_an_error() {
        let dir = template_dir();
        create_dir_all(dir.path().join("tests")).unwrap();
        write(dir.path().join("tests/data.schema.json"), SCHEMA).unwrap();

        let errors = check_schemas(
            dir.path(),
            &manifest(&schema_input("tests/data.schema.json", true)),
            SchemaSelection::Validated,
        )
        .errors;
        assert_eq!(
            errors,
            ["Input 'data': schema file 'tests/data.schema.json' is excluded from the packed template"]
        );
    }

    #[test]
    fn excluded_schema_of_an_unvalidated_input_is_fine() {
        let dir = template_dir();
        create_dir_all(dir.path().join("tests")).unwrap();
        write(dir.path().join("tests/data.schema.json"), SCHEMA).unwrap();

        let errors = check_schemas(
            dir.path(),
            &manifest(&schema_input("tests/data.schema.json", false)),
            SchemaSelection::All,
        )
        .errors;
        assert!(errors.is_empty(), "got: {errors:?}");
    }

    #[test]
    fn schema_of_an_unvalidated_input_is_skipped_for_validated_selection() {
        let dir = template_dir();

        let errors = check_schemas(
            dir.path(),
            &manifest(&schema_input("nope.json", false)),
            SchemaSelection::Validated,
        )
        .errors;
        assert!(errors.is_empty(), "got: {errors:?}");
    }

    #[test]
    fn schema_of_an_unvalidated_input_is_checked_for_all_selection() {
        let dir = template_dir();

        let errors = check_schemas(
            dir.path(),
            &manifest(&schema_input("nope.json", false)),
            SchemaSelection::All,
        )
        .errors;
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].contains("failed to read schema file 'nope.json'"),
            "got: {errors:?}"
        );
    }

    #[test]
    fn default_pdf_export_is_valid() {
        assert!(check_pdf_export(&manifest("")).is_empty());
    }

    #[test]
    fn incompatible_pdf_standards_are_an_error() {
        let errors = check_pdf_export(&manifest(
            "\n[tool.oicana.export.pdf]\nstandards = [\"a-4\", \"ua-1\"]\n",
        ));
        assert_eq!(errors.len(), 1, "got: {errors:?}");
    }

    #[test]
    fn tagged_standard_without_tagging_is_an_error() {
        let errors = check_pdf_export(&manifest(
            "\n[tool.oicana.export.pdf]\nstandards = [\"ua-1\"]\ntagged = false\n",
        ));
        assert_eq!(errors.len(), 1, "got: {errors:?}");
    }

    const VALUE_SCHEMA: &str = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name"]
    }"#;

    fn json_input(default: Option<&str>, development: Option<&str>) -> InputDefinition {
        InputDefinition::Json(JsonInputDefinition {
            key: "data".to_string(),
            required: true,
            default: default.map(str::to_string),
            development: development.map(str::to_string),
            schema: None,
            validate: true,
        })
    }

    fn blob_input(default: Option<&str>, development: Option<&str>) -> InputDefinition {
        let fallback = |file: &str| FallbackBlobInput {
            file: file.to_string(),
            meta: None,
        };
        InputDefinition::Blob(BlobInputDefinition {
            key: "logo".to_string(),
            required: true,
            default: default.map(fallback),
            development: development.map(fallback),
        })
    }

    /// Write the given fallback values and check them against [`VALUE_SCHEMA`].
    fn check_values(default_value: Option<&str>, dev_value: Option<&str>) -> Vec<String> {
        let dir = tempdir().unwrap();
        if let Some(value) = default_value {
            write(dir.path().join("default.json"), value).unwrap();
        }
        if let Some(value) = dev_value {
            write(dir.path().join("dev.json"), value).unwrap();
        }

        let InputDefinition::Json(mut json_def) = json_input(
            default_value.map(|_| "default.json"),
            dev_value.map(|_| "dev.json"),
        ) else {
            unreachable!("json_input builds a JSON input");
        };
        json_def.schema = Some("data.schema.json".to_string());

        let value_schema = serde_json::from_str(VALUE_SCHEMA).unwrap();
        let validators = HashMap::from([(
            "data".to_string(),
            jsonschema::validator_for(&value_schema).unwrap(),
        )]);

        check_fallback_values(dir.path(), &[InputDefinition::Json(json_def)], &validators)
    }

    #[test]
    fn valid_default_and_dev_values() {
        let errors = check_values(
            Some(r#"{"name": "Alice", "age": 30}"#),
            Some(r#"{"name": "Bob"}"#),
        );
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
    }

    #[test]
    fn invalid_default_value_reports_error() {
        let errors = check_values(
            Some(r#"{"age": 30}"#), // missing required "name"
            None,
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("default"));
        assert!(errors[0].contains("does not match schema"));
    }

    #[test]
    fn invalid_dev_value_reports_error() {
        let errors = check_values(
            None,
            Some(r#"{"name": 42}"#), // name should be string
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("development"));
        assert!(errors[0].contains("does not match schema"));
    }

    #[test]
    fn both_default_and_dev_invalid() {
        let errors = check_values(
            Some(r#"{"age": "not a number"}"#), // missing name, age wrong type
            Some(r#"[]"#),                      // wrong type entirely
        );
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("default"));
        assert!(errors[1].contains("development"));
    }

    #[test]
    fn invalid_json_in_file_reports_error() {
        let errors = check_values(Some("not valid json {"), None);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not valid JSON"));
    }

    #[test]
    fn no_schema_skips_validation() {
        let dir = tempdir().unwrap();
        write(dir.path().join("default.json"), "not even json").unwrap();
        let inputs = vec![json_input(Some("default.json"), None)];

        let errors = check_fallback_values(dir.path(), &inputs, &HashMap::new());
        assert!(errors.is_empty(), "No schema means no validation");
    }

    #[test]
    fn missing_json_fallback_files_report_errors() {
        let dir = tempdir().unwrap();
        let inputs = vec![json_input(Some("nonexistent.json"), Some("nope.json"))];

        let errors = check_fallback_values(dir.path(), &inputs, &HashMap::new());
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("failed to read default value file 'nonexistent.json'"));
        assert!(errors[1].contains("failed to read development value file 'nope.json'"));
    }

    #[test]
    fn missing_blob_fallback_files_report_errors() {
        let dir = tempdir().unwrap();
        let inputs = vec![blob_input(
            Some("missing-default.png"),
            Some("missing-dev.png"),
        )];

        let errors = check_fallback_values(dir.path(), &inputs, &HashMap::new());
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("failed to read default value file 'missing-default.png'"));
        assert!(errors[1].contains("failed to read development value file 'missing-dev.png'"));
    }

    #[test]
    fn existing_blob_fallback_file_passes() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("logo.png")).unwrap();
        let inputs = vec![blob_input(None, Some("logo.png"))];

        let errors = check_fallback_values(dir.path(), &inputs, &HashMap::new());
        assert!(errors.is_empty(), "got: {errors:?}");
    }

    #[test]
    fn required_input_without_any_fallback_warns() {
        let warnings = missing_fallbacks(&[json_input(None, None), blob_input(None, None)]);
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("Input 'data' is required"));
        assert!(warnings[1].contains("Input 'logo' is required"));
    }

    #[test]
    fn optional_input_without_fallback_does_not_warn() {
        let InputDefinition::Json(mut json_def) = json_input(None, None) else {
            unreachable!("json_input builds a JSON input");
        };
        json_def.required = false;

        let warnings = missing_fallbacks(&[InputDefinition::Json(json_def)]);
        assert!(warnings.is_empty(), "got: {warnings:?}");
    }

    #[test]
    fn required_input_with_fallback_does_not_warn() {
        let warnings = missing_fallbacks(&[json_input(None, Some("dev.json"))]);
        assert!(warnings.is_empty(), "got: {warnings:?}");
    }
}
