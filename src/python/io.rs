use pyo3::prelude::*;
use crate::io;

#[pyfunction]
/// Reads a FASTA file and returns its sequences.
///
/// This function provides a Python interface to read a FASTA file and obtain a list of sequences.
///
/// Args:
///     filename (str): The path to the FASTA file.
///     expect_unique_header (bool): Whether to expect unique headers in the file.
///     verbose (bool): Whether to enable verbose output.
///
/// Returns:
///     List[List[str]]: A list where each element is a [header, sequence] pair.
///
/// Raises:
///     Exception: If the file cannot be read or parsed.
pub fn read_fasta(
    filename: String,
    expect_unique_header: bool,
    verbose: bool,
) -> PyResult<Vec<Vec<String>>> {
    io::internal_parse_fasta_file(&filename, expect_unique_header, None, verbose)
        .map_err(|e| pyo3::exceptions::PyException::new_err(e))
}

#[pyfunction]
#[pyo3(signature = (fasta_data, filename, line_length = None, verbose = true, append_to_fasta = false))]
/// Writes sequences to a FASTA file.
///
/// This function provides a Python interface to write sequences to a FASTA file.
///
/// Args:
///     fasta_data (List[List[str]]): A list of [header, sequence] pairs to write.
///     filename (str): The output file path.
///     line_length (Optional[int]): Optional line length for wrapping sequences.
///     verbose (bool): Whether to enable verbose output.
///     append_to_fasta (bool): Whether to append to the file instead of overwriting.
///
/// Raises:
///     Exception: If the file cannot be written.
pub fn write_fasta(
    fasta_data: Vec<Vec<String>>,
    filename: &str,
    line_length: Option<usize>,
    verbose: bool,
    append_to_fasta: bool,
) -> PyResult<()> {
    io::write_fasta(fasta_data, filename, line_length, verbose, append_to_fasta)
        .map_err(|e| pyo3::exceptions::PyException::new_err(e))
}

/// Registers the IO functions with the Python module.
///
/// This function is called internally to add the `read_fasta` and `write_fasta` functions to the Python module.
///
/// Args:
///     _py (Python): The Python interpreter instance.
///     m (&PyModule): The module to register functions with.
///
/// Returns:
///     PyResult<()>: The result of the registration.
pub fn register(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_fasta, m)?)?;
    m.add_function(wrap_pyfunction!(write_fasta, m)?)?;
    Ok(())
}
