use pyo3::prelude::*;

// Import modules
pub mod utilities;
pub mod configs;
pub mod io;

// Re-export everything that should be available to Rust users
pub use configs::*;
pub use utilities::*;
pub use io::*;

// Python module configuration
// #[cfg(feature = "python")]
#[pymodule]
fn pfasta_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    // Bind functions from the `utilities` module
    m.add_function(wrap_pyfunction!(utilities::build_custom_dictionary, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::convert_to_valid, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::check_sequence_is_valid, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::convert_invalid_sequences, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::remove_invalid_sequences, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::fail_on_invalid_sequences, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::convert_list_to_dictionary, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::fail_on_duplicates, m)?)?;
    m.add_function(wrap_pyfunction!(utilities::remove_duplicates, m)?)?;
    
    // Bind functions from the `io` module (if required for Python)
    m.add_function(wrap_pyfunction!(read_fasta, m)?)?;

    Ok(())
}

// Function to parse FASTA, bridging Rust and Python
// #[cfg(feature = "python")]
#[pyfunction]
fn read_fasta(
    filename: String,
    expect_unique_header: bool,
    verbose: bool,
) -> PyResult<Vec<Vec<String>>> {
    Ok(io::internal_parse_fasta_file(&filename, expect_unique_header, None, verbose))
}