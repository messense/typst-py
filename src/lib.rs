use std::path::PathBuf;

use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyString};

use query::{query as typst_query, QueryCommand, SerializationFormat};
use std::collections::HashMap;
use typst::foundations::{Dict, Value};
use world::SystemWorld;

mod compiler;
mod download;
mod fonts;
mod package;
mod query;
mod world;

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

fn resources_path(py: Python<'_>, package: &str) -> PyResult<PathBuf> {
    let resources = match py.import_bound("importlib.resources") {
        Ok(module) => module,
        Err(_) => py.import_bound("importlib_resources")?,
    };
    let files = resources.call_method1("files", (package,))?;
    let files = resources.call_method1("as_file", (files,))?;
    let path = files.call_method0("__enter__")?; // enter python context manager
    match path.extract() {
        Ok(path) => {
            let none = py.None();
            files.call_method1("__exit__", (&none, &none, &none))?;
            Ok(path)
        }
        Err(err) => {
            files
                .call_method1(
                    "__exit__",
                    (
                        err.get_type_bound(py),
                        err.value_bound(py),
                        err.traceback_bound(py),
                    ),
                )
                .unwrap();

            Err(err)
        }
    }
}

/// A typst compiler
#[pyclass(module = "typst._typst")]
pub struct Compiler {
    world: SystemWorld,
}

impl Compiler {
    fn compile(&mut self, format: Option<&str>, ppi: Option<f32>) -> PyResult<Vec<Vec<u8>>> {
        let buffer = self
            .world
            .compile(format, ppi)
            .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))?;
        Ok(buffer)
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
            Err(msg) => Err(PyRuntimeError::new_err(msg.to_string())),
        }
    }
}

#[pymethods]
impl Compiler {
    /// Create a new typst compiler instance
    #[new]
    #[pyo3(signature = (input, root = None, font_paths = Vec::new(), sys_inputs = HashMap::new()))]
    fn new(
        input: PathBuf,
        root: Option<PathBuf>,
        font_paths: Vec<PathBuf>,
        sys_inputs: HashMap<String, String>,
    ) -> PyResult<Self> {
        let input = input.canonicalize()?;
        let root = if let Some(root) = root {
            root.canonicalize()?
        } else if let Some(dir) = input.parent() {
            dir.into()
        } else {
            PathBuf::new()
        };
        let resource_path = Python::with_gil(|py| resources_path(py, "typst"))?;

        // Create the world that serves sources, fonts and files.
        let mut default_fonts = Vec::new();
        for entry in walkdir::WalkDir::new(resource_path.join("fonts")) {
            let path = entry
                .map_err(|err| PyIOError::new_err(err.to_string()))?
                .into_path();
            let Some(extension) = path.extension() else {
                continue;
            };
            if extension == "ttf" || extension == "otf" {
                default_fonts.push(path);
            }
        }
        let world = SystemWorld::builder(root, input)
            .inputs(Dict::from_iter(
                sys_inputs
                    .into_iter()
                    .map(|(k, v)| (k.into(), Value::Str(v.into()))),
            ))
            .font_paths(font_paths)
            .font_files(default_fonts)
            .build()
            .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))?;
        Ok(Self { world })
    }

    /// Compile a typst file to PDF
    #[pyo3(name = "compile", signature = (output = None, format = None, ppi = None))]
    fn py_compile(
        &mut self,
        py: Python<'_>,
        output: Option<PathBuf>,
        format: Option<&str>,
        ppi: Option<f32>,
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
            let buffers = py.allow_threads(|| self.compile(format, ppi))?;

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
            let buffers = py.allow_threads(|| self.compile(format, ppi))?;
            if buffers.len() == 1 {
                // Return a single buffer as a single byte string
                Ok(PyBytes::new_bound(py, &buffers[0]).into())
            } else {
                let list = PyList::empty_bound(py);
                for buffer in buffers {
                    list.append(PyBytes::new_bound(py, &buffer))?;
                }
                Ok(list.into())
            }
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
            .map(|s| PyString::new_bound(py, &s).into())
    }
}

/// Compile a typst document to PDF
#[pyfunction]
#[pyo3(signature = (input, output = None, root = None, font_paths = Vec::new(), format = None, ppi = None, sys_inputs = HashMap::new()))]
#[allow(clippy::too_many_arguments)]
fn compile(
    py: Python<'_>,
    input: PathBuf,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    font_paths: Vec<PathBuf>,
    format: Option<&str>,
    ppi: Option<f32>,
    sys_inputs: HashMap<String, String>,
) -> PyResult<PyObject> {
    let mut compiler = Compiler::new(input, root, font_paths, sys_inputs)?;
    compiler.py_compile(py, output, format, ppi)
}

/// Query a typst document
#[pyfunction]
#[pyo3(name = "query", signature = (input, selector, field = None, one = false, format = None, root = None, font_paths = Vec::new(), sys_inputs = HashMap::new()))]
#[allow(clippy::too_many_arguments)]
fn py_query(
    py: Python<'_>,
    input: PathBuf,
    selector: &str,
    field: Option<&str>,
    one: bool,
    format: Option<&str>,
    root: Option<PathBuf>,
    font_paths: Vec<PathBuf>,
    sys_inputs: HashMap<String, String>,
) -> PyResult<PyObject> {
    let mut compiler = Compiler::new(input, root, font_paths, sys_inputs)?;
    compiler.py_query(py, selector, field, one, format)
}

/// Python binding to typst
#[pymodule]
fn _typst(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Compiler>()?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(py_query, m)?)?;
    Ok(())
}
