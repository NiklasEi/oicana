use std::collections::HashMap;

use oicana_ffi_core::BlobWithMetadata;

/// How required inputs without a value are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum CompilationMode {
    /// Fall back to the development value, then the default value.
    Development,
    /// Fall back to the default value.
    Production,
}

impl From<CompilationMode> for oicana_ffi_core::CompilationMode {
    fn from(mode: CompilationMode) -> Self {
        match mode {
            CompilationMode::Development => oicana_ffi_core::CompilationMode::Development,
            CompilationMode::Production => oicana_ffi_core::CompilationMode::Production,
        }
    }
}

/// Coloring of compilation diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DiagnosticColor {
    /// Plain text.
    None,
    /// ANSI escape codes.
    Ansi,
}

impl From<DiagnosticColor> for oicana_ffi_core::DiagnosticColor {
    fn from(color: DiagnosticColor) -> Self {
        match color {
            DiagnosticColor::None => oicana_ffi_core::DiagnosticColor::None,
            DiagnosticColor::Ansi => oicana_ffi_core::DiagnosticColor::Ansi,
        }
    }
}

/// Format of an exported document.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum ExportFormat {
    /// PDF, using the standards from the template manifest.
    Pdf,
    /// PNG image.
    Png {
        /// Pixels per typographic point.
        pixels_per_pt: f32,
    },
    /// SVG image.
    Svg,
}

impl From<ExportFormat> for oicana_ffi_core::ExportFormat {
    fn from(format: ExportFormat) -> Self {
        match format {
            ExportFormat::Pdf => oicana_ffi_core::ExportFormat::Pdf,
            ExportFormat::Png { pixels_per_pt } => {
                oicana_ffi_core::ExportFormat::Png { pixels_per_pt }
            }
            ExportFormat::Svg => oicana_ffi_core::ExportFormat::Svg,
        }
    }
}

/// A contiguous range of pages, with 0-based inclusive bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct PageRange {
    /// First page to export. `None` starts at the first page.
    #[uniffi(default = None)]
    pub start: Option<u64>,
    /// Last page to export. `None` ends at the last page.
    #[uniffi(default = None)]
    pub end: Option<u64>,
}

impl From<PageRange> for oicana_ffi_core::PageRange {
    fn from(range: PageRange) -> Self {
        oicana_ffi_core::PageRange {
            start: range.start.map(saturating_usize),
            end: range.end.map(saturating_usize),
        }
    }
}

/// Limits for reading a packed template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct ZipLimits {
    /// Maximum number of zip entries. `None` keeps the default.
    #[uniffi(default = None)]
    pub max_entries: Option<u64>,
    /// Maximum total decompressed size in bytes. `None` keeps the default.
    #[uniffi(default = None)]
    pub max_total_decompressed_bytes: Option<u64>,
}

impl From<ZipLimits> for Option<oicana_ffi_core::ZipLimits> {
    fn from(limits: ZipLimits) -> Self {
        oicana_ffi_core::ZipLimits::from_optional(
            limits.max_entries,
            limits.max_total_decompressed_bytes,
        )
    }
}

/// A blob input value.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct BlobInput {
    /// Raw bytes of the blob.
    pub data: Vec<u8>,
    /// Metadata of the blob as a JSON object.
    #[uniffi(default = "{}")]
    pub metadata: String,
}

/// Size of a page in typographic points.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct PageSize {
    /// Page width.
    pub width: f64,
    /// Page height.
    pub height: f64,
}

impl From<oicana_ffi_core::PageSize> for PageSize {
    fn from(size: oicana_ffi_core::PageSize) -> Self {
        PageSize {
            width: size.width,
            height: size.height,
        }
    }
}

/// A font face registered by the host.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RegisteredFont {
    /// The family name.
    pub family: String,
    /// The file the face was read from, if it was registered by path.
    pub path: Option<String>,
}

