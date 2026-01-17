use std::{env, path::PathBuf, sync::Arc};

use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyUserWarning, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyString, PyTuple};

use query::{QueryCommand, SerializationFormat, query as typst_query};
use std::collections::HashMap;
use typst::diag::SourceDiagnostic;
use typst::foundations::{Dict, Value};
use typst::text::FontStyle;
use world::SystemWorld;

mod compiler;
mod download;
mod query;
mod world;

// Create custom exceptions that inherit from RuntimeError
create_exception!(typst, TypstError, PyRuntimeError);

// TypstWarning inherits from UserWarning instead of RuntimeError since it's not an error
create_exception!(typst, TypstWarning, PyUserWarning);

/// Intermediate structure to hold diagnostic details without requiring GIL
#[derive(Debug, Clone)]
pub struct TypstDiagnosticDetails {
    pub message: String,
    pub hints: Vec<String>,
    pub trace: Vec<String>,
    pub diagnostic: String,
}

impl TypstDiagnosticDetails {
    /// Convert to TypstError (requires GIL)
    pub fn into_py_err(self, py: Python<'_>) -> PyResult<PyErr> {
        let exception = TypstError::new_err(self.message.clone());
        let exception_obj = exception.value(py);

        // Set our structured data as attributes
        exception_obj.setattr("message", self.message)?;
        exception_obj.setattr("hints", self.hints)?;
        exception_obj.setattr("trace", self.trace)?;
        exception_obj.setattr("diagnostic", self.diagnostic)?;

        Ok(exception)
    }

    /// Convert to TypstWarning (requires GIL)
    pub fn into_py_warning(self, py: Python<'_>) -> PyResult<PyErr> {
        let warning = TypstWarning::new_err(self.message.clone());
        let warning_obj = warning.value(py);

        // Set our structured data as attributes
        warning_obj.setattr("message", self.message)?;
        warning_obj.setattr("hints", self.hints)?;
        warning_obj.setattr("trace", self.trace)?;
        warning_obj.setattr("diagnostic", self.diagnostic)?;

        Ok(warning)
    }
}

/// Result of compilation that may include warnings
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub data: Vec<Vec<u8>>,
    pub warnings: Vec<TypstDiagnosticDetails>,
}

mod output_template {
    const INDEXABLE: [&str; 3] = ["{p}", "{0p}", "{n}"];

    pub fn has_indexable_template(output: &str) -> bool {
        INDEXABLE.iter().any(|template| output.contains(template))
    }

    pub fn format(output: &str, this_page: usize, total_pages: usize) -> String {
        // Find the base 10 width of number `i`
        fn width(i: usize) -> usize {
            1 + i.checked_ilog10().unwrap_or(0) as usize
        }

        let other_templates = ["{t}"];
        INDEXABLE
            .iter()
            .chain(other_templates.iter())
            .fold(output.to_string(), |out, template| {
                let replacement = match *template {
                    "{p}" => format!("{this_page}"),
                    "{0p}" | "{n}" => format!("{:01$}", this_page, width(total_pages)),
                    "{t}" => format!("{total_pages}"),
                    _ => unreachable!("unhandled template placeholder {template}"),
                };
                out.replace(template, replacement.as_str())
            })
    }
}

/// Create structured error details from diagnostics
fn create_typst_diagnostic_details(
    world: &SystemWorld,
    errors: &[SourceDiagnostic],
    warnings: &[SourceDiagnostic],
) -> TypstDiagnosticDetails {
    // Get the main error message by formatting all diagnostics
    let diagnostic = crate::compiler::format_diagnostics(world, errors, warnings)
        .unwrap_or_else(|e| format!("Failed to format diagnostic message: {:?}", e));

    // Extract structured information from the first error (most relevant)
    let primary_error = errors.first();
    let message = primary_error
        .map(|err| err.message.to_string())
        .unwrap_or_default();

    let (hints, trace) = if let Some(error) = primary_error {
        // Extract hints
        let hints = error
            .hints
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>();

        // Extract trace information
        let trace = error
            .trace
            .iter()
            .map(|point| format!("at {}", point.v))
            .collect::<Vec<_>>();

        (hints, trace)
    } else {
        (Vec::new(), Vec::new())
    };

    TypstDiagnosticDetails {
        message,
        hints,
        trace,
        diagnostic,
    }
}

