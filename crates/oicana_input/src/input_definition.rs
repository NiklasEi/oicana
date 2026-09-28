use crate::InputKind;
use serde::{Deserialize, Serialize};
use typst::syntax::package::UnknownFields;

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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    /// All parsed but unknown fields
    #[serde(flatten, skip_serializing)]
    pub unknown_fields: UnknownFields,
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
    /// All parsed but unknown fields
    #[serde(flatten, skip_serializing)]
    pub unknown_fields: UnknownFields,
}

/// Default value of a blob input.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FallbackBlobInput {
    /// Path to a file to use as the default blob value.
    pub file: String,
    /// Meta information of the default blob.
    pub meta: Option<toml::Value>,
    /// All parsed but unknown fields
    #[serde(flatten, skip_serializing)]
    pub unknown_fields: UnknownFields,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::InputDefinition;

    #[derive(serde::Deserialize)]
    struct Inputs {
        inputs: Vec<InputDefinition>,
    }

    #[test]
    fn unknown_fields_are_collected() {
        let Inputs { inputs } = toml::from_str(
            r#"
[[inputs]]
type = "json"
key = "data"
dev = "dev.json"

[[inputs]]
type = "blob"
key = "logo"
mandatory = false
default = { file = "logo.png", metadata = { image_format = "png" } }
"#,
        )
        .unwrap();

        let [InputDefinition::Json(json), InputDefinition::Blob(blob)] = &inputs[..] else {
            panic!("expected a json and a blob input, got {inputs:?}");
        };
        assert_eq!(
            json.unknown_fields.keys().collect::<Vec<_>>(),
            ["dev"]
        );
        assert_eq!(blob.unknown_fields.keys().collect::<Vec<_>>(), ["mandatory"]);
        assert_eq!(
            blob.default
                .as_ref()
                .unwrap()
                .unknown_fields
                .keys()
                .collect::<Vec<_>>(),
            ["metadata"]
        );
    }
}
