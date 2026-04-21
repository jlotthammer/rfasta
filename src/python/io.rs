use pyo3::prelude::*;

use crate::io::{parse_fasta_file, write_fasta_file, FastaRecord, ParseOptions, WriteOptions};

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
#[pyo3(signature = (filename, expect_unique_header = true, verbose = false))]
/// Reads a FASTA file and returns a list of `[header, sequence]` pairs.
pub fn read_fasta(
    filename: String,
    expect_unique_header: bool,
    verbose: bool,
) -> PyResult<Vec<Vec<String>>> {
    parse_fasta_file(
        &filename,
        ParseOptions {
            expect_unique_header,
        },
        verbose,
    )
    .map(records_to_rows)
    .map_err(crate::python::to_py_err)
}

#[pyfunction]
#[pyo3(signature = (fasta_data, filename, line_length = None, verbose = true, append_to_fasta = false))]
/// Writes `[header, sequence]` pairs to a FASTA file.
pub fn write_fasta(
    fasta_data: Vec<Vec<String>>,
    filename: &str,
    line_length: Option<usize>,
    verbose: bool,
    append_to_fasta: bool,
) -> PyResult<()> {
    let records = rows_to_records(fasta_data)?;
    write_fasta_file(
        &records,
        filename,
        WriteOptions {
            line_length,
            append: append_to_fasta,
        },
        verbose,
    )
    .map_err(crate::python::to_py_err)
}

pub fn register(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_fasta, m)?)?;
    m.add_function(wrap_pyfunction!(write_fasta, m)?)?;
    Ok(())
}
