use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use crate::utilities;

// Wraps utilities::build_custom_dictionary for Python
#[pyfunction]
pub fn build_custom_dictionary(additional_dictionary: &PyDict) -> PyResult<HashMap<String, String>> {
    let mut rust_dict = HashMap::new();
    for (key, value) in additional_dictionary {
        let key_str: String = key.extract()?;
        let value_str: String = value.extract()?;
        rust_dict.insert(key_str, value_str);
    }
    let final_dict = utilities::build_custom_dictionary(rust_dict);
    Ok(final_dict)
}

// Wraps utilities::convert_to_valid for Python
#[pyfunction]
#[pyo3(signature = (seq, alignment = false, correction_dictionary = None))]
pub fn convert_to_valid(seq: &str, alignment: bool, correction_dictionary: Option<HashMap<String, String>>) -> PyResult<String> {
    Ok(utilities::convert_to_valid(seq, alignment, correction_dictionary))
}

// Wraps utilities::check_sequence_is_valid for Python
#[pyfunction]
pub fn check_sequence_is_valid(seq: &str, alignment: bool) -> PyResult<(bool, char)> {
    Ok(utilities::check_sequence_is_valid(seq, alignment))
}

// Wraps utilities::convert_invalid_sequences for Python
#[pyfunction]
#[pyo3(signature = (dataset, correction_dictionary = None, alignment = false))]
pub fn convert_invalid_sequences(
    dataset: Vec<Vec<String>>,
    correction_dictionary: Option<HashMap<String, String>>,
    alignment: bool,
) -> PyResult<(Vec<Vec<String>>, usize)> {
    Ok(utilities::convert_invalid_sequences(dataset, correction_dictionary, alignment))
}

// Wraps utilities::remove_invalid_sequences for Python
#[pyfunction]
pub fn remove_invalid_sequences(
    dataset: Vec<Vec<String>>,
    alignment: bool,
) -> PyResult<Vec<Vec<String>>> {
    Ok(utilities::remove_invalid_sequences(dataset, alignment))
}

// Wraps utilities::fail_on_invalid_sequences for Python
#[pyfunction]
pub fn fail_on_invalid_sequences(sequences: Vec<Vec<String>>, alignment: bool) -> PyResult<()> {
    utilities::fail_on_invalid_sequences(sequences, alignment).map_err(|e| pyo3::exceptions::PyException::new_err(e))
}

// Wraps utilities::convert_list_to_dictionary for Python
#[pyfunction]
pub fn convert_list_to_dictionary(raw_list: Vec<Vec<String>>, verbose: bool) -> PyResult<HashMap<String, String>> {
    Ok(utilities::convert_list_to_dictionary(raw_list, verbose))
}

// Wraps utilities::fail_on_duplicates for Python
#[pyfunction]
pub fn fail_on_duplicates(dataset: Vec<(String, String)>) -> PyResult<()> {
    utilities::fail_on_duplicates(dataset).map_err(|e| pyo3::exceptions::PyException::new_err(e))
}

// Wraps utilities::remove_duplicates for Python
#[pyfunction]
pub fn remove_duplicates(dataset: Vec<(String, String)>) -> PyResult<Vec<(String, String)>> {
    Ok(utilities::remove_duplicates(dataset))
}

// Registers the functions with the Python module
pub fn register(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(build_custom_dictionary, m)?)?;
    m.add_function(wrap_pyfunction!(convert_to_valid, m)?)?;
    m.add_function(wrap_pyfunction!(check_sequence_is_valid, m)?)?;
    m.add_function(wrap_pyfunction!(convert_invalid_sequences, m)?)?;
    m.add_function(wrap_pyfunction!(remove_invalid_sequences, m)?)?;
    m.add_function(wrap_pyfunction!(fail_on_invalid_sequences, m)?)?;
    m.add_function(wrap_pyfunction!(convert_list_to_dictionary, m)?)?;
    m.add_function(wrap_pyfunction!(fail_on_duplicates, m)?)?;
    m.add_function(wrap_pyfunction!(remove_duplicates, m)?)?;
    Ok(())
}
