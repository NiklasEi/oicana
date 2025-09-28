//! Types and logic for Oicana template tests

use std::{
    fs::{read, read_to_string, File},
    io::{self, Read},
    iter::once,
    path::{Path, PathBuf},
};

use log::{debug, trace};
use oicana_input::{
    input::{
        blob::{Blob, BlobInput},
        json::JsonInput,
    },
    CompilationConfig, CompilationMode, TemplateInputs,
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use typst::foundations::Dict;

/// Methods to collect test cases
pub mod collect;
/// Test execution
pub mod execution;

/// A test case for a template
#[derive(Debug)]
pub struct Test {
    /// Inputs ready for test execution.
    pub inputs: TemplateInputs,
    /// Json inputs to be fuzzed.
    pub fuzzed_inputs: Option<FuzzJson>,
    /// Name of this test.
    pub name: String,
    /// Snapshot file
    pub snapshot: Snapshot,
    /// The collection that contains this Test
    pub collection: PathBuf,
    /// Descriptor of the tests
    pub descriptor: String,
}

/// Test snapshot
#[derive(Debug)]
pub enum Snapshot {
    /// The test has no snapshot and should not have one
    None,
    /// The test has no snapshot, but should have one at the given path
    Missing(PathBuf, SnapshotMode),
    /// The test has a snapshot file at the given path
    Some(PathBuf, SnapshotMode),
}

/// What to do with the test snapshot file
#[derive(Debug, Clone, Copy)]
pub enum SnapshotMode {
    /// Update the snapshot if it doesn't exist or comparison fails
    Update,
    /// Only compare the snapshot file and fail if it doesn't match
    Compare,
}

impl Test {
    /// Create a new test
    pub fn new(
        template_test: TemplateTest,
        collection_name: Option<String>,
        path_components: &[String],
        collection_path: &Path,
        snapshot_mode: SnapshotMode,
        root: &Path,
    ) -> Result<Self, PrepareTestError> {
        let snapshot_path = match template_test.snapshot {
            None => Some(root.join(format!(
                "{}{}.png",
                collection_name
                    .as_ref()
                    .map(|name| format!("{name}."))
                    .unwrap_or("".to_owned()),
                template_test.name
            ))),
            Some(SnapshotConfig::Path(path)) => Some(root.join(path)),
            Some(SnapshotConfig::Disabled) => None,
        };
        let snapshot = match snapshot_path {
            Some(path) => {
                if path.is_file() {
                    Snapshot::Some(path, snapshot_mode)
                } else {
                    Snapshot::Missing(path, snapshot_mode)
                }
            }
            None => Snapshot::None,
        };
        let (inputs, fuzzed_inputs) =
            Self::build_inputs(template_test.mode, template_test.inputs, root)?;
        trace!("Collecting test {}", &template_test.name);

        Ok(Test {
            inputs,
            fuzzed_inputs,
            collection: collection_path.to_path_buf(),
            descriptor: path_components
                .iter()
                .cloned()
                .chain(collection_name)
                .chain(once(template_test.name.clone()))
                .collect::<Vec<String>>()
                .join(" > "),
            name: template_test.name,
            snapshot,
        })
    }

    fn build_inputs(
        mode: CompilationMode,
        input_values: Vec<InputValue>,
        root: &Path,
    ) -> Result<(TemplateInputs, Option<FuzzJson>), PrepareTestError> {
        let mut inputs = TemplateInputs::new();
        let mut fuzzed_input = None;

        inputs.with_config(CompilationConfig::new(mode));

        for input in input_values {
            match input {
                InputValue::Json(json_input) => match json_input {
                    JsonInputValue {
                        key,
                        value: JsonTestValue::File { file },
                    } => {
                        let file_path = root.join(file);
                        let value = read_to_string(&file_path)
                            .map_err(|source| PrepareTestError::Io { file_path, source })?;
                        inputs.with_input(JsonInput::new(key, value));
                    }
                    JsonInputValue {
                        key,
                        value: JsonTestValue::Fuzz { samples },
                    } => {
                        if fuzzed_input.is_some() {
                            return Err(PrepareTestError::OnlyOneFuzzedInput);
                        }
                        let _ = fuzzed_input.insert(FuzzJson { samples, key });
                    }
                },
                InputValue::Blob(blob) => {
                    let file_path = root.join(blob.file);
                    let value = read(&file_path)
                        .map_err(|source| PrepareTestError::Io { file_path, source })?;
                    let mut blob_value: Blob = value.into();
                    blob_value.metadata = match blob.meta {
                        Some(meta) => Deserialize::deserialize(meta)?,
                        None => Dict::new(),
                    };
                    inputs.with_input(BlobInput::new(blob.key, blob_value));
                }
            }
        }

        Ok((inputs, fuzzed_input))
    }
}

/// Json input to be fuzzed as part of the test execution.
#[derive(Debug)]
pub struct FuzzJson {
    samples: usize,
    key: String,
}

/// Error cases while preparing a test case for execution
#[derive(Debug, Error)]
pub enum PrepareTestError {
    /// Error opening or reading file
    #[error("Failed to read file '{file_path}': {source}")]
    Io {
        /// The path of the file causing the error
        file_path: PathBuf,
        /// The io error
        #[source]
        source: io::Error,
    },
    /// The test does not have a snapshot file
    #[error("Failed to find snapshot file at '{0}'")]
    NoSnapshot(PathBuf),
    /// Only one input can be fuzzed per test
    #[error("Only one input can be fuzzed per test")]
    OnlyOneFuzzedInput,
    /// Failed to convert metadata to Typst dict
    #[error("Failed to convert metadata to Typst dictionary '{0}'")]
    FailedToConvertMetadata(#[from] toml::de::Error),
}

/// A collection of test definitions
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplateTestCollection {
    /// Version of the test collection manifest.
    ///
    /// This will allow changes in the manifest.
    pub tests_version: u8,
    /// Name of the test collection.
    pub name: Option<String>,
    /// The tests defined in this collection.
    #[serde(default = "Vec::new", rename = "test")]
    pub tests: Vec<TemplateTest>,
}

impl TemplateTestCollection {
    fn read_from(path: &Path) -> Result<Self, TestCollectionError> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        debug!("Parsing {:?} to template test collection.", path);
        let mut collection = toml::de::from_str::<TemplateTestCollection>(&content)?;

        if collection.name.is_none() {
            // use name from the file if there is one
            let file_name = path
                .file_name()
                .map(|file_name| {
                    file_name
                        .to_string_lossy()
                        .strip_suffix(".tests.toml")
                        .unwrap_or("")
                        .trim()
                        .to_owned()
                })
                .unwrap_or_default();
            if !file_name.is_empty() {
                collection.name = Some(file_name);
            }
        }

        Ok(collection)
    }
}

/// Errors that can be produced when collection test cases
#[derive(Debug, Error)]
pub enum TestCollectionError {
    /// Error opening or reading file
    #[error("Failed to read file")]
    IoError(#[from] io::Error),
    /// Failed to parse the test collection file as such
    #[error("Failed to parse test collection")]
    ParsingError(#[from] toml::de::Error),
}

/// A template test
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplateTest {
    /// Name of the test
    ///
    /// The name has to be unique for the template under test.
    pub name: String,
    /// Relative path to snapshot file for this test
    #[serde(deserialize_with = "snapshot_config", default = "none")]
    pub snapshot: Option<SnapshotConfig>,
    /// The input values for this test.
    #[serde(default = "Vec::new")]
    pub inputs: Vec<InputValue>,
    /// The input values for this test.
    #[serde(default = "production")]
    pub mode: CompilationMode,
}

fn none() -> Option<SnapshotConfig> {
    None
}

fn snapshot_config<'de, D>(deserializer: D) -> Result<Option<SnapshotConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Bool(false) => Ok(Some(SnapshotConfig::Disabled)),
        serde_json::Value::String(s) => Ok(Some(SnapshotConfig::Path(s))),
        other => Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Other(&format!("{:?}", other)),
            &"a boolean false or a string",
        )),
    }
}