impl From<oicana_ffi_core::RegisteredFont> for RegisteredFont {
    fn from(font: oicana_ffi_core::RegisteredFont) -> Self {
        RegisteredFont {
            family: font.family,
            path: font.path,
        }
    }
}

/// An exported document with the warnings of its compilation.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ExportOnceResult {
    /// The exported document.
    pub bytes: Vec<u8>,
    /// Compilation warnings.
    pub warnings: Vec<Diagnostic>,
}

/// A compilation diagnostic, like a warning, with its location in the template.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Diagnostic {
    /// Whether this is an error or a warning.
    pub severity: DiagnosticSeverity,
    /// The message describing the problem.
    pub message: String,
    /// Where in the template the problem is, if it points into a file.
    pub location: Option<SourceLocation>,
    /// Hints on how to address the problem.
    pub hints: Vec<String>,
    /// The function calls leading to the problem, innermost first.
    pub trace: Vec<TracePoint>,
    /// The diagnostic formatted for humans, with source excerpts and the configured
    /// diagnostic color.
    pub formatted: String,
}

impl From<oicana_ffi_core::Diagnostic> for Diagnostic {
    fn from(diagnostic: oicana_ffi_core::Diagnostic) -> Self {
        Diagnostic {
            severity: diagnostic.severity.into(),
            message: diagnostic.message,
            location: diagnostic.location.map(Into::into),
            hints: diagnostic.hints,
            trace: diagnostic.trace.into_iter().map(Into::into).collect(),
            formatted: diagnostic.formatted,
        }
    }
}

pub(crate) fn diagnostics(diagnostics: Vec<oicana_ffi_core::Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.into_iter().map(Into::into).collect()
}

/// The severity of a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DiagnosticSeverity {
    /// A fatal error.
    Error,
    /// A non-fatal warning.
    Warning,
}

impl From<oicana_ffi_core::DiagnosticSeverity> for DiagnosticSeverity {
    fn from(severity: oicana_ffi_core::DiagnosticSeverity) -> Self {
        match severity {
            oicana_ffi_core::DiagnosticSeverity::Error => DiagnosticSeverity::Error,
            oicana_ffi_core::DiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
        }
    }
}

/// A range in a file of the template.
///
/// Lines and columns are 1-based. Columns count characters.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SourceLocation {
    /// Path of the file in the template, like `/main.typ`.
    /// Files of packages are prefixed with the package, like `@preview/example:0.1.0/lib.typ`.
    pub file: String,
    /// Line of the start of the range.
    pub line: u32,
    /// Column of the start of the range.
    pub column: u32,
    /// Line of the end of the range.
    pub end_line: u32,
    /// Column of the end of the range (exclusive).
    pub end_column: u32,
}

impl From<oicana_ffi_core::SourceLocation> for SourceLocation {
    fn from(location: oicana_ffi_core::SourceLocation) -> Self {
        SourceLocation {
            file: location.file,
            line: saturating_u32(location.line),
            column: saturating_u32(location.column),
            end_line: saturating_u32(location.end_line),
            end_column: saturating_u32(location.end_column),
        }
    }
}

/// A step in the function calls leading to a [`Diagnostic`].
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct TracePoint {
    /// What happened at this step, like `error occurred in this function call`.
    pub message: String,
    /// Where the step happened, if it points into a file.
    pub location: Option<SourceLocation>,
}

impl From<oicana_ffi_core::TracePoint> for TracePoint {
    fn from(point: oicana_ffi_core::TracePoint) -> Self {
        TracePoint {
            message: point.message,
            location: point.location.map(Into::into),
        }
    }
}

pub(crate) fn core_blobs(blobs: HashMap<String, BlobInput>) -> HashMap<String, BlobWithMetadata> {
    blobs
        .into_iter()
        .map(|(key, blob)| {
            (
                key,
                BlobWithMetadata {
                    bytes: blob.data,
                    meta: blob.metadata,
                },
            )
        })
        .collect()
}

pub(crate) fn saturating_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn saturating_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
