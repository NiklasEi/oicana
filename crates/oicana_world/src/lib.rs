//! The Typst World wrapper for Oicana

use std::fmt::Display;

use thiserror::Error;
use typst::layout::PagedDocument;

/// Diagnostics for the Typst World with codespan-reporting
pub mod diagnostics;
mod fonts;
/// Get the manifest of an Oicana World
pub mod manifest;
/// Oicana implementation of a Typst World
pub mod world;

/// A successfully compiled document with warning diagnostics.
pub struct CompiledDocument {
    /// The compiled document.
    pub document: PagedDocument,
    /// warnings from the compilation.
    pub warnings: Option<String>,
}

/// Error while compiling a template.
#[derive(Error, Debug)]
pub struct TemplateCompilationFailure {
    /// Error message that failed the compilation
    error: String,
    /// Warning messages from the template compilation
    warnings: Option<String>,
}

impl Display for TemplateCompilationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.error)?;
        if let Some(ref warnings) = self.warnings {
            writeln!(f, "{warnings}")?;
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
