use codespan_reporting::{
    diagnostic::{Diagnostic as CodespanDiagnostic, Label},
    files::{Error as CodespanError, Files as CodespanFiles},
    term::{
        self,
        termcolor::{Ansi, NoColor, WriteColor},
    },
};
use ecow::EcoVec;
use oicana_files::TemplateFiles;
use serde::{Deserialize, Serialize};
use typst::{
    diag::{Severity, SourceDiagnostic},
    syntax::{DiagSpan, FileId, Source, VirtualRoot},
    WorldExt,
};

use crate::world::OicanaWorld;

/// Display name of a file: its path in the template, prefixed with the package for package files.
fn file_name(id: FileId) -> String {
    let rooted = id.vpath().get_with_slash();
    match id.root() {
        VirtualRoot::Project => rooted.to_string(),
        VirtualRoot::Package(package) => format!("{package}{rooted}"),
    }
}

impl<'a, Files: TemplateFiles> CodespanFiles<'a> for OicanaWorld<Files> {
    type FileId = FileId;
    type Name = String;
    type Source = Source;

    fn name(&'a self, id: FileId) -> Result<Self::Name, CodespanError> {
        Ok(file_name(id))
    }

    fn source(&'a self, id: FileId) -> Result<Self::Source, CodespanError> {
        self.files
            .source(id)
            .map_err(|_| CodespanError::FileMissing)
    }

    fn line_index(&'a self, id: FileId, given: usize) -> Result<usize, CodespanError> {
        let source = self
            .files
            .source(id)
            .map_err(|_| CodespanError::FileMissing)?;
        let source = source.lines();
        source
            .byte_to_line(given)
            .ok_or_else(|| CodespanError::IndexTooLarge {
                given,
                max: source.len_bytes(),
            })
    }

    fn line_range(
        &'a self,
        id: FileId,
        given: usize,
    ) -> Result<std::ops::Range<usize>, CodespanError> {
        let source = self
            .files
            .source(id)
            .map_err(|_| CodespanError::FileMissing)?;
        let source = source.lines();
        source
            .line_to_range(given)
            .ok_or_else(|| CodespanError::LineTooLarge {
                given,
                max: source.len_lines(),
            })
    }

    fn column_number(&'a self, id: FileId, _: usize, given: usize) -> Result<usize, CodespanError> {
        let source = self
            .files
            .source(id)
            .map_err(|_| CodespanError::FileMissing)?;
        let source = source.lines();
        source.byte_to_column(given).ok_or_else(|| {
            let max = source.len_bytes();
            if given <= max {
                CodespanError::InvalidCharBoundary { given }
            } else {
                CodespanError::IndexTooLarge { given, max }
            }
        })
    }
}

/// Format Typst source diagnostics
pub trait TemplateDiagnostics {
    /// Convert source diagnostics to readable error and warning messages
    fn format_diagnostics(&self, diagnostics: EcoVec<SourceDiagnostic>) -> Vec<u8>;
}

impl<Files: TemplateFiles> OicanaWorld<Files> {
    /// Create a label for a span.
    fn label(&self, span: impl Into<DiagSpan>) -> Option<Label<FileId>> {
        let span = span.into();
        Some(Label::primary(span.id()?, self.range(span)?))
    }
}

impl<Files: TemplateFiles> TemplateDiagnostics for OicanaWorld<Files> {
    fn format_diagnostics(&self, diagnostics: EcoVec<SourceDiagnostic>) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(1024);
        let errors: &mut dyn WriteColor = match self.color {
            DiagnosticColor::Ansi => &mut Ansi::new(&mut buffer),
            DiagnosticColor::None => &mut NoColor::new(&mut buffer),
        };
        let config = term::Config {
            tab_width: 2,
            ..Default::default()
        };

        for diagnostic in diagnostics {
            let diag = match diagnostic.severity {
                Severity::Error => CodespanDiagnostic::error(),
                Severity::Warning => CodespanDiagnostic::warning(),
            }
            .with_message(diagnostic.message.clone())
            .with_notes(
                diagnostic
                    .hints
                    .iter()
                    .filter(|hint| hint.span.is_detached())
                    .map(|hint| format!("hint: {}", hint.v))
                    .collect(),
            )
            .with_labels(
                self.label(diagnostic.span)
                    .into_iter()
                    .chain(diagnostic.hints.iter().filter_map(|hint| {
                        let id = hint.span.id()?;
                        let range = self.range(hint.span)?;
                        Some(Label::secondary(id, range).with_message(&hint.v))
                    }))
                    .collect(),
            );

            term::emit_to_write_style(errors, &config, self, &diag)
                .expect("Failed to format diagnostics");

            // Stacktrace-like helper diagnostics.
            for point in &diagnostic.trace {
                let message = point.v.to_string();
                let help = CodespanDiagnostic::help()
                    .with_message(message)
                    .with_labels(self.label(point.span).into_iter().collect());

                term::emit_to_write_style(errors, &config, self, &help)
                    .expect("Failed to format diagnostics");
            }
        }

