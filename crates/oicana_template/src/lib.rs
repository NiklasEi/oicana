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
    /// Whether to validate JSON inputs against their schemas by default.
    ///
    /// When `true` (the default), JSON inputs that have a schema defined will be
    /// validated before compilation. Set to `false` to disable validation for all
    /// inputs by default. This can be overridden at runtime per template instance.
    /// Individual inputs can also opt out via their own `validate` property.
    #[serde(default = "default_true")]
    pub validate_json_inputs_by_default: bool,
    /// path to the tests of the template
    #[serde(default = "default_test_dir")]
    pub tests: PathBuf,
    /// Export configuration for the template.
    #[serde(default)]
    pub export: ExportConfig,
}

fn default_true() -> bool {
    true
}

fn default_test_dir() -> PathBuf {
    PathBuf::from("tests")
}

/// Configuration for exporting compiled documents.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ExportConfig {
    /// PDF export configuration.
    #[serde(default)]
    pub pdf: PdfExportConfig,
}

/// A PDF standard that Typst can enforce conformance with.
///
/// Several standards can be combined as long as the combination is compatible:
/// at most one base PDF version, at most one PDF/A standard and at most one
/// PDF/UA standard, with overlapping PDF versions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
#[allow(non_camel_case_types)]
#[serde(rename_all = "kebab-case")]
pub enum PdfStandard {
    /// PDF 1.4.
    #[serde(rename = "1.4")]
    V_1_4,
    /// PDF 1.5.
    #[serde(rename = "1.5")]
    V_1_5,
    /// PDF 1.6.
    #[serde(rename = "1.6")]
    V_1_6,
    /// PDF 1.7.
    #[serde(rename = "1.7")]
    V_1_7,
    /// PDF 2.0.
    #[serde(rename = "2.0")]
    V_2_0,
    /// PDF/A-1b.
    #[serde(rename = "a-1b")]
    A_1b,
    /// PDF/A-1a.
    #[serde(rename = "a-1a")]
    A_1a,
    /// PDF/A-2b.
    #[serde(rename = "a-2b")]
    A_2b,
    /// PDF/A-2u.
    #[serde(rename = "a-2u")]
    A_2u,
    /// PDF/A-2a.
    #[serde(rename = "a-2a")]
    A_2a,
    /// PDF/A-3b.
    #[serde(rename = "a-3b")]
    A_3b,
    /// PDF/A-3u.
    #[serde(rename = "a-3u")]
    A_3u,
    /// PDF/A-3a.
    #[serde(rename = "a-3a")]
    A_3a,
    /// PDF/A-4.
    #[serde(rename = "a-4")]
    A_4,
    /// PDF/A-4f.
    #[serde(rename = "a-4f")]
    A_4f,
    /// PDF/A-4e.
    #[serde(rename = "a-4e")]
    A_4e,
    /// PDF/UA-1.
    #[serde(rename = "ua-1")]
    Ua_1,
}

/// Configuration for PDF export.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PdfExportConfig {
    /// The PDF standards to enforce during export.
    ///
    /// Several standards can be combined as long as the combination is
    /// compatible: at most one base PDF version, at most one PDF/A standard
    /// and at most one PDF/UA standard, with overlapping PDF versions.
    ///
    /// PDF/A standards are geared towards archival use and maximum compatibility
    /// with current and future PDF tooling. PDF/UA standards ensure universal
    /// accessibility.
    ///
    /// Defaults to `["a-3b"]` (PDF/A-3b) if not specified.
    #[serde(default = "default_pdf_standards")]
    pub standards: Vec<PdfStandard>,
    /// Whether to produce a tagged (accessible) PDF.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_true")]
    pub tagged: bool,
}

impl Default for PdfExportConfig {
    fn default() -> Self {
        Self {
            standards: default_pdf_standards(),
            tagged: default_true(),
        }
    }
}

fn default_pdf_standards() -> Vec<PdfStandard> {
    vec![PdfStandard::A_3b]
}

