//! FASTA writing APIs.
//!
//! The functions in this module write records to either an arbitrary writer or a filesystem path,
//! using buffered output and optional line wrapping.

pub use crate::io::{write_fasta_file, write_fasta_writer, WriteOptions};
