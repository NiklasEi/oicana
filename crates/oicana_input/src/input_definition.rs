use crate::InputKind;
use serde::{Deserialize, Serialize};

/// Oicana template inputs that can be defined in the manifest.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum InputDefinition {
    /// An input for JSON values.
    #[serde(rename = "json")]
    Json(JsonInputDefinition),
    /// An input for blob values.
    ///
    /// Commonly this is used for image files or files that should be embedded into the document.
    #[serde(rename = "blob")]
    Blob(BlobInputDefinition),
}

impl InputDefinition {
    /// The key identifying this input.
    pub fn key(&self) -> &str {
        match self {
            InputDefinition::Json(def) => &def.key,
            InputDefinition::Blob(def) => &def.key,
        }
    }

    /// The kind of value this input expects.
    pub fn kind(&self) -> InputKind {
        match self {
            InputDefinition::Json(_) => InputKind::Json,
            InputDefinition::Blob(_) => InputKind::Blob,
        }
    }

    /// Whether this input is required.
    pub fn required(&self) -> bool {
        match self {
            InputDefinition::Json(def) => def.required,
            InputDefinition::Blob(def) => def.required,
        }
    }
}

/// An input for JSON values.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct JsonInputDefinition {
    /// The key of the input.
    ///
    /// Use this in the Typst code to refer to the current value of the input.
    pub key: String,
    /// Whether this input must have a value when compiling the template.
    ///
    /// Defaults to `true`. When `true`, compilation will fail if no value
    /// is supplied and no default or development value is configured.
    #[serde(default = "default_true")]
    pub required: bool,
    /// Path to a file used as default value for this input in case no other value is supplied.
    ///
    /// During development, the value of [`Self::development`] is preferred.
    pub default: Option<String>,
    /// Path to a file used as input value during development, when no value is supplied.
    pub development: Option<String>,
    /// Path to a JSON schema to validate input against.
    pub schema: Option<String>,
    /// Whether to validate this input against its schema.
    ///
    /// Defaults to `true`. Set to `false` to skip validation for this input
    /// even if a schema is defined. When `false`, no validator is compiled
    /// for this input during template initialization.
    #[serde(default = "default_true")]
    pub validate: bool,
}

/// A blob input that can be defined in an Oicana template manifest.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlobInputDefinition {
    /// The key of the input.
    ///
    /// Use this in the Typst code to refer to the current value of the data set.
    pub key: String,
    /// Whether this input must have a value when compiling the template.
    ///
    /// Defaults to `true`. When `true`, compilation will fail if no value
    /// is supplied and no default or development value is configured.
    #[serde(default = "default_true")]
    pub required: bool,
    /// Default value of this input in case no other value is supplied.
    ///
    /// In development mode, [`Self::development`] is preferred.
    pub default: Option<FallbackBlobInput>,
    /// Value for this input in development mode, when no value is supplied.
    pub development: Option<FallbackBlobInput>,
}

/// Default value of a blob input.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FallbackBlobInput {
    /// Path to a file to use as the default blob value.
    pub file: String,
    /// Meta information of the default blob.
    pub meta: Option<toml::Value>,
}

fn default_true() -> bool {
    true
}
