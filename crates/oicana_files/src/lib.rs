//! File abstractions for Oicana, a set of libraries and tools for document templating with Typst.
//!
//! Oicana templates are mostly used in their packaged form as a zip file containing the complete
//! Typst project. For development and testing, they can also be accessed as a directory
//! in a file system, or as a map of Strings.

use typst::diag::FileResult;
use typst::foundations::Bytes;
use typst::syntax::{FileId, Source};

/// Template files in a native file system.
#[cfg(feature = "native")]
pub mod native;
/// Template files in a packed Oicana template.
pub mod packed;
/// Preloaded template files.
pub mod preloaded;

/// Access files in an Oicana template.
///
/// In most scenarios templates are supposed to work without a file system and without
/// internet access. In those cases the implementations of this trait will offer other access to the
/// file contents.
pub trait TemplateFiles: Send + Sync {
    /// Try to access the specified source file.
    fn source(&self, id: FileId) -> FileResult<Source>;

    /// Try to access the specified file.
    fn file(&self, id: FileId) -> FileResult<Bytes>;

    /// Search for all font files in the template
    fn font_files(&self) -> &Vec<FileId>;
}