        // codespan-reporting appends trailing blank lines after each diagnostic;
        // drop them so the serialized message ends cleanly.
        while buffer.last().is_some_and(u8::is_ascii_whitespace) {
            buffer.pop();
        }

        buffer
    }
}

/// A [`TemplateDiagnostics`] implementation that formats messages without
/// source context.
pub struct PlainDiagnostics;

impl TemplateDiagnostics for PlainDiagnostics {
    fn format_diagnostics(&self, diagnostics: EcoVec<SourceDiagnostic>) -> Vec<u8> {
        let mut buffer = String::new();
        for diagnostic in &diagnostics {
            let severity = match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            buffer.push_str(severity);
            buffer.push_str(": ");
            buffer.push_str(&diagnostic.message);
            buffer.push('\n');
            for hint in &diagnostic.hints {
                buffer.push_str("hint: ");
                buffer.push_str(&hint.v);
                buffer.push('\n');
            }
            for point in &diagnostic.trace {
                buffer.push_str("  ");
                buffer.push_str(&point.v.to_string());
                buffer.push('\n');
            }
        }
        buffer.truncate(buffer.trim_end().len());
        buffer.into_bytes()
    }
}

/// A compilation diagnostic, like a warning, with its location in the template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Whether this is an error or a warning.
    pub severity: DiagnosticSeverity,
    /// The message describing the problem.
    pub message: String,
    /// Where in the template the problem is, if it points into a file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    /// Hints on how to address the problem.
    pub hints: Vec<String>,
    /// The function calls leading to the problem, innermost first.
    pub trace: Vec<TracePoint>,
    /// The diagnostic formatted for humans, with source excerpts and the configured
    /// [`DiagnosticColor`].
    pub formatted: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.formatted)
    }
}

/// The severity of a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// A fatal error.
    Error,
    /// A non-fatal warning.
    Warning,
}

/// A range in a file of the template.
///
/// Lines and columns are 1-based. Columns count characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// Path of the file in the template, like `/main.typ`.
    /// Files of packages are prefixed with the package, like `@preview/example:0.1.0/lib.typ`.
    pub file: String,
    /// Line of the start of the range.
    pub line: usize,
    /// Column of the start of the range.
    pub column: usize,
    /// Line of the end of the range.
    pub end_line: usize,
    /// Column of the end of the range (exclusive).
    pub end_column: usize,
}

/// A step in the function calls leading to a [`Diagnostic`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracePoint {
    /// What happened at this step, like `error occurred in this function call`.
    pub message: String,
    /// Where the step happened, if it points into a file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

impl<Files: TemplateFiles> OicanaWorld<Files> {
    /// Resolve Typst source diagnostics into [`Diagnostic`]s.
    pub fn diagnostics(&self, diagnostics: EcoVec<SourceDiagnostic>) -> Vec<Diagnostic> {
        diagnostics
            .into_iter()
            .map(|diagnostic| Diagnostic {
                severity: match diagnostic.severity {
                    Severity::Error => DiagnosticSeverity::Error,
                    Severity::Warning => DiagnosticSeverity::Warning,
                },
                message: diagnostic.message.to_string(),
                location: self.location(diagnostic.span),
                hints: diagnostic
                    .hints
                    .iter()
                    .map(|hint| hint.v.to_string())
                    .collect(),
                trace: diagnostic
                    .trace
                    .iter()
                    .map(|point| TracePoint {
                        message: point.v.to_string(),
                        location: self.location(point.span),
                    })
                    .collect(),
                formatted: String::from_utf8_lossy(
                    &self.format_diagnostics(EcoVec::from([diagnostic])),
                )
                .into_owned(),
            })
            .collect()
    }

    fn location(&self, span: impl Into<DiagSpan>) -> Option<SourceLocation> {
        let span = span.into();
        let id = span.id()?;
        let range = self.range(span)?;
        let source = self.files.source(id).ok()?;
        let lines = source.lines();
        let (line, column) = lines.byte_to_line_column(range.start)?;
        let (end_line, end_column) = lines.byte_to_line_column(range.end)?;
        Some(SourceLocation {
            file: file_name(id),
            line: line + 1,
            column: column + 1,
            end_line: end_line + 1,
            end_column: end_column + 1,
        })
    }
}

/// Color mode for diagnostics
#[derive(Debug)]
pub enum DiagnosticColor {
    /// No colors in diagnostics
    None,
    /// ANSI codes for colors in diagnostics
    Ansi,
}
