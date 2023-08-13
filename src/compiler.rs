use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{self, termcolor};
use typst::diag::{Severity, SourceDiagnostic, StrResult};
use typst::doc::Document;
use typst::eval::{eco_format, Tracer};
use typst::syntax::{FileId, Source};
use typst::World;

use crate::world::SystemWorld;

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

impl SystemWorld {
    pub fn compile(&mut self) -> StrResult<Vec<u8>> {
        // Reset everything and ensure that the main file is present.
        self.reset();
        self.source(self.main()).map_err(|err| err.to_string())?;

        let mut tracer = Tracer::default();
        let result = typst::compile(self, &mut tracer);
        let warnings = tracer.warnings();

        match result {
            // Export the PDF / PNG.
            Ok(document) => Ok(export_pdf(&document)?),
            Err(errors) => Err(format_diagnostics(self, &errors, &warnings).unwrap().into()),
        }
    }
}

/// Export to a PDF.
#[inline]
fn export_pdf(document: &Document) -> StrResult<Vec<u8>> {
    let buffer = typst::export::pdf(document);
    Ok(buffer)
}

/// Format diagnostic messages.\
pub fn format_diagnostics(
    world: &SystemWorld,
    errors: &[SourceDiagnostic],
    warnings: &[SourceDiagnostic],
) -> Result<String, codespan_reporting::files::Error> {
    let mut w = termcolor::Buffer::no_color();

    let config = term::Config {
        tab_width: 2,
        ..Default::default()
    };

    for diagnostic in warnings.iter().chain(errors.iter()) {
        let diag = match diagnostic.severity {
            Severity::Error => Diagnostic::error(),
            Severity::Warning => Diagnostic::warning(),
        }
        .with_message(diagnostic.message.clone())
        .with_notes(
            diagnostic
                .hints
                .iter()
                .map(|e| (eco_format!("hint: {e}")).into())
                .collect(),
        )
        .with_labels(vec![Label::primary(
            diagnostic.span.id(),
            world.range(diagnostic.span),
        )]);

        term::emit(&mut w, &config, world, &diag)?;

        // Stacktrace-like helper diagnostics.
        for point in &diagnostic.trace {
            let message = point.v.to_string();
            let help = Diagnostic::help()
                .with_message(message)
                .with_labels(vec![Label::primary(
                    point.span.id(),
                    world.range(point.span),
                )]);

            term::emit(&mut w, &config, world, &help)?;
        }
    }

    let s = String::from_utf8(w.into_inner()).unwrap();
    Ok(s)
}

impl<'a> codespan_reporting::files::Files<'a> for SystemWorld {
    type FileId = FileId;
    type Name = FileId;
    type Source = Source;

    fn name(&'a self, id: FileId) -> CodespanResult<Self::Name> {
        Ok(id)
    }

    fn source(&'a self, id: FileId) -> CodespanResult<Self::Source> {
        Ok(self.lookup(id))
    }

    fn line_index(&'a self, id: FileId, given: usize) -> CodespanResult<usize> {
        let source = self.lookup(id);
        source
            .byte_to_line(given)
            .ok_or_else(|| CodespanError::IndexTooLarge {
                given,
                max: source.len_bytes(),
            })
    }

    fn line_range(&'a self, id: FileId, given: usize) -> CodespanResult<std::ops::Range<usize>> {
        let source = self.lookup(id);
        source
            .line_to_range(given)
            .ok_or_else(|| CodespanError::LineTooLarge {
                given,
                max: source.len_lines(),
            })
    }

    fn column_number(&'a self, id: FileId, _: usize, given: usize) -> CodespanResult<usize> {
        let source = self.lookup(id);
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