/// Configure or disable snapshot file comparison for the test
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged, deny_unknown_fields)]
pub enum SnapshotConfig {
    /// This test should onlz be compiled
    /// there will be no comparison of the output with a snapshot file
    Disabled,
    /// Relative path to the snapshot file for this test
    Path(String),
}

fn production() -> CompilationMode {
    CompilationMode::Production
}

/// An input value for an Oicana template.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum InputValue {
    /// An input for JSON values.
    #[serde(rename = "json")]
    Json(JsonInputValue),
    /// An input for blob values.
    ///
    /// Commonly this is used for image files or files that should be embedded into the document.
    #[serde(rename = "blob")]
    Blob(BlobInputValue),
}

/// An input with JSON values.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct JsonInputValue {
    /// The key of the input.
    ///
    /// Use this in the Typst code to refer to the current value of the input.
    pub key: String,
    /// Path to the file containing the JSON.
    #[serde(flatten)]
    pub value: JsonTestValue,
}

/// Values for json inputs in tests
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged, deny_unknown_fields)]
pub enum JsonTestValue {
    /// the input value should be fuzzed based on the json schema
    Fuzz {
        /// Number of fuzzing samples for this input
        samples: usize,
    },
    /// The input value is a json file
    File {
        /// Relative path to the file that is the value of this input for the test
        file: String,
    },
}