#[cfg(test)]
mod tests {
    use crate::{validate_native_template, ExportConfig, OicanaConfig, PdfStandard, TemplateError};
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
            validate_json_inputs_by_default: true,
            tests: PathBuf::from("tests"),
            export: ExportConfig::default(),
        };
        assert_eq!(result.unwrap().tool.oicana, expected);
    }

    #[test]
    fn parses_custom_pdf_standards() {
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

                [tool.oicana.export.pdf]
                standards = ["2.0", "a-4"]
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());
        let config = result.unwrap().tool.oicana;

        assert_eq!(
            config.export.pdf.standards,
            vec![PdfStandard::V_2_0, PdfStandard::A_4]
        );
    }

    #[test]
    fn parses_pdf_tagged_false() {
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

                [tool.oicana.export.pdf]
                tagged = false
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());
        let config = result.unwrap().tool.oicana;

        assert!(!config.export.pdf.tagged);
    }

    #[test]
    fn defaults_pdf_tagged_to_true() {
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
        let config = result.unwrap().tool.oicana;

        assert!(config.export.pdf.tagged);
    }

    #[test]
    fn defaults_pdf_standards_to_a3b() {
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
        let config = result.unwrap().tool.oicana;

        assert_eq!(config.export.pdf.standards, vec![PdfStandard::A_3b]);
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
                    required: true,
                    default: Some("invoice.json".to_string()),
                    development: None,
                    schema: Some("invoice.schema.json".to_string()),
                    validate: true,
                }),
                InputDefinition::Blob(BlobInputDefinition {
                    key: "logo".to_string(),
                    required: true,
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
                    required: true,
                    default: None,
                    development: None,
                    schema: None,
                    validate: true,
                }),
            ],
            validate_json_inputs_by_default: true,
            export: ExportConfig::default(),
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

    #[test]
    fn validate_json_inputs_by_default_defaults_to_true() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "test"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                manifest_version = 1
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());
        assert!(result.unwrap().tool.oicana.validate_json_inputs_by_default);
    }

    #[test]
    fn validate_json_inputs_by_default_can_be_set_to_false() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "test"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                manifest_version = 1
                validate_json_inputs_by_default = false
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());
        assert!(!result.unwrap().tool.oicana.validate_json_inputs_by_default);
    }

    #[test]
    fn json_input_validate_defaults_to_true() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "test"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                manifest_version = 1

                [[tool.oicana.inputs]]
                type = "json"
                key = "data"
                schema = "data.schema.json"
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());
        let config = result.unwrap().tool.oicana;
        let InputDefinition::Json(json_def) = &config.inputs[0] else {
            panic!("Expected JSON input");
        };
        assert!(json_def.validate);
    }

    #[test]
    fn json_input_validate_can_be_set_to_false() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "test"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                manifest_version = 1

                [[tool.oicana.inputs]]
                type = "json"
                key = "data"
                schema = "data.schema.json"
                validate = false
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());
        let config = result.unwrap().tool.oicana;
        let InputDefinition::Json(json_def) = &config.inputs[0] else {
            panic!("Expected JSON input");
        };
        assert!(!json_def.validate);
    }

    #[test]
    fn json_input_validate_true_without_schema_is_valid() {
        let template = tempdir().unwrap();
        {
            let path = template.path().join("typst.toml");
            let mut file = File::create(&path).unwrap();
            write!(
                &mut file,
                r#"
                [package]
                name = "test"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                manifest_version = 1

                [[tool.oicana.inputs]]
                type = "json"
                key = "data"
                validate = true
                "#
            )
            .unwrap();
        }

        let result = validate_native_template(template.path());
        let config = result.unwrap().tool.oicana;
        let InputDefinition::Json(json_def) = &config.inputs[0] else {
            panic!("Expected JSON input");
        };
        assert!(json_def.validate);
        assert!(json_def.schema.is_none());
    }
}
