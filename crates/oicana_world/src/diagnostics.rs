use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::{Error as CodespanError, Files as CodespanFiles},
    term::{
        self,
        termcolor::{Ansi, NoColor, WriteColor},
    },
};
use ecow::EcoVec;
use oicana_files::TemplateFiles;
use typst::{
    diag::{Severity, SourceDiagnostic},
    syntax::{DiagSpan, FileId, Source, VirtualRoot},
    WorldExt,
};

use crate::world::OicanaWorld;

impl<'a, Files: TemplateFiles> CodespanFiles<'a> for OicanaWorld<Files> {
    type FileId = FileId;
    type Name = String;
    type Source = Source;

    fn name(&'a self, id: FileId) -> Result<Self::Name, CodespanError> {
        let rooted = id.vpath().get_with_slash();
        Ok(match id.root() {
            VirtualRoot::Project => rooted.to_string(),
            VirtualRoot::Package(package) => format!("{package}{rooted}"),
        })
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
                Severity::Error => Diagnostic::error(),
                Severity::Warning => Diagnostic::warning(),
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
                let help = Diagnostic::help()
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

/// Color mode for diagnostics
#[derive(Debug)]
pub enum DiagnosticColor {
    /// No colors in diagnostics
    None,
    /// ANSI codes for colors in diagnostics
    Ansi,
}
