/// Blob input.
pub mod blob;
/// JSON input.
pub mod json;

use blob::BlobInputDefinition;
use json::JsonInputDefinition;
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
