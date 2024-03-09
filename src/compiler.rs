use chrono::{Datelike, Timelike};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{self, termcolor};
use ecow::eco_format;
use typst::diag::{Severity, SourceDiagnostic, StrResult};
use typst::eval::Tracer;
use typst::foundations::Datetime;
use typst::model::Document;
use typst::syntax::{FileId, Source, Span};
use typst::visualize::Color;
use typst::{World, WorldExt};

use crate::world::SystemWorld;

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

impl SystemWorld {
    pub fn compile(&mut self, format: Option<&str>, ppi: Option<f32>) -> StrResult<Vec<u8>> {
        // Reset everything and ensure that the main file is present.
        self.reset();
        self.source(self.main()).map_err(|err| err.to_string())?;

        let mut tracer = Tracer::default();
        let result = typst::compile(self, &mut tracer);
        let warnings = tracer.warnings();

        match result {
            // Export the PDF / PNG.
            Ok(document) => {
                // Assert format is "pdf" or "png" or "svg"
                match format.unwrap_or("pdf").to_ascii_lowercase().as_str() {
                    "pdf" => Ok(export_pdf(&document, self)?),
                    "png" => Ok(export_image(&document, ImageExportFormat::Png, ppi)?),
                    "svg" => Ok(export_image(&document, ImageExportFormat::Svg, ppi)?),
                    fmt => Err(eco_format!("unknown format: {fmt}")),
                }
            }
            Err(errors) => Err(format_diagnostics(self, &errors, &warnings).unwrap().into()),
        }
    }
}

/// Export to a PDF.
#[inline]
fn export_pdf(document: &Document, world: &SystemWorld) -> StrResult<Vec<u8>> {
    let ident = world.input().to_string_lossy();
    let buffer = typst_pdf::pdf(document, typst::foundations::Smart::Custom(&ident), now());
    Ok(buffer)
}

/// Get the current date and time in UTC.
fn now() -> Option<Datetime> {
    let now = chrono::Local::now().naive_utc();
    Datetime::from_ymd_hms(
        now.year(),
        now.month().try_into().ok()?,
        now.day().try_into().ok()?,
        now.hour().try_into().ok()?,
        now.minute().try_into().ok()?,
        now.second().try_into().ok()?,
    )
}

/// An image format to export in.
enum ImageExportFormat {
    Png,
    Svg,
}

/// Export the first frame to PNG or SVG.
fn export_image(
    document: &Document,
    fmt: ImageExportFormat,
    ppi: Option<f32>,
) -> StrResult<Vec<u8>> {
    // Find the first frame
    let frame = &document.pages.first().unwrap().frame;
    match fmt {
        ImageExportFormat::Png => {
            let pixmap = typst_render::render(frame, ppi.unwrap_or(144.0) / 72.0, Color::WHITE);
            pixmap
                .encode_png()
                .map_err(|err| eco_format!("failed to write PNG file ({err})"))
        }
        ImageExportFormat::Svg => {
            let svg = typst_svg::svg(frame);
            Ok(svg.as_bytes().to_vec())
        }
    }
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
        .with_labels(label(world, diagnostic.span).into_iter().collect());

        term::emit(&mut w, &config, world, &diag)?;

        // Stacktrace-like helper diagnostics.
        for point in &diagnostic.trace {
            let message = point.v.to_string();
            let help = Diagnostic::help()
                .with_message(message)
                .with_labels(label(world, point.span).into_iter().collect());

            term::emit(&mut w, &config, world, &help)?;
        }
    }

    let s = String::from_utf8(w.into_inner()).unwrap();
    Ok(s)
}

/// Create a label for a span.
fn label(world: &SystemWorld, span: Span) -> Option<Label<FileId>> {
    Some(Label::primary(span.id()?, world.range(span)?))
}

impl<'a> codespan_reporting::files::Files<'a> for SystemWorld {
    type FileId = FileId;
    type Name = String;
    type Source = Source;

    fn name(&'a self, id: FileId) -> CodespanResult<Self::Name> {
        let vpath = id.vpath();
        Ok(if let Some(package) = id.package() {
            format!("{package}{}", vpath.as_rooted_path().display())
        } else {
            // Try to express the path relative to the working directory.
            vpath
                .resolve(self.root())
                .and_then(|abs| pathdiff::diff_paths(abs, self.workdir()))
                .as_deref()
                .unwrap_or_else(|| vpath.as_rootless_path())
                .to_string_lossy()
                .into()
        })
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
