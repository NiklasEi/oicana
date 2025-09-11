//! Template abstraction for Oicana, a set of libraries and tools for document templating with Typst.
//!
//! This crate contains a definition for Oicana template manifests and functionality to package
//! a template.
//!
//! Parts of this crate's code are taken from the [Typst package bundler](https://github.com/typst/packages) under its Apache 2.0 License

use crate::manifest::{ManifestValidationError, TemplateManifest};
use log::error;
use oicana_input::input_definition::InputDefinition;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Manifest of an Oicana template.
pub mod manifest;
/// Method to package a template.
pub mod package;

/// Validate a native Oicana template given its path.
pub fn validate_native_template(
    path: impl Into<PathBuf>,
) -> Result<TemplateManifest, TemplateError> {
    let path = path.into();
    if !path.is_dir() {
        error!("Template {path:?} is not a directory!");
        return Err(TemplateError::NotADirectory);
    }
    let template_meta = path.join("typst.toml");
    let manifest = TemplateManifest::from_toml(&read_to_string(template_meta)?)?;
    manifest.validate()?;

    Ok(manifest)
}

/// Errors for reading and validating an Oicana template.
#[derive(Error, Debug)]
pub enum TemplateError {
    /// The given template path is not a directory.
    #[error("The given template path is not a directory")]
    NotADirectory,
    /// The manifest is not valid.
    #[error("Issue in the manifest: {0}")]
    ManifestValidationError(#[from] ManifestValidationError),
    /// The manifest could not be read.
    #[error("Failed to read manifest: {0}")]
    ManifestAccessError(#[from] io::Error),
    /// The manifest could not be parsed.
    #[error("The manifest could not be parsed: {0}")]
    ManifestParsingError(#[from] toml::de::Error),
}

/// The relevant part of the tool section in an Oicana template manifest.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OicanaConfig {
    /// Version of the manifest.
    ///
    /// This will enable compatibility after breaking changes in the manifest.
    pub manifest_version: u8,
    /// The input definitions of the Oicana template.
    #[serde(default = "Vec::new")]
    pub inputs: Vec<InputDefinition>,
    /// path to the tests of the template
    #[serde(default = "default_test_dir")]
    pub tests: PathBuf,
}

fn default_test_dir() -> PathBuf {
    PathBuf::from("tests")
}

#[cfg(test)]
mod tests {
    use crate::{validate_native_template, OicanaConfig, TemplateError};
    use oicana_input::input_definition::blob::{BlobInputDefinition, FallbackBlobInput};
    use oicana_input::input_definition::json::JsonInputDefinition;
    use oicana_input::input_definition::InputDefinition;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use toml::map::Map;
    use toml::Value;

    #[test]
    fn validates_minimal_template() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "invoice"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                manifest_version = 1
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());

        let expected = OicanaConfig {
            manifest_version: 1,
            inputs: vec![],
            tests: PathBuf::from("tests"),
        };
        assert_eq!(result.unwrap().tool.oicana, expected);
    }

    #[test]
    fn validates_maximal_template() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "invoice"
                version = "0.1.0"
                entrypoint = "main.typ"
                authors = ["Oicana <hello@oicana.com>"]
                description = "An example invoice template."

                [tool.oicana]
                manifest_version = 1
                tests = "custom_tests/dir"

                [[tool.oicana.inputs]]
                type = "json"
                key = "invoice"
                default = "invoice.json"
                schema = "invoice.schema.json"

                [[tool.oicana.inputs]]
                type = "blob"
                key = "logo"
                default = {{ file = "logo.jpg", meta = {{ image_format = "jpg", foo = "bar" }} }}

                [[tool.oicana.inputs]]
                type = "json"
                key = "test"
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());

        let expected = OicanaConfig {
            manifest_version: 1,
            tests: PathBuf::from("custom_tests/dir"),
            inputs: vec![
                InputDefinition::Json(JsonInputDefinition {
                    key: "invoice".to_string(),
                    default: Some("invoice.json".to_string()),
                    development: None,
                    schema: Some("invoice.schema.json".to_string()),
                }),
                InputDefinition::Blob(BlobInputDefinition {
                    key: "logo".to_string(),
                    default: Some(FallbackBlobInput {
                        file: "logo.jpg".to_string(),
                        meta: Some(toml::Value::Table({
                            let mut table = Map::default();
                            table.insert("image_format".into(), Value::String("jpg".into()));
                            table.insert("foo".into(), Value::String("bar".into()));

                            table
                        })),
                    }),
                    development: None,
                }),
                InputDefinition::Json(JsonInputDefinition {
                    key: "test".to_string(),
                    default: None,
                    development: None,
                    schema: None,
                }),
            ],
        };
        assert_eq!(result.unwrap().tool.oicana, expected);
    }

    #[test]
    fn manifest_missing_version() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "invoice"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());

        let TemplateError::ManifestParsingError(error) = result.unwrap_err() else {
            panic!("Parsing manifest did not fail with the expected error!")
        };

        assert_eq!(error.message(), "missing field `manifest_version`");
    }

    #[test]
    fn non_existing_path() {
        let template = tempdir().unwrap();
        let path = template.path().join("not_a_directory.txt");

        let result = validate_native_template(path);

        assert!(matches!(result, Err(TemplateError::NotADirectory)));
    }

    #[test]
    fn file_path() {
        let template = tempdir().unwrap();
        let path = template.path().join("not_a_directory.txt");
        {
            let mut file = File::create(&path).unwrap();
            write!(&mut file, "This is not a template!").unwrap();
        }

        let result = validate_native_template(path);

        assert!(matches!(result, Err(TemplateError::NotADirectory)));
    }
}
