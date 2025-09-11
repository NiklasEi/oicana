use serde::{Deserialize, Serialize};

/// A blob input that can be defined in an Oicana template manifest.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlobInputDefinition {
    /// The key of the input.
    ///
    /// Use this in the Typst code to refer to the current value of the data set.
    pub key: String,
    /// Default value of this input in case no other value is supplied.
    ///
    /// In development mode, [`Self::development`] is preferred.
    pub default: Option<FallbackBlobInput>,
    /// Value for this input in development mode.
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