/// Create structured warning details from diagnostics
fn create_typst_warning_details_from_diagnostics(
    world: &SystemWorld,
    warnings: &[SourceDiagnostic],
) -> Vec<TypstDiagnosticDetails> {
    warnings
        .iter()
        .map(|warning| {
            // Format just this warning
            let message = warning.message.to_string();
            let diagnostic =
                crate::compiler::format_diagnostics(world, &[], std::slice::from_ref(warning))
                    .unwrap_or_else(|_| message.clone());

            // Extract hints
            let hints = warning
                .hints
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<_>>();

            // Extract trace information
            let trace = warning
                .trace
                .iter()
                .map(|point| format!("at {}", point.v))
                .collect::<Vec<_>>();

            TypstDiagnosticDetails {
                message,
                hints,
                trace,
                diagnostic,
            }
        })
        .collect()
}

#[derive(FromPyObject)]
pub enum Input {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

/// Enum to represent sys_inputs parameter in compile methods:
/// - Ellipsis: keep existing sys_inputs (default)
/// - None: clear sys_inputs (empty dict)
/// - Dict: use the provided dictionary
#[derive(Debug, Clone)]
pub enum SysInputsOption {
    Keep,                         // Ellipsis - keep existing
    Clear,                        // None - clear to empty
    Set(HashMap<String, String>), // dict - use provided
}

/// Extract sys_inputs option from Python object
fn extract_sys_inputs_option(obj: &Bound<'_, PyAny>) -> PyResult<SysInputsOption> {
    // Check for Ellipsis (keep existing)
    if obj.is(obj.py().Ellipsis()) {
        return Ok(SysInputsOption::Keep);
    }
    // Check for None (clear)
    if obj.is_none() {
        return Ok(SysInputsOption::Clear);
    }
    // Otherwise, try to extract as dict
    let dict: HashMap<String, String> = obj.extract()?;
    Ok(SysInputsOption::Set(dict))
}

/// Immutable information about a single font variant
#[pyclass(module = "typst._typst", frozen)]
#[derive(Clone)]
pub struct FontInfo {
    /// The font family name
    #[pyo3(get)]
    family: String,
    /// The font style: "normal", "italic", or "oblique"
    #[pyo3(get)]
    style: String,
    /// The font weight (100-900)
    #[pyo3(get)]
    weight: u16,
    /// The font stretch ratio (0.5-2.0)
    #[pyo3(get)]
    stretch: f64,
    /// The file path of the font, or None for embedded fonts
    #[pyo3(get)]
    path: Option<String>,
    /// The index of the font in its collection (0 if not a collection)
    #[pyo3(get)]
    index: u32,
}

#[pymethods]
impl FontInfo {
    fn __repr__(&self) -> String {
        if let Some(path) = &self.path {
            format!(
                "FontInfo(family={:?}, style={:?}, weight={}, stretch={:.2}, path={:?}, index={})",
                self.family, self.style, self.weight, self.stretch, path, self.index
            )
        } else {
            format!(
                "FontInfo(family={:?}, style={:?}, weight={}, stretch={:.2}, path=None, index={})",
                self.family, self.style, self.weight, self.stretch, self.index
            )
        }
    }
}

#[pyclass(module = "typst._typst")]
#[derive(Clone)]
pub struct Fonts(Arc<typst_kit::fonts::Fonts>);

impl Fonts {
    fn font_style_to_string(style: FontStyle) -> &'static str {
        match style {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
            FontStyle::Oblique => "oblique",
        }
    }
}

