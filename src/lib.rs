//! `rfasta` is a Rust-native toolkit for parsing, cleaning, writing, and sharding protein FASTA
//! files.
//!
//! This crate keeps the public Rust API intentionally small:
//!
//! - [`parse`] for reading FASTA data
//! - [`clean`] for `protfasta`-style sequence cleanup policies
//! - [`write`] for FASTA output
//! - [`shard`] for one-pass round-robin sharding
//! - [`error`] for the common error type
//!
//! The full user guide is designed to live outside rustdoc as a guide-first docs site. Rustdoc is
//! the API reference layer, so the examples here stay short and module-focused.
//!
//! # Quick Start
//!
//! Parse from any buffered reader:
//!
//! ```
//! use std::io::Cursor;
//!
//! use rfasta::parse::{parse_fasta_reader, ParseOptions};
//!
//! let input = b">sp|P1|\nacde\n>sp|P2|\nTTTT\n";
//! let records = parse_fasta_reader(Cursor::new(input), ParseOptions::default())?;
//! assert_eq!(records.len(), 2);
//! assert_eq!(records[0].sequence, "ACDE");
//! # Ok::<(), rfasta::RfastaError>(())
//! ```
//!
//! Clean records using explicit policy types:
//!
//! ```
//! use rfasta::clean::{clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction};
//! use rfasta::parse::FastaRecord;
//!
//! let records = vec![
//!     FastaRecord::new("seq1", "ACDX"),
//!     FastaRecord::new("seq2", "ACD*"),
//! ];
//! let cleaned = clean_sequences(
//!     records,
//!     &CleanOptions {
//!         invalid_sequence_action: InvalidSequenceAction::ConvertRemove,
//!         duplicate_record_action: DuplicateAction::Fail,
//!         duplicate_sequence_action: DuplicateAction::Ignore,
//!         ..CleanOptions::default()
//!     },
//! )?;
//! assert_eq!(cleaned[0].sequence, "ACDG");
//! # Ok::<(), rfasta::RfastaError>(())
//! ```
//!
//! Write FASTA to any writer:
//!
//! ```
//! use rfasta::parse::FastaRecord;
//! use rfasta::write::{write_fasta_writer, WriteOptions};
//!
//! let records = vec![FastaRecord::new("seq1", "ACDEFGHIK")];
//! let mut output = Vec::new();
//! write_fasta_writer(&mut output, &records, &WriteOptions::default())?;
//! let text = String::from_utf8(output).unwrap();
//! assert!(text.starts_with(">seq1"));
//! # Ok::<(), rfasta::RfastaError>(())
//! ```
//!
//! Shard a large FASTA file in one pass:
//!
//! ```no_run
//! use rfasta::shard::split_fasta_file_round_robin;
//!
//! split_fasta_file_round_robin("uniref.fasta", "shards", 16, Some(60), true)?;
//! # Ok::<(), rfasta::RfastaError>(())
//! ```

#[cfg(feature = "python")]
use pyo3::prelude::*;

mod cli;
mod configs;
mod errors;
mod io;
mod sequence_processing;
mod utilities;

pub mod clean;
pub mod error;
pub mod parse;
pub mod shard;
pub mod write;

#[cfg(feature = "python")]
mod python;

pub use error::RfastaError;

#[doc(hidden)]
pub fn run_cli(args: &[String]) -> Result<(), RfastaError> {
    crate::cli::main(args)
}

#[cfg(feature = "python")]
#[pyfunction]
/// Runs the `rfasta` CLI using the current Python process arguments.
fn cli_main(py: Python) -> PyResult<()> {
    let sys = py.import("sys")?;
    let args: Vec<String> = sys.getattr("argv")?.extract()?;
    run_cli(&args).map_err(crate::python::to_py_err)
}

#[cfg(feature = "python")]
#[pymodule]
fn rfasta(_py: Python, m: &PyModule) -> PyResult<()> {
    python::register(_py, m)?;

    #[pyfn(m)]
    fn run_cli(args: Vec<String>) -> PyResult<()> {
        crate::run_cli(&args).map_err(crate::python::to_py_err)?;
        Ok(())
    }

    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    Ok(())
}
