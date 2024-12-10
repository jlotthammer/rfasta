#[cfg(feature = "python")]
use pyo3::prelude::*;

// Import modules
pub mod utilities;
pub mod configs;
pub mod io;
pub mod cli;
pub mod sequence_processing;

#[cfg(feature = "python")]
pub mod python;

// Re-export everything that should be available to Rust users
pub use configs::*;
pub use utilities::*;
pub use io::*;
pub use cli::*;

#[cfg(feature = "python")]
#[pyfunction]
/// Runs the rfasta CLI with the given arguments.
///
/// This function acts as a wrapper around the `cli::main` function. It imports the arguments from
/// the Python `sys.argv` and passes them to the CLI main function.
///
/// # Arguments
///
/// * `py` - The Python interpreter.
///
/// # Returns
///
/// * `PyResult<()>` - Returns `Ok(())` if successful, or a `PyRuntimeError` if an error occurs.
fn cli_main(py: Python) -> PyResult<()> {
    let sys = py.import("sys")?;
    let args: Vec<String> = sys.getattr("argv")?.extract()?;

    crate::cli::main(&args).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

#[cfg(feature = "python")]
#[pymodule]
/// The `rfasta` Python module.
///
/// This module provides Python bindings for the rfasta library, allowing access to Rust functions
/// from Python code.
fn rfasta(_py: Python, m: &PyModule) -> PyResult<()> {
    python::utilities::register(_py, m)?;
    python::io::register(_py, m)?;
    
    // Update CLI function to use new module path
    #[pyfn(m)]
    fn run_cli(args: Vec<String>) -> PyResult<()> {
        crate::cli::main(&args).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
        Ok(())
    }
    
    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    Ok(())
}