#[pymethods]
impl Fonts {
    #[new]
    #[pyo3(signature = (
        include_system_fonts = true,
        include_embedded_fonts = true,
        font_paths = Vec::new(),
    ))]
    pub fn new(
        include_system_fonts: bool,
        include_embedded_fonts: bool,
        font_paths: Vec<PathBuf>,
    ) -> Self {
        let mut searcher = typst_kit::fonts::FontSearcher::new();
        searcher
            .include_system_fonts(include_system_fonts)
            .include_embedded_fonts(include_embedded_fonts);
        let fonts = searcher.search_with(&font_paths);
        Self(Arc::new(fonts))
    }

    /// Return a list of all font variants found
    ///
    /// Returns:
    ///     List[FontInfo]: A list of FontInfo objects for each font variant
    pub fn fonts(&self) -> Vec<FontInfo> {
        let book = &self.0.book;
        let font_slots = &self.0.fonts;

        let mut result = Vec::new();
        for (idx, slot) in font_slots.iter().enumerate() {
            if let Some(info) = book.info(idx) {
                let path = slot
                    .path()
                    .map(|p: &std::path::Path| p.to_string_lossy().into_owned());

                result.push(FontInfo {
                    family: info.family.clone(),
                    style: Self::font_style_to_string(info.variant.style).to_string(),
                    weight: info.variant.weight.to_number(),
                    stretch: info.variant.stretch.to_ratio().get(),
                    path,
                    index: slot.index(),
                });
            }
        }
        result
    }

    /// Return a sorted list of unique font family names
    ///
    /// Returns:
    ///     List[str]: A sorted list of unique font family names
    pub fn families(&self) -> Vec<String> {
        self.0
            .book
            .families()
            .map(|(family, _)| family.to_string())
            .collect()
    }
}

#[derive(FromPyObject)]
pub enum FontsOrPaths {
    Fonts(Fonts),
    Paths(Vec<PathBuf>),
}

/// A typst compiler
#[pyclass(module = "typst._typst")]
pub struct Compiler {
    world: SystemWorld,
}

impl Compiler {
    fn apply_input(&mut self, input: Option<Input>) -> PyResult<()> {
        if let Some(input) = input {
            self.world
                .set_input(input)
                .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))?;
        }
        Ok(())
    }

    fn apply_sys_inputs(&mut self, sys_inputs: SysInputsOption) {
        match sys_inputs {
            SysInputsOption::Keep => {
                // Do nothing, keep existing sys_inputs
            }
            SysInputsOption::Clear => {
                // Clear to empty dict
                self.world.set_inputs(Dict::default());
            }
            SysInputsOption::Set(inputs) => {
                // Set the provided inputs
                self.world.set_inputs(Dict::from_iter(
                    inputs
                        .into_iter()
                        .map(|(k, v)| (k.into(), Value::Str(v.into()))),
                ));
            }
        }
    }

    fn compile(
        &mut self,
        format: Option<&str>,
        ppi: Option<f32>,
        pdf_standards: &[typst_pdf::PdfStandard],
    ) -> Result<Vec<Vec<u8>>, TypstDiagnosticDetails> {
        let ret = match self
            .world
            .compile_with_diagnostics(format, ppi, pdf_standards)
        {
            Ok((buffer, _warnings)) => Ok(buffer), // Ignore warnings for backward compatibility
            Err((errors, warnings)) => Err(create_typst_diagnostic_details(
                &self.world,
                &errors,
                &warnings,
            )),
        };
        // Reset the world state after compilation to ensure file changes are detected in next compilation
        self.world.reset();
        ret
    }

    fn compile_with_warnings(
        &mut self,
        format: Option<&str>,
        ppi: Option<f32>,
        pdf_standards: &[typst_pdf::PdfStandard],
    ) -> Result<CompilationResult, TypstDiagnosticDetails> {
        let ret = match self
            .world
            .compile_with_diagnostics(format, ppi, pdf_standards)
        {
            Ok((buffer, warnings)) => {
                let warning_details =
                    create_typst_warning_details_from_diagnostics(&self.world, &warnings);
                Ok(CompilationResult {
                    data: buffer,
                    warnings: warning_details,
                })
            }
            Err((errors, warnings)) => Err(create_typst_diagnostic_details(
                &self.world,
                &errors,
                &warnings,
            )),
        };
        // Reset the world state after compilation to ensure file changes are detected in next compilation
        self.world.reset();
        ret
    }

    fn query(
        &mut self,
        selector: &str,
        field: Option<&str>,
        one: bool,
        format: Option<&str>,
    ) -> PyResult<String> {
        let format = format.unwrap_or("json");
        let format = match format {
            "json" => SerializationFormat::Json,
            "yaml" => SerializationFormat::Yaml,
            _ => return Err(PyValueError::new_err("unsupported serialization format")),
        };
        let result = typst_query(
            &mut self.world,
            &QueryCommand {
                selector: selector.into(),
                field: field.map(Into::into),
                one,
                format,
            },
        );
        match result {
            Ok(data) => Ok(data),
            Err(msg) => {
                // For query errors, we don't have structured diagnostics, so fall back to RuntimeError
                Err(PyRuntimeError::new_err(msg.to_string()))
            }
        }
    }
}

