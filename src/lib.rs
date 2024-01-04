use std::cell::RefCell;
use std::path::PathBuf;

use pyo3::exceptions::{PyIOError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

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

/// Create a new world.
#[pyfunction]
#[pyo3(signature = (input, output = None, root = None, font_paths = Vec::new(), format = None, ppi = None))]
fn compile(
    py: Python<'_>,
    input: PathBuf,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
    font_paths: Vec<PathBuf>,
    format: Option<&str>,
    ppi: Option<f32>,
) -> PyResult<PyObject> {
    // static world for each thread
    thread_local! {
        static WORLD: RefCell<Option<SystemWorld>> = RefCell::new(None);
    }

    py.allow_threads(move || {
        // canonicalize the input path and root path
        let input = input.canonicalize()?;
        let root = if let Some(root) = root {
            root.canonicalize()?
        } else if let Some(dir) = input.parent() {
            dir.into()
        } else {
            PathBuf::new()
        };
        // if the world is not initialized or the input path, root path is changed,
        // reinitialize the world
        if WORLD.with(|world| {
            world.borrow().is_none()
                || root.as_path() != world.borrow().as_ref().unwrap().root()
                || input.as_path() != world.borrow().as_ref().unwrap().input().as_path()
        }) {
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

            let _ = WORLD.with(|world| {
                let mut world = world.borrow_mut();
                let sys_world = SystemWorld::builder(root, input)
                    .font_paths(font_paths)
                    .font_files(default_fonts)
                    .build()
                    .map_err(|msg| PyRuntimeError::new_err(msg.to_string()));
                match sys_world {
                    Ok(sys_world) => {
                        *world = Some(sys_world);
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            });
        }
        let bytes = WORLD.with(|world| {
            let mut world = world.borrow_mut();
            world
                .as_mut()
                .unwrap()
                .compile(format, ppi)
                .map_err(|msg| PyRuntimeError::new_err(msg.to_string()))
        });
        match bytes {
            Ok(bytes) => {
                if let Some(output) = output {
                    std::fs::write(output, bytes)?;
                    Ok(Python::with_gil(|py| py.None()))
                } else {
                    Ok(Python::with_gil(|py| PyBytes::new(py, &bytes).into()))
                }
            }
            Err(err) => Err(err),
        }
    })
}

/// Python binding to typst
#[pymodule]
fn _typst(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    Ok(())
}
