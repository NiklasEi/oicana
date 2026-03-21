use crate::target::TargetArgs;
use clap::Args;
use console::{style, Emoji};
use log::info;
use oicana_input::input_definition::InputDefinition;
use oicana_template::validate_native_template;
use std::path::Path;

static CHECKMARK: Emoji<'_, '_> = Emoji("✔️", "");

#[derive(Debug, Args)]
pub struct ValidateArgs {
    #[clap(flatten)]
    target: TargetArgs,
}

#[rustfmt::skip]
pub const VALIDATE_AFTER_HELP: &str = color_print::cstr!("\
<s><u>Examples:</></>
  oicana validate templates/invoice
  oicana validate -a
  oicana validate templates -a
");

pub fn validate(args: ValidateArgs) -> anyhow::Result<()> {
    let templates = args.target.get_targets()?;

    let mut all_passed = true;
    let mut passed_count = 0;
    let template_count = templates.len();

    for template in templates {
        let validation_result = validate_native_template(&template.path);
        match validation_result {
            Err(e) => {
                eprintln!(
                    "Template {:?}: manifest validation failed: {e}",
                    template.path
                );
                all_passed = false;
            }
            Ok(manifest) => {
                info!("Template {:?}: manifest valid", template.path);

                let entrypoint = template.path.join(manifest.package.entrypoint.as_str());
                if !entrypoint.exists() {
                    eprintln!(
                        "Template {:?}: entrypoint file '{}' does not exist",
                        template.path, manifest.package.entrypoint
                    );
                    all_passed = false;
                    continue;
                }

                let fallback_errors =
                    validate_json_fallback_values(&template.path, &manifest.tool.oicana.inputs);

                if fallback_errors.is_empty() {
                    info!("Template {:?}: all checks passed", template.path);
                    passed_count += 1;
                    println!(
                        "{CHECKMARK}  {} valid",
                        style(&manifest.package.name).bold(),
                    );
                } else {
                    all_passed = false;
                    for error in &fallback_errors {
                        eprintln!("Template {:?}: {error}", template.path);
                    }
                }
            }
        }
    }

    if all_passed {
        println!(
            "\nValidated {} template{} successfully",
            passed_count,
            if template_count == 1 { "" } else { "s" },
        );
        Ok(())
    } else {
        anyhow::bail!("Validation failed for one or more templates.")
    }
}

/// Validate default and development JSON files against their schemas.
///
/// For each JSON input that has a schema, this checks that any configured
/// `default` or `development` value files exist, contain valid JSON, and
/// conform to the schema.
fn validate_json_fallback_values(template_path: &Path, inputs: &[InputDefinition]) -> Vec<String> {
    let mut errors = Vec::new();

    for input in inputs {
        let InputDefinition::Json(json_def) = input else {
            continue;
        };
        let Some(schema_path) = &json_def.schema else {
            continue;
        };

        let full_schema_path = template_path.join(schema_path);
        let schema_bytes = match std::fs::read(&full_schema_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                errors.push(format!(
                    "Input '{}': failed to read schema file '{}': {e}",
                    json_def.key, schema_path
                ));
                continue;
            }
        };

        let schema_value: serde_json::Value = match serde_json::from_slice(&schema_bytes) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!(
                    "Input '{}': failed to parse schema file '{}': {e}",
                    json_def.key, schema_path
                ));
                continue;
            }
        };

        let validator = match jsonschema::validator_for(&schema_value) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!(
                    "Input '{}': failed to compile schema '{}': {e}",
                    json_def.key, schema_path
                ));
                continue;
            }
        };

        for (label, file_path) in [
            ("default", &json_def.default),
            ("development", &json_def.development),
        ] {
            let Some(file_path) = file_path else {
                continue;
            };

            let full_path = template_path.join(file_path);
            let content = match std::fs::read(&full_path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    errors.push(format!(
                        "Input '{}': failed to read {label} value file '{file_path}': {e}",
                        json_def.key,
                    ));
                    continue;
                }
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

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn setup_template(
        schema: &str,
        default_value: Option<&str>,
        dev_value: Option<&str>,
    ) -> (tempfile::TempDir, Vec<InputDefinition>) {
        let dir = tempdir().unwrap();

        let mut schema_file = File::create(dir.path().join("data.schema.json")).unwrap();
        write!(schema_file, "{schema}").unwrap();

        if let Some(value) = default_value {
            let mut f = File::create(dir.path().join("default.json")).unwrap();
            write!(f, "{value}").unwrap();
        }

        if let Some(value) = dev_value {
            let mut f = File::create(dir.path().join("dev.json")).unwrap();
            write!(f, "{value}").unwrap();
        }

        let inputs = vec![InputDefinition::Json(
            oicana_input::input_definition::json::JsonInputDefinition {
                key: "data".to_string(),
                default: default_value.map(|_| "default.json".to_string()),
                development: dev_value.map(|_| "dev.json".to_string()),
                schema: Some("data.schema.json".to_string()),
                validate: true,
            },
        )];

        (dir, inputs)
    }

    const SCHEMA: &str = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name"]
    }"#;

    #[test]
    fn valid_default_and_dev_values() {
        let (dir, inputs) = setup_template(
            SCHEMA,
            Some(r#"{"name": "Alice", "age": 30}"#),
            Some(r#"{"name": "Bob"}"#),
        );

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
    }

    #[test]
    fn invalid_default_value_reports_error() {
        let (dir, inputs) = setup_template(
            SCHEMA,
            Some(r#"{"age": 30}"#), // missing required "name"
            None,
        );

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("default"));
        assert!(errors[0].contains("does not match schema"));
    }

    #[test]
    fn invalid_dev_value_reports_error() {
        let (dir, inputs) = setup_template(
            SCHEMA,
            None,
            Some(r#"{"name": 42}"#), // name should be string
        );

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("development"));
        assert!(errors[0].contains("does not match schema"));
    }

    #[test]
    fn both_default_and_dev_invalid() {
        let (dir, inputs) = setup_template(
            SCHEMA,
            Some(r#"{"age": "not a number"}"#), // missing name, age wrong type
            Some(r#"[]"#),                      // wrong type entirely
        );

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("default"));
        assert!(errors[1].contains("development"));
    }

    #[test]
    fn missing_default_file_reports_error() {
        let dir = tempdir().unwrap();

        let mut schema_file = File::create(dir.path().join("data.schema.json")).unwrap();
        write!(schema_file, "{SCHEMA}").unwrap();

        let inputs = vec![InputDefinition::Json(
            oicana_input::input_definition::json::JsonInputDefinition {
                key: "data".to_string(),
                default: Some("nonexistent.json".to_string()),
                development: None,
                schema: Some("data.schema.json".to_string()),
                validate: true,
            },
        )];

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("failed to read default value file"));
    }

    #[test]
    fn invalid_json_in_file_reports_error() {
        let dir = tempdir().unwrap();

        let mut schema_file = File::create(dir.path().join("data.schema.json")).unwrap();
        write!(schema_file, "{SCHEMA}").unwrap();

        let mut f = File::create(dir.path().join("default.json")).unwrap();
        write!(f, "not valid json {{").unwrap();

        let inputs = vec![InputDefinition::Json(
            oicana_input::input_definition::json::JsonInputDefinition {
                key: "data".to_string(),
                default: Some("default.json".to_string()),
                development: None,
                schema: Some("data.schema.json".to_string()),
                validate: true,
            },
        )];

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not valid JSON"));
    }

    #[test]
    fn no_schema_skips_validation() {
        let dir = tempdir().unwrap();

        let mut f = File::create(dir.path().join("default.json")).unwrap();
        write!(f, "not even json").unwrap();

        let inputs = vec![InputDefinition::Json(
            oicana_input::input_definition::json::JsonInputDefinition {
                key: "data".to_string(),
                default: Some("default.json".to_string()),
                development: None,
                schema: None,
                validate: true,
            },
        )];

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert!(errors.is_empty(), "No schema means no validation");
    }

    #[test]
    fn missing_schema_file_reports_error() {
        let dir = tempdir().unwrap();

        let mut f = File::create(dir.path().join("default.json")).unwrap();
        write!(f, r#"{{"name": "Alice"}}"#).unwrap();

        let inputs = vec![InputDefinition::Json(
            oicana_input::input_definition::json::JsonInputDefinition {
                key: "data".to_string(),
                default: Some("default.json".to_string()),
                development: None,
                schema: Some("missing.schema.json".to_string()),
                validate: true,
            },
        )];

        let errors = validate_json_fallback_values(dir.path(), &inputs);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("failed to read schema file"));
    }
}
