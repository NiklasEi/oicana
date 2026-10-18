//! The Typst World wrapper for Oicana

use std::fmt::Display;

use diagnostics::Diagnostic;
use thiserror::Error;
use typst_layout::PagedDocument;

/// Diagnostics for the Typst World with codespan-reporting
pub mod diagnostics;
/// Fonts available to a template and the ones a host can provide
pub mod fonts;
/// Get the manifest of an Oicana World
pub mod manifest;
/// Oicana implementation of a Typst World
pub mod world;

pub use world::evict_cache;
pub use world::{
    InputError, InputMismatchError, InputValidationError, InvalidJsonInput, WrongInputKind,
};

/// A successfully compiled document with warning diagnostics.
pub struct CompiledDocument {
    /// The compiled document.
    pub document: PagedDocument,
    /// Warnings from the compilation.
    pub warnings: Vec<Diagnostic>,
}

/// Error while compiling a template.
#[derive(Error, Debug)]
pub struct TemplateCompilationFailure {
    /// Error message that failed the compilation
    pub error: String,
    /// Warnings from the template compilation
    pub warnings: Vec<Diagnostic>,
}

impl Display for TemplateCompilationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.error)?;
        for warning in &self.warnings {
            writeln!(f, "\n{warning}")?;
        }
        Ok(())
    }
}

/// Get the current timestamp in milliseconds.
pub fn get_current_time() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        return js_sys::Date::now();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Utc::now().timestamp_millis() as f64
    }
}
