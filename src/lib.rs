use std::path::PathBuf;

use pyo3::exceptions::{PyIOError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use std::collections::HashMap;
use typst::foundations::{Dict, Value};
use world::SystemWorld;

mod compiler;
mod download;
mod fonts;
mod package;
mod world;

fn resources_path(py: Python<'_>, package: &str) -> PyResult<PathBuf> {
    let resources = match py.import("importlib.resources") {
        Ok(module) => module,
        Err(_) => py.import("importlib_resources")?,
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
                    (err.get_type(py), err.value(py), err.traceback(py)),
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
    fn compile(&mut self, format: Option<&str>, ppi: Option<f32>) -> PyResult<Vec<u8>> {
        let buffer = self
            .world
            .compile(format, ppi)
            .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))?;
        Ok(buffer)
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
        let bytes = py.allow_threads(|| self.compile(format, ppi))?;
        if let Some(output) = output {
            std::fs::write(output, bytes)?;
            Ok(py.None())
        } else {
            Ok(PyBytes::new(py, &bytes).into())
        }
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

/// Python binding to typst
#[pymodule]
fn _typst(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Compiler>()?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    Ok(())
}
