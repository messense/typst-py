use std::path::PathBuf;

use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyUserWarning, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyString, PyTuple};

use query::{query as typst_query, QueryCommand, SerializationFormat};
use std::collections::HashMap;
use typst::diag::SourceDiagnostic;
use typst::foundations::{Dict, Value};
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
    let formatted_message = crate::compiler::format_diagnostics(world, errors, warnings)
        .unwrap_or_else(|_| "Failed to format diagnostic message".to_string());

    // Extract structured information from the first error (most relevant)
    let primary_error = errors.first();

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
        message: formatted_message,
        hints,
        trace,
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
            let formatted_message =
                crate::compiler::format_diagnostics(world, &[], &[warning.clone()])
                    .unwrap_or_else(|_| warning.message.to_string());

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
                message: formatted_message,
                hints,
                trace,
            }
        })
        .collect()
}

#[derive(FromPyObject)]
pub enum Input {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

/// A typst compiler
#[pyclass(module = "typst._typst")]
pub struct Compiler {
    world: SystemWorld,
}

impl Compiler {
    fn compile(
        &mut self,
        format: Option<&str>,
        ppi: Option<f32>,
        pdf_standards: &[typst_pdf::PdfStandard],
    ) -> Result<Vec<Vec<u8>>, TypstDiagnosticDetails> {
        match self
            .world
            .compile_with_diagnostics(format, ppi, pdf_standards)
        {
            Ok((buffer, _warnings)) => Ok(buffer), // Ignore warnings for backward compatibility
            Err((errors, warnings)) => Err(create_typst_diagnostic_details(
                &self.world,
                &errors,
                &warnings,
            )),
        }
    }

    fn compile_with_warnings(
        &mut self,
        format: Option<&str>,
        ppi: Option<f32>,
        pdf_standards: &[typst_pdf::PdfStandard],
    ) -> Result<CompilationResult, TypstDiagnosticDetails> {
        match self
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
        }
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
        input,
        root = None,
        font_paths = Vec::new(),
        ignore_system_fonts = false,
        sys_inputs = HashMap::new()
    ))]
    fn new(
        input: Input,
        root: Option<PathBuf>,
        font_paths: Vec<PathBuf>,
        ignore_system_fonts: bool,
        sys_inputs: HashMap<String, String>,
    ) -> PyResult<Self> {
        let root = if let Some(root) = root {
            root.canonicalize()?
        } else if let Input::Path(path) = &input {
            path.canonicalize()?
                .parent()
                .map(Into::into)
                .unwrap_or_else(|| PathBuf::new())
        } else {
            PathBuf::new()
        };

        // Create the world that serves sources, fonts and files.
        let world = SystemWorld::builder(root, input)
            .inputs(Dict::from_iter(
                sys_inputs
                    .into_iter()
                    .map(|(k, v)| (k.into(), Value::Str(v.into()))),
            ))
            .font_paths(font_paths)
            .ignore_system_fonts(ignore_system_fonts)
            .build()
            .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))?;
        Ok(Self { world })
    }

    /// Compile a typst file to PDF
    #[pyo3(name = "compile", signature = (output = None, format = None, ppi = None, pdf_standards = Vec::new()))]
    fn py_compile(
        &mut self,
        py: Python<'_>,
        output: Option<PathBuf>,
        format: Option<&str>,
        ppi: Option<f32>,
        #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
    ) -> PyResult<PyObject> {
        if let Some(output) = output {
            // if format is None and output with postfix ".pdf", ".png" and ".svg" is
            // provided, use the postfix as format
            let format = match format {
                Some(format) => Some(format),
                None => {
                    let output = output.to_str().unwrap();
                    if output.ends_with(".pdf") {
                        Some("pdf")
                    } else if output.ends_with(".png") {
                        Some("png")
                    } else if output.ends_with(".svg") {
                        Some("svg")
                    } else {
                        None
                    }
                }
            };

            let buffers = py
                .allow_threads(|| self.compile(format, ppi, &pdf_standards))
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
                .allow_threads(|| self.compile(format, ppi, &pdf_standards))
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
    #[pyo3(name = "compile_with_warnings", signature = (output = None, format = None, ppi = None, pdf_standards = Vec::new()))]
    fn py_compile_with_warnings(
        &mut self,
        py: Python<'_>,
        output: Option<PathBuf>,
        format: Option<&str>,
        ppi: Option<f32>,
        #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
    ) -> PyResult<PyObject> {
        let result = py
            .allow_threads(|| self.compile_with_warnings(format, ppi, &pdf_standards))
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
            let compiled_data: PyObject = if result.data.len() == 1 {
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
    ) -> PyResult<PyObject> {
        py.allow_threads(|| self.query(selector, field, one, format))
            .map(|s| PyString::new(py, &s).into())
    }
}

/// Compile a typst document
#[pyfunction]
#[pyo3(signature = (
    input,
    output = None,
    root = None,
    font_paths = Vec::new(),
    ignore_system_fonts = false,
    format = None, ppi = None,
    sys_inputs = HashMap::new(),
    pdf_standards = Vec::new()
))]
#[allow(clippy::too_many_arguments)]
fn compile(
    py: Python<'_>,
    input: Input,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    font_paths: Vec<PathBuf>,
    ignore_system_fonts: bool,
    format: Option<&str>,
    ppi: Option<f32>,
    sys_inputs: HashMap<String, String>,
    #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
) -> PyResult<PyObject> {
    let mut compiler = Compiler::new(input, root, font_paths, ignore_system_fonts, sys_inputs)?;
    compiler.py_compile(py, output, format, ppi, pdf_standards)
}

