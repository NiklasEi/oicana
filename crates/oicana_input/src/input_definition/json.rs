use serde::{Deserialize, Serialize};

/// An input for JSON values.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct JsonInputDefinition {
    /// The key of the input.
    ///
    /// Use this in the Typst code to refer to the current value of the input.
    pub key: String,
    /// Path to a file used as default value for this input in case no other value is supplied.
    ///
    /// During development, the value of [`Self::development`] is preferred.
    pub default: Option<String>,
    /// Path to a file used as input value during development.
    pub development: Option<String>,
    /// Path to a JSON schema to validate input against.
    pub schema: Option<String>,
}
