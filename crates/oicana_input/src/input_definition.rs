/// Blob input.
pub mod blob;
/// JSON input.
pub mod json;

use crate::InputKind;
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
