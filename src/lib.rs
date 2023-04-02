use std::path::PathBuf;

use pyo3::prelude::*;

use compiler::SystemWorld;

mod compiler;

/// A typst compiler
#[pyclass(module = "typst._typst")]
pub struct Compiler {
    world: compiler::SystemWorld,
}

#[pymethods]
impl Compiler {
    /// Create a new typst compiler instance
    #[new]
    #[pyo3(signature = (root, font_paths = Vec::new()))]
    fn new(root: PathBuf, font_paths: Vec<PathBuf>) -> Self {
        let world = SystemWorld::new(root, &font_paths);
        Self { world }
    }

    /// Compile a typst file to PDF
    #[pyo3(signature = (input, output = None))]
    fn compile(&mut self, input: PathBuf, output: Option<PathBuf>) -> PyResult<()> {
        let output = match output {
            Some(path) => path,
            None => input.with_extension("pdf"),
        };
        let buffer = self.world.compile(&input).unwrap();
        std::fs::write(output, buffer)?;
        Ok(())
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
) -> PyResult<()> {
    let root = if let Some(root) = root {
        root
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
        let mut compiler = Compiler::new(root, font_paths);
        compiler.compile(input, output)
    })?;
    Ok(())
}

/// Python binding to typst
#[pymodule]
fn _typst(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Compiler>()?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    Ok(())
}
