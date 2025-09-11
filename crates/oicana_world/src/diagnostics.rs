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
    syntax::{FileId, Source, Span},
    WorldExt,
};

use crate::world::OicanaWorld;

impl<'a, Files: TemplateFiles> CodespanFiles<'a> for OicanaWorld<Files> {
    type FileId = FileId;
    type Name = String;
    type Source = Source;

    fn name(&'a self, id: FileId) -> Result<Self::Name, CodespanError> {
        let vpath = id.vpath();
        Ok(if let Some(package) = id.package() {
            format!("{package}{}", vpath.as_rooted_path().display())
        } else {
            vpath.as_rooted_path().to_string_lossy().into()
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
    fn label(&self, span: Span) -> Option<Label<FileId>> {
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
                    .map(|e| format!("hint: {e}"))
                    .collect(),
            )
            .with_labels(self.label(diagnostic.span).into_iter().collect());

            term::emit(errors, &config, self, &diag).expect("Failed to format diagnostics");

            // Stacktrace-like helper diagnostics.
            for point in &diagnostic.trace {
                let message = point.v.to_string();
                let help = Diagnostic::help()
                    .with_message(message)
                    .with_labels(self.label(point.span).into_iter().collect());

                term::emit(errors, &config, self, &help).expect("Failed to format diagnostics");
            }
        }

        buffer
    }
}

/// Color mode for diagnostic logs
#[derive(Debug)]
pub enum DiagnosticColor {
    /// No colors in diagnostic output
    None,
    /// ANSI codes for colors in diagnostic output
    Ansi,
}
