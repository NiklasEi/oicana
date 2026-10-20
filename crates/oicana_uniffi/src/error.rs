use oicana_ffi_core::FfiError;

/// Errors raised by Oicana.
///
/// Each variant carries the human-readable message, including rendered compilation
/// diagnostics.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[expect(
    missing_docs,
    reason = "the message field of every variant is described in the enum docs"
)]
pub enum OicanaError {
    /// The packed template could not be read or its manifest is invalid.
    #[error("{message}")]
    InvalidTemplate { message: String },
    /// An input value, its blob metadata, or a page range is invalid.
    #[error("{message}")]
    InvalidInput { message: String },
    /// Compiling the template failed.
    #[error("{message}")]
    Compilation { message: String },
    /// A file requested from the template could not be loaded.
    #[error("{message}")]
    FileNotFound { message: String },
    /// Encoding the document to the requested format failed.
    #[error("{message}")]
    Export { message: String },
    /// A template or document is no longer registered.
    #[error("{message}")]
    NotFound { message: String },
    /// An unexpected internal failure.
    #[error("{message}")]
    Internal { message: String },
}

impl From<FfiError> for OicanaError {
    fn from(error: FfiError) -> Self {
        let message = error.to_string();
        match error {
            FfiError::PackedTemplate(_) | FfiError::Manifest(_) | FfiError::WorldCreation(_) => {
                OicanaError::InvalidTemplate { message }
            }
            FfiError::InputValidation(_)
            | FfiError::BlobMetadata { .. }
            | FfiError::PageRangeParse(_) => OicanaError::InvalidInput { message },
            FfiError::Compilation(_) | FfiError::WarmUp(_) => OicanaError::Compilation { message },
            FfiError::SourceLoad { .. } | FfiError::FileLoad { .. } => {
                OicanaError::FileNotFound { message }
            }
            FfiError::Export { .. } => OicanaError::Export { message },
            FfiError::TemplateNotRegistered(_)
            | FfiError::DocumentNotFound(_)
            | FfiError::InvalidDocumentId(_) => OicanaError::NotFound { message },
            FfiError::ManifestSerialization(_)
            | FfiError::PageSizesSerialization(_)
            | FfiError::ExportFormatParse(_) => OicanaError::Internal { message },
        }
    }
}
