//! FASTA sharding for large sequential workloads.
//!
//! The default sharding strategy is one-pass round-robin splitting, which keeps I/O sequential and
//! avoids rereading the source FASTA for UniRef-scale inputs.

pub use crate::io::split_fasta_file_round_robin;
