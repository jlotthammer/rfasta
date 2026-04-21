use pyo3::exceptions::PyException;
use pyo3::PyErr;

use crate::RfastaError;

mod io;
mod utilities;

pub fn to_py_err(error: RfastaError) -> PyErr {
    PyException::new_err(error.to_string())
}

pub fn register(py: pyo3::Python<'_>, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    utilities::register(py, m)?;
    io::register(py, m)?;
    Ok(())
}

#[cfg(all(test, feature = "python"))]
mod tests {
    use std::fs;

    use crate::python::io;
    use pyo3::Python;

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        dir.push(format!("{prefix}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn python_read_fasta_returns_rows() {
        let dir = unique_temp_dir("rfasta_py_read");
        let input = dir.join("input.fasta");
        fs::write(&input, ">seq1\nAAAA\n>seq2\ncccc\n").unwrap();

        let rows = Python::with_gil(|_| {
            io::read_fasta(input.to_string_lossy().to_string(), true, false).unwrap()
        });

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1][1], "CCCC");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn python_write_fasta_surfaces_structured_errors() {
        let dir = unique_temp_dir("rfasta_py_write");
        let output = dir.join("out.fasta");

        let error = Python::with_gil(|_| {
            io::write_fasta(
                vec![vec!["seq1".to_string()]],
                output.to_str().unwrap(),
                None,
                false,
                false,
            )
            .unwrap_err()
        });

        assert!(error.to_string().contains("help:"));
        fs::remove_dir_all(dir).unwrap();
    }
}