#[pymethods]
impl Compiler {
    /// Create a new typst compiler instance
    #[new]
    #[pyo3(signature = (
        input = None,
        root = None,
        font_paths = FontsOrPaths::Paths(Vec::new()),
        ignore_system_fonts = false,
        sys_inputs = HashMap::new(),
        package_path = None,
    ))]
    fn new(
        input: Option<Input>,
        root: Option<PathBuf>,
        font_paths: FontsOrPaths,
        ignore_system_fonts: bool,
        sys_inputs: HashMap<String, String>,
        package_path: Option<PathBuf>,
    ) -> PyResult<Self> {
        let input = input.unwrap_or_else(|| Input::Bytes(Vec::new()));
        let root = if let Some(root) = root {
            root.canonicalize()?
        } else if let Input::Path(path) = &input {
            path.canonicalize()?
                .parent()
                .map(Into::into)
                .unwrap_or_else(PathBuf::new)
        } else {
            env::current_dir()
                .and_then(|cwd| cwd.canonicalize())
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?
        };
        let fonts = match font_paths {
            FontsOrPaths::Paths(paths) => Fonts::new(!ignore_system_fonts, true, paths),
            FontsOrPaths::Fonts(fonts) => fonts,
        };

        // Create the world that serves sources, fonts and files.
        let world = SystemWorld::builder(root, input)
            .inputs(Dict::from_iter(
                sys_inputs
                    .into_iter()
                    .map(|(k, v)| (k.into(), Value::Str(v.into()))),
            ))
            .fonts(Some(fonts.0))
            .package_path(package_path)
            .build()
            .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))?;
        Ok(Self { world })
    }

    /// Compile a typst file to PDF
    #[allow(clippy::too_many_arguments)]
    #[pyo3(name = "compile", signature = (input = None, output = None, format = None, ppi = None, sys_inputs = SysInputsOption::Keep, pdf_standards = Vec::new()))]
    fn py_compile(
        &mut self,
        py: Python<'_>,
        input: Option<Input>,
        output: Option<PathBuf>,
        format: Option<&str>,
        ppi: Option<f32>,
        #[pyo3(from_py_with = extract_sys_inputs_option)] sys_inputs: SysInputsOption,
        #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
    ) -> PyResult<Py<PyAny>> {
        self.apply_input(input)?;
        self.apply_sys_inputs(sys_inputs);
        if let Some(output) = output {
            // if format is None and output with postfix ".pdf", ".png", "html", and ".svg" is
            // provided, use the postfix as format
            let format = match format {
                Some(format) => Some(format),
                None => {
                    let output = output.to_str().expect("There should be one");
                    if output.ends_with(".pdf") {
                        Some("pdf")
                    } else if output.ends_with(".png") {
                        Some("png")
                    } else if output.ends_with(".svg") {
                        Some("svg")
                    } else if output.ends_with(".html") {
                        Some("html")
                    } else {
                        None
                    }
                }
            };

            let buffers = py
                .detach(|| self.compile(format, ppi, &pdf_standards))
                .map_err(|err_details| err_details.into_py_err(py).unwrap())?;

            let can_handle_multiple =
                output_template::has_indexable_template(output.to_str().unwrap());
            if !can_handle_multiple && buffers.len() > 1 {
                return Err(PyRuntimeError::new_err(
                    "output path does not support multiple pages".to_string(),
                ));
            }
            if !can_handle_multiple && buffers.len() == 1 {
                // Write a single buffer to the output file
                std::fs::write(output, &buffers[0])?;
            } else {
                // Write each buffer to a separate file
                for (i, buffer) in buffers.iter().enumerate() {
                    let output =
                        output_template::format(output.to_str().unwrap(), i + 1, buffers.len());
                    std::fs::write(output, buffer)?;
                }
            }
            Ok(py.None())
        } else {
            let buffers = py
                .detach(|| self.compile(format, ppi, &pdf_standards))
                .map_err(|err_details| err_details.into_py_err(py).unwrap())?;
            if buffers.len() == 1 {
                // Return a single buffer as a single byte string
                Ok(PyBytes::new(py, &buffers[0]).into())
            } else {
                let list = PyList::empty(py);
                for buffer in buffers {
                    list.append(PyBytes::new(py, &buffer))?;
                }
                Ok(list.into())
            }
        }
    }

    /// Compile a typst file and return both result and warnings
    #[allow(clippy::too_many_arguments)]
    #[pyo3(name = "compile_with_warnings", signature = (input = None, output = None, format = None, ppi = None, sys_inputs = SysInputsOption::Keep, pdf_standards = Vec::new()))]
    fn py_compile_with_warnings(
        &mut self,
        py: Python<'_>,
        input: Option<Input>,
        output: Option<PathBuf>,
        format: Option<&str>,
        ppi: Option<f32>,
        #[pyo3(from_py_with = extract_sys_inputs_option)] sys_inputs: SysInputsOption,
        #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
    ) -> PyResult<Py<PyAny>> {
        self.apply_input(input)?;
        self.apply_sys_inputs(sys_inputs);
        let result = py
            .detach(|| self.compile_with_warnings(format, ppi, &pdf_standards))
            .map_err(|err_details| err_details.into_py_err(py).unwrap())?;

        // Convert warnings to Python objects
        let warnings_list = PyList::empty(py);
        for warning_detail in &result.warnings {
            let warning_obj = warning_detail.clone().into_py_warning(py)?;
            warnings_list.append(warning_obj.value(py))?;
        }

        if let Some(output) = output {
            let can_handle_multiple =
                output_template::has_indexable_template(output.to_str().unwrap());
            if !can_handle_multiple && result.data.len() > 1 {
                return Err(PyRuntimeError::new_err(
                    "output path does not support multiple pages".to_string(),
                ));
            }
            if !can_handle_multiple && result.data.len() == 1 {
                // Write a single buffer to the output file
                std::fs::write(output, &result.data[0])?;
            } else {
                // Write each buffer to a separate file
                for (i, buffer) in result.data.iter().enumerate() {
                    let output =
                        output_template::format(output.to_str().unwrap(), i + 1, result.data.len());
                    std::fs::write(output, buffer)?;
                }
            }

            // Return (None, warnings) tuple
            Ok(PyTuple::new(py, [py.None(), warnings_list.into()])?.into())
        } else {
            let compiled_data: Py<PyAny> = if result.data.len() == 1 {
                // Return a single buffer as a single byte string
                PyBytes::new(py, &result.data[0]).into()
            } else {
                let list = PyList::empty(py);
                for buffer in result.data {
                    list.append(PyBytes::new(py, &buffer))?;
                }
                list.into()
            };

            // Return (data, warnings) tuple
            Ok(PyTuple::new(py, [compiled_data, warnings_list.into()])?.into())
        }
    }

    /// Query a typst document
    #[pyo3(name = "query", signature = (selector, field = None, one = false, format = None))]
    fn py_query(
        &mut self,
        py: Python<'_>,
        selector: &str,
        field: Option<&str>,
        one: bool,
        format: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        py.detach(|| self.query(selector, field, one, format))
            .map(|s| PyString::new(py, &s).into())
    }
}

