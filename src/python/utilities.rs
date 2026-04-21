use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::io::FastaRecord;
use crate::utilities;

fn rows_to_records(rows: Vec<Vec<String>>) -> PyResult<Vec<FastaRecord>> {
    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() != 2 {
            return Err(pyo3::exceptions::PyException::new_err(
                crate::RfastaError::invalid_record(
                    format!(
                        "expected each FASTA entry to contain exactly two elements, found {}",
                        row.len()
                    ),
                    "Pass a list like [[header, sequence], ...].",
                )
                .to_string(),
            ));
        }
        let mut row = row;
        let sequence = row.pop().expect("validated row length");
        let header = row.pop().expect("validated row length");
        records.push(FastaRecord::new(header, sequence));
    }
    Ok(records)
}

fn records_to_rows(records: Vec<FastaRecord>) -> Vec<Vec<String>> {
    records
        .into_iter()
        .map(|record| vec![record.header, record.sequence])
        .collect()
}

#[pyfunction]
pub fn build_custom_dictionary(
    additional_dictionary: &PyDict,
) -> PyResult<HashMap<String, String>> {
    let mut rust_dict = HashMap::new();
    for (key, value) in additional_dictionary {
        rust_dict.insert(key.extract()?, value.extract()?);
    }
    Ok(utilities::build_custom_dictionary(rust_dict))
}

#[pyfunction]
#[pyo3(signature = (seq, alignment = false, correction_dictionary = None))]
pub fn convert_to_valid(
    seq: &str,
    alignment: bool,
    correction_dictionary: Option<HashMap<String, String>>,
) -> PyResult<String> {
    Ok(utilities::convert_to_valid(
        seq,
        alignment,
        correction_dictionary,
    ))
}

#[pyfunction]
#[pyo3(signature = (seq, alignment = false))]
pub fn check_sequence_is_valid(seq: &str, alignment: bool) -> PyResult<(bool, char)> {
    Ok(utilities::check_sequence_is_valid(seq, alignment))
}

#[pyfunction]
#[pyo3(signature = (dataset, correction_dictionary = None, alignment = false))]
pub fn convert_invalid_sequences(
    dataset: Vec<Vec<String>>,
    correction_dictionary: Option<HashMap<String, String>>,
    alignment: bool,
) -> PyResult<(Vec<Vec<String>>, usize)> {
    let records = rows_to_records(dataset)?;
    let (converted, count) =
        utilities::convert_invalid_sequences(records, correction_dictionary, alignment);
    Ok((records_to_rows(converted), count))
}

#[pyfunction]
#[pyo3(signature = (dataset, alignment = false))]
pub fn remove_invalid_sequences(
    dataset: Vec<Vec<String>>,
    alignment: bool,
) -> PyResult<Vec<Vec<String>>> {
    let records = rows_to_records(dataset)?;
    Ok(records_to_rows(utilities::remove_invalid_sequences(
        records, alignment,
    )))
}

#[pyfunction]
#[pyo3(signature = (sequences, alignment = false))]
pub fn fail_on_invalid_sequences(sequences: Vec<Vec<String>>, alignment: bool) -> PyResult<()> {
    let records = rows_to_records(sequences)?;
    utilities::fail_on_invalid_sequences(&records, alignment).map_err(crate::python::to_py_err)
}

#[pyfunction]
pub fn convert_list_to_dictionary(
    raw_list: Vec<Vec<String>>,
    verbose: bool,
) -> PyResult<HashMap<String, String>> {
    let records = rows_to_records(raw_list)?;
    Ok(utilities::convert_records_to_dictionary(&records, verbose))
}

#[pyfunction]
pub fn fail_on_duplicates(dataset: Vec<(String, String)>) -> PyResult<()> {
    let records = dataset
        .into_iter()
        .map(|(header, sequence)| FastaRecord::new(header, sequence))
        .collect::<Vec<_>>();
    utilities::fail_on_duplicates(&records).map_err(crate::python::to_py_err)
}

#[pyfunction]
pub fn remove_duplicates(dataset: Vec<(String, String)>) -> PyResult<Vec<(String, String)>> {
    let records = dataset
        .into_iter()
        .map(|(header, sequence)| FastaRecord::new(header, sequence))
        .collect::<Vec<_>>();
    Ok(utilities::remove_duplicates(records)
        .into_iter()
        .map(|record| (record.header, record.sequence))
        .collect())
}

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
