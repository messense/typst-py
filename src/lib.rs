use std::path::PathBuf;

use pyo3::exceptions::{PyIOError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use compiler::SystemWorld;

mod compiler;

fn resources_path(py: Python<'_>, package: &str) -> PyResult<PathBuf> {
    let resources = py.import("importlib.resources")?;
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
    world: compiler::SystemWorld,
}

impl Compiler {
    fn compile(&mut self, input: PathBuf) -> PyResult<Vec<u8>> {
        let buffer = self
            .world
            .compile(&input)
            .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))?;
        Ok(buffer)
    }
}

#[pymethods]
impl Compiler {
    /// Create a new typst compiler instance
    #[new]
    #[pyo3(signature = (root, font_paths = Vec::new()))]
    fn new(root: PathBuf, font_paths: Vec<PathBuf>) -> PyResult<Self> {
        let resource_path = Python::with_gil(|py| resources_path(py, "typst"))?;
        let mut default_fonts = Vec::new();
        for entry in walkdir::WalkDir::new(resource_path.join("fonts")) {
            let path = entry
                .map_err(|err| PyIOError::new_err(err.to_string()))?
                .into_path();
            let Some(extension) = path.extension() else { continue };
            if extension == "ttf" || extension == "otf" {
                default_fonts.push(path);
            }
        }
        let world = SystemWorld::new(root, &font_paths, &default_fonts);
        Ok(Self { world })
    }

    /// Compile a typst file to PDF
    #[pyo3(name = "compile", signature = (input, output = None))]
    fn py_compile(
        &mut self,
        py: Python<'_>,
        input: PathBuf,
        output: Option<PathBuf>,
    ) -> PyResult<PyObject> {
        let pdf_bytes = py.allow_threads(|| self.compile(input))?;
        if let Some(output) = output {
            std::fs::write(output, pdf_bytes)?;
            Ok(py.None())
        } else {
            Ok(PyBytes::new(py, &pdf_bytes).into())
        }
    }
}

/// Compile a typst document to PDF
#[pyfunction]
#[pyo3(signature = (input, output = None, root = None, font_paths = Vec::new()))]
fn compile(
    py: Python<'_>,
    input: PathBuf,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    font_paths: Vec<PathBuf>,
) -> PyResult<PyObject> {
    let root = if let Some(root) = root {
        root.canonicalize()?
    } else if let Some(dir) = input
        .canonicalize()
        .ok()
        .as_ref()
        .and_then(|path| path.parent())
    {
        dir.into()
    } else {
        PathBuf::new()
    };

    py.allow_threads(move || {
        // Create the world that serves sources, fonts and files.
        let mut compiler = Compiler::new(root, font_paths)?;
        let pdf_bytes = compiler.compile(input)?;
        if let Some(output) = output {
            std::fs::write(output, pdf_bytes)?;
            Ok(Python::with_gil(|py| py.None()))
        } else {
            Ok(Python::with_gil(|py| PyBytes::new(py, &pdf_bytes).into()))
        }
    })
}

/// Python binding to typst
#[pymodule]
fn _typst(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Compiler>()?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    Ok(())
}
