use chrono::{Datelike, Timelike};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{self, termcolor};
use ecow::eco_format;
use typst::WorldExt;
use typst::diag::{At, Severity, SourceDiagnostic, SourceResult, StrResult, Warned};
use typst::foundations::Datetime;
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Lines, Span};
use typst_html::HtmlDocument;

use crate::world::SystemWorld;

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

type CompileSuccess = (Vec<Vec<u8>>, Vec<SourceDiagnostic>);
type CompileError = (Vec<SourceDiagnostic>, Vec<SourceDiagnostic>);

impl SystemWorld {
    /// Compile and return structured diagnostics for error handling
    pub fn compile_with_diagnostics(
        &mut self,
        format: Option<&str>,
        ppi: Option<f32>,
        pdf_standards: &[typst_pdf::PdfStandard],
    ) -> Result<CompileSuccess, CompileError> {
        let normalized_format = format.unwrap_or("pdf").to_ascii_lowercase();

        let Warned { output, warnings } = match normalized_format.as_str() {
            "html" => self.compile_and_export_html(),
            "pdf" | "png" | "svg" => {
                self.compile_and_export_paged(normalized_format.as_str(), ppi, pdf_standards)
            }
            _ => return Err((vec![], vec![])),
        };

        match output {
            Ok(data) => Ok((data, warnings.to_vec())),
            Err(errors) => Err((errors.to_vec(), warnings.to_vec())),
        }
    }

    /// Compile and export paginated formats (PDF, PNG, SVG) - similar to compile_and_export in typst-cli
    fn compile_and_export_paged(
        &mut self,
        format: &str,
        ppi: Option<f32>,
        pdf_standards: &[typst_pdf::PdfStandard],
    ) -> Warned<SourceResult<Vec<Vec<u8>>>> {
        let Warned { output, warnings } = typst::compile::<PagedDocument>(self);
        // Evict comemo cache to limit memory usage after compilation
        comemo::evict(10);

        let result = output.and_then(|document| match format {
            "pdf" => {
                let standards = typst_pdf::PdfStandards::new(pdf_standards)
                    .map_err(|e| eco_format!("PDF standards error: {:?}", e))
                    .at(Span::detached())?;
                export_pdf(&document, self, standards).map(|pdf| vec![pdf])
            }
            "png" => export_image(&document, ImageExportFormat::Png, ppi).at(Span::detached()),
            "svg" => export_image(&document, ImageExportFormat::Svg, ppi).at(Span::detached()),
            _ => unreachable!(),
        });

        Warned {
            output: result,
            warnings,
        }
    }

    /// Compile and export HTML format - similar to compile_and_export in typst-cli
    fn compile_and_export_html(&mut self) -> Warned<SourceResult<Vec<Vec<u8>>>> {
        let Warned { output, warnings } = typst::compile::<HtmlDocument>(self);
        // Evict comemo cache to limit memory usage after compilation
        comemo::evict(10);

        let result =
            output.and_then(|document| export_html(&document, self).map(|html| vec![html]));

        Warned {
            output: result,
            warnings,
        }
    }
}

/// Export to a html.
#[inline]
fn export_html(document: &HtmlDocument, _world: &SystemWorld) -> SourceResult<Vec<u8>> {
    let buffer = typst_html::html(document)?;
    Ok(buffer.into())
}

/// Export to a PDF.
#[inline]
fn export_pdf(
    document: &PagedDocument,
    _world: &SystemWorld,
    standards: typst_pdf::PdfStandards,
) -> SourceResult<Vec<u8>> {
    let buffer = typst_pdf::pdf(
        document,
        &typst_pdf::PdfOptions {
            ident: typst::foundations::Smart::Auto,
            timestamp: now().map(typst_pdf::Timestamp::new_utc),
            standards,
            ..Default::default()
        },
    )?;
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

/// Export the frames to PNGs or SVGs.
fn export_image(
    document: &PagedDocument,
    fmt: ImageExportFormat,
    ppi: Option<f32>,
) -> StrResult<Vec<Vec<u8>>> {
    let mut buffers = Vec::new();
    for page in &document.pages {
        let buffer = match fmt {
            ImageExportFormat::Png => typst_render::render(page, ppi.unwrap_or(144.0) / 72.0)
                .encode_png()
                .map_err(|err| eco_format!("failed to write PNG file ({err})"))?,
            ImageExportFormat::Svg => {
                let svg = typst_svg::svg(page);
                svg.as_bytes().to_vec()
            }
        };
        buffers.push(buffer);
    }
    Ok(buffers)
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

        term::emit_to_write_style(&mut w, &config, world, &diag)?;

        // Stacktrace-like helper diagnostics.
        for point in &diagnostic.trace {
            let message = point.v.to_string();
            let help = Diagnostic::help()
                .with_message(message)
                .with_labels(label(world, point.span).into_iter().collect());

            term::emit_to_write_style(&mut w, &config, world, &help)?;
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
    type Source = Lines<String>;

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