/// A blob input.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BlobInputValue {
    /// The key of the input.
    pub key: String,
    /// Path to the file.
    pub file: String,
    /// Meta information for this input value.
    pub meta: Option<toml::Value>,
}

#[cfg(test)]
mod tests {
    use crate::TemplateTestCollection;
    use oicana_input::CompilationMode;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn takes_name_from_file_content() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("bar.tests.toml");
        let mut file = File::create(&path).unwrap();
        write!(
            &mut file,
            r#"
            name = "foo"
            tests_version = 1
            "#
        )
        .unwrap();

        let test_collection = TemplateTestCollection::read_from(&path)
            .expect("Failed to read test collection from file");
        assert_eq!(test_collection.name, Some(String::from("foo")));
    }

    #[test]
    fn takes_name_from_file_name() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("bar.tests.toml");
        let mut file = File::create(&path).unwrap();
        write!(
            &mut file,
            r#"
            tests_version = 1
            "#
        )
        .unwrap();

        let test_collection = TemplateTestCollection::read_from(&path)
            .expect("Failed to read test collection from file");
        assert_eq!(test_collection.name, Some(String::from("bar")));
    }

    #[test]
    fn no_name() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("tests.toml");
        let mut file = File::create(&path).unwrap();
        write!(
            &mut file,
            r#"
            tests_version = 1
            "#
        )
        .unwrap();

        let test_collection = TemplateTestCollection::read_from(&path)
            .expect("Failed to read test collection from file");
        assert_eq!(test_collection.name, None);
    }

    #[test]
    fn mode() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("tests.toml");
        let mut file = File::create(&path).unwrap();
        write!(
            &mut file,
            r#"
                tests_version = 1

                [[test]]
                name = "test"
                mode = "development"
                "#
        )
        .unwrap();

        let test_collection = TemplateTestCollection::read_from(&path)
            .expect("Failed to read test collection from file");
        assert_eq!(test_collection.tests[0].mode, CompilationMode::Development);
    }

    #[test]
    fn default_mode() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("tests.toml");
        let mut file = File::create(&path).unwrap();
        write!(
            &mut file,
            r#"
                        tests_version = 1

                        [[test]]
                        name = "test"
                        "#
        )
        .unwrap();

        let test_collection = TemplateTestCollection::read_from(&path)
            .expect("Failed to read test collection from file");
        assert_eq!(test_collection.tests[0].mode, CompilationMode::Production);
    }

    #[test]
    fn short_mode() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("tests.toml");
        let mut file = File::create(&path).unwrap();
        write!(
            &mut file,
            r#"
                        tests_version = 1

                        [[test]]
                        name = "test"
                        mode = "dev"
                        "#
        )
        .unwrap();

        let test_collection = TemplateTestCollection::read_from(&path)
            .expect("Failed to read test collection from file");
        assert_eq!(test_collection.tests[0].mode, CompilationMode::Development);
    }
}