/// Compile a typst document
#[pyfunction]
#[pyo3(signature = (
    input,
    output = None,
    root = None,
    font_paths = FontsOrPaths::Paths(Vec::new()),
    ignore_system_fonts = false,
    format = None, ppi = None,
    sys_inputs = HashMap::new(),
    pdf_standards = Vec::new(),
    package_path=None,
))]
#[allow(clippy::too_many_arguments)]
fn compile(
    py: Python<'_>,
    input: Input,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    font_paths: FontsOrPaths,
    ignore_system_fonts: bool,
    format: Option<&str>,
    ppi: Option<f32>,
    sys_inputs: HashMap<String, String>,
    #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
    package_path: Option<PathBuf>,
) -> PyResult<Py<PyAny>> {
    let mut compiler = Compiler::new(
        Some(input),
        root,
        font_paths,
        ignore_system_fonts,
        sys_inputs,
        package_path,
    )?;
    compiler.py_compile(
        py,
        None,
        output,
        format,
        ppi,
        SysInputsOption::Keep,
        pdf_standards,
    )
}

/// Compile a typst file and return both result and warnings
#[pyfunction]
#[pyo3(signature = (
    input,
    output = None,
    root = None,
    font_paths = FontsOrPaths::Paths(Vec::new()),
    ignore_system_fonts = false,
    format = None, ppi = None,
    sys_inputs = HashMap::new(),
    pdf_standards = Vec::new(),
    package_path=None
))]
#[allow(clippy::too_many_arguments)]
fn compile_with_warnings(
    py: Python<'_>,
    input: Input,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    font_paths: FontsOrPaths,
    ignore_system_fonts: bool,
    format: Option<&str>,
    ppi: Option<f32>,
    sys_inputs: HashMap<String, String>,
    #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
    package_path: Option<PathBuf>,
) -> PyResult<Py<PyAny>> {
    let mut compiler = Compiler::new(
        Some(input),
        root,
        font_paths,
        ignore_system_fonts,
        sys_inputs,
        package_path,
    )?;
    compiler.py_compile_with_warnings(
        py,
        None,
        output,
        format,
        ppi,
        SysInputsOption::Keep,
        pdf_standards,
    )
}

