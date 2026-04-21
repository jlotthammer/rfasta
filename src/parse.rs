//! Streaming and in-memory FASTA parsing.
//!
//! Use this module when you need to read protein FASTA records from a file or from any buffered
//! reader. The streaming visitors are the right default for large files; the collection helpers are
//! more convenient for workflows that need the full dataset in memory.

pub use crate::io::{
    parse_fasta_file, parse_fasta_reader, visit_fasta_file, visit_fasta_reader, FastaRecord,
    ParseOptions,
};