/// Compile a typst file and return both result and warnings
#[pyfunction]
#[pyo3(signature = (
    input,
    output = None,
    root = None,
    font_paths = Vec::new(),
    ignore_system_fonts = false,
    format = None, ppi = None,
    sys_inputs = HashMap::new(),
    pdf_standards = Vec::new()
))]
#[allow(clippy::too_many_arguments)]
fn compile_with_warnings(
    py: Python<'_>,
    input: Input,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    font_paths: Vec<PathBuf>,
    ignore_system_fonts: bool,
    format: Option<&str>,
    ppi: Option<f32>,
    sys_inputs: HashMap<String, String>,
    #[pyo3(from_py_with = extract_pdf_standards)] pdf_standards: Vec<typst_pdf::PdfStandard>,
) -> PyResult<PyObject> {
    let mut compiler = Compiler::new(input, root, font_paths, ignore_system_fonts, sys_inputs)?;
    compiler.py_compile_with_warnings(py, output, format, ppi, pdf_standards)
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
        font_paths = Vec::new(),
        ignore_system_fonts = false,
        sys_inputs = HashMap::new()
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
    font_paths: Vec<PathBuf>,
    ignore_system_fonts: bool,
    sys_inputs: HashMap<String, String>,
) -> PyResult<PyObject> {
    let mut compiler = Compiler::new(input, root, font_paths, ignore_system_fonts, sys_inputs)?;
    compiler.py_query(py, selector, field, one, format)
}

/// Python binding to typst
#[pymodule(gil_used = false)]
fn _typst(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Compiler>()?;
    m.add("TypstError", py.get_type::<TypstError>())?;
    m.add("TypstWarning", py.get_type::<TypstWarning>())?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(compile_with_warnings, m)?)?;
    m.add_function(wrap_pyfunction!(py_query, m)?)?;
    Ok(())
}

fn extract_pdf_standard(obj: &Bound<'_, PyAny>) -> PyResult<typst_pdf::PdfStandard> {
    match &*obj.extract::<std::borrow::Cow<'_, str>>()? {
        "1.7" => Ok(typst_pdf::PdfStandard::V_1_7),
        "a-2b" => Ok(typst_pdf::PdfStandard::A_2b),
        "a-3b" => Ok(typst_pdf::PdfStandard::A_3b),
        s => Err(PyValueError::new_err(format!("unknown pdf standard: {s}"))),
    }
}

fn extract_pdf_standards(obj: &Bound<'_, PyAny>) -> PyResult<Vec<typst_pdf::PdfStandard>> {
    if obj.is_none() {
        Ok(vec![])
    } else if let Ok(s) = obj.downcast::<PyList>() {
        s.iter().map(|s| extract_pdf_standard(&s)).collect()
    } else {
        extract_pdf_standard(obj).map(|s| vec![s])
    }
}