/// Query a typst document
#[pyfunction]
#[pyo3(
    name = "query",
    signature = (
        input,
        selector,
        field = None,
        one = false,
        format = None,
        root = None,
        font_paths = FontsOrPaths::Paths(Vec::new()),
        ignore_system_fonts = false,
        sys_inputs = HashMap::new(),
        package_path=None,
    )
)]
#[allow(clippy::too_many_arguments)]
fn py_query(
    py: Python<'_>,
    input: Input,
    selector: &str,
    field: Option<&str>,
    one: bool,
    format: Option<&str>,
    root: Option<PathBuf>,
    font_paths: FontsOrPaths,
    ignore_system_fonts: bool,
    sys_inputs: HashMap<String, String>,
    package_path: Option<PathBuf>,
) -> PyResult<Py<PyAny>> {
    let mut compiler = Compiler::new(
        Some(input),
        root,
        font_paths,
        ignore_system_fonts,
        sys_inputs,
        package_path,
    )?;
    compiler.py_query(py, selector, field, one, format)
}

/// Python binding to typst
#[pymodule(gil_used = false)]
mod _typst {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::{Compiler, FontInfo, Fonts, TypstError, TypstWarning};

    #[pymodule_export]
    use super::{compile, compile_with_warnings, py_query as query};

    #[pymodule_init]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        Ok(())
    }
}

fn extract_pdf_standard(obj: &Bound<'_, PyAny>) -> PyResult<typst_pdf::PdfStandard> {
    match &*obj.extract::<std::borrow::Cow<'_, str>>()? {
        "1.4" => Ok(typst_pdf::PdfStandard::V_1_4),
        "1.5" => Ok(typst_pdf::PdfStandard::V_1_5),
        "1.6" => Ok(typst_pdf::PdfStandard::V_1_6),
        "1.7" => Ok(typst_pdf::PdfStandard::V_1_7),
        "2.0" => Ok(typst_pdf::PdfStandard::V_2_0),
        "a-1a" => Ok(typst_pdf::PdfStandard::A_1a),
        "a-1b" => Ok(typst_pdf::PdfStandard::A_1b),
        "a-2a" => Ok(typst_pdf::PdfStandard::A_2a),
        "a-2b" => Ok(typst_pdf::PdfStandard::A_2b),
        "a-2u" => Ok(typst_pdf::PdfStandard::A_2u),
        "a-3a" => Ok(typst_pdf::PdfStandard::A_3a),
        "a-3b" => Ok(typst_pdf::PdfStandard::A_3b),
        "a-3u" => Ok(typst_pdf::PdfStandard::A_3u),
        "a-4" => Ok(typst_pdf::PdfStandard::A_4),
        "a-4e" => Ok(typst_pdf::PdfStandard::A_4e),
        "a-4f" => Ok(typst_pdf::PdfStandard::A_4f),
        "ua-1" => Ok(typst_pdf::PdfStandard::Ua_1),
        s => Err(PyValueError::new_err(format!("unknown pdf standard: {s}"))),
    }
}

fn extract_pdf_standards(obj: &Bound<'_, PyAny>) -> PyResult<Vec<typst_pdf::PdfStandard>> {
    if obj.is_none() {
        Ok(vec![])
    } else if let Ok(s) = obj.cast::<PyList>() {
        s.iter().map(|s| extract_pdf_standard(&s)).collect()
    } else {
        extract_pdf_standard(obj).map(|s| vec![s])
    }
}
