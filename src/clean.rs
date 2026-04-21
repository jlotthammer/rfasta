//! FASTA cleaning and validation policies.
//!
//! This module contains the user-facing cleaning API: duplicate handling, invalid-sequence
//! behavior, and record filtering. It intentionally exposes the policy types and the top-level
//! cleaning entry point, while lower-level residue conversion helpers remain internal.

pub use crate::sequence_processing::{
    clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction,
};
