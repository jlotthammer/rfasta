use std::collections::HashMap;

use clap::ValueEnum;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::errors::RfastaError;
use crate::io::FastaRecord;
use crate::utilities;

/// Action for duplicate record or duplicate sequence handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum DuplicateAction {
    /// Preserve all matching records.
    Ignore,
    /// Return an error when a duplicate is encountered.
    Fail,
    /// Keep the first occurrence and remove later duplicates.
    Remove,
}

/// Action for invalid sequence handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum InvalidSequenceAction {
    /// Preserve sequences even if they contain invalid residues.
    Ignore,
    /// Return an error when invalid residues are present.
    Fail,
    /// Drop sequences that still contain invalid residues.
    Remove,
    /// Convert known residues, then fail if anything invalid remains.
    Convert,
    /// Convert known residues and keep any sequences that still contain invalid residues.
    #[value(name = "convert-ignore")]
    ConvertIgnore,
    /// Convert known residues, then remove any sequences that still contain invalid residues.
    #[value(name = "convert-remove")]
    ConvertRemove,
}

/// Options used to clean parsed FASTA records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanOptions {
    /// Policy for invalid residue handling.
    pub invalid_sequence_action: InvalidSequenceAction,
    /// Policy for exact duplicate record handling.
    pub duplicate_record_action: DuplicateAction,
    /// Policy for duplicate sequence handling across headers.
    pub duplicate_sequence_action: DuplicateAction,
    /// Minimum sequence length to keep.
    pub shortest_seq: Option<usize>,
    /// Maximum sequence length to keep.
    pub longest_seq: Option<usize>,
    /// Number of records to retain after shuffling.
    pub random_subsample: Option<usize>,
    /// Replace commas in headers with semicolons.
    pub remove_comma_from_header: bool,
    /// Allow `-` as a valid residue for aligned sequences.
    pub alignment: bool,
    /// Emit informational output while cleaning.
    pub verbose: bool,
    /// Optional custom residue conversion dictionary.
    pub correction_dictionary: Option<HashMap<String, String>>,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            invalid_sequence_action: InvalidSequenceAction::Fail,
            duplicate_record_action: DuplicateAction::Fail,
            duplicate_sequence_action: DuplicateAction::Ignore,
            shortest_seq: None,
            longest_seq: None,
            random_subsample: None,
            remove_comma_from_header: false,
            alignment: false,
            verbose: false,
            correction_dictionary: None,
        }
    }
}

fn deal_with_invalid_sequences(
    data: Vec<FastaRecord>,
    options: &CleanOptions,
) -> Result<Vec<FastaRecord>, RfastaError> {
    match options.invalid_sequence_action {
        InvalidSequenceAction::Fail => {
            utilities::fail_on_invalid_sequences(&data, options.alignment)?;
            Ok(data)
        }
        InvalidSequenceAction::Remove => {
            let original_len = data.len();
            let updated = utilities::remove_invalid_sequences(data, options.alignment);
            if options.verbose {
                println!(
                    "[INFO]: Removed {} of {} sequences due to invalid characters",
                    original_len - updated.len(),
                    original_len
                );
            }
            Ok(updated)
        }
        InvalidSequenceAction::Convert | InvalidSequenceAction::ConvertIgnore => {
            let (updated, count) = utilities::convert_invalid_sequences(
                data,
                options.correction_dictionary.clone(),
                options.alignment,
            );
            if options.verbose {
                println!("[INFO]: Converted {count} sequences to valid sequences");
            }
            if matches!(
                options.invalid_sequence_action,
                InvalidSequenceAction::Convert
            ) {
                utilities::fail_on_invalid_sequences(&updated, options.alignment)?;
            }
            Ok(updated)
        }
        InvalidSequenceAction::ConvertRemove => {
            let (updated, count) = utilities::convert_invalid_sequences(
                data,
                options.correction_dictionary.clone(),
                options.alignment,
            );
            if options.verbose {
                println!("[INFO]: Converted {count} sequences to valid sequences");
            }
            let original_len = updated.len();
            let filtered = utilities::remove_invalid_sequences(updated, options.alignment);
            if options.verbose {
                println!(
                    "[INFO]: Removed {} of {} sequences due to invalid characters",
                    original_len - filtered.len(),
                    original_len
                );
            }
            Ok(filtered)
        }
        InvalidSequenceAction::Ignore => Ok(data),
    }
}

fn deal_with_duplicate_records(
    data: Vec<FastaRecord>,
    action: DuplicateAction,
    verbose: bool,
) -> Result<Vec<FastaRecord>, RfastaError> {
    match action {
        DuplicateAction::Ignore => Ok(data),
        DuplicateAction::Fail => {
            utilities::fail_on_duplicates(&data)?;
            Ok(data)
        }
        DuplicateAction::Remove => {
            let original_len = data.len();
            let updated = utilities::remove_duplicates(data);
            if verbose {
                println!(
                    "[INFO]: Removed {} of {} sequences due to duplicate records",
                    original_len - updated.len(),
                    original_len
                );
            }
            Ok(updated)
        }
    }
}

fn deal_with_duplicate_sequences(
    data: Vec<FastaRecord>,
    action: DuplicateAction,
    verbose: bool,
) -> Result<Vec<FastaRecord>, RfastaError> {
    match action {
        DuplicateAction::Ignore => Ok(data),
        DuplicateAction::Fail => {
            utilities::fail_on_duplicate_sequences(&data)?;
            Ok(data)
        }
        DuplicateAction::Remove => {
            let original_len = data.len();
            let updated = utilities::remove_duplicate_sequences(data);
            if verbose {
                println!(
                    "[INFO]: Removed {} of {} sequences due to duplicate sequences",
                    original_len - updated.len(),
                    original_len
                );
            }
            Ok(updated)
        }
    }
}

fn clean_sequences_with_rng(
    data: Vec<FastaRecord>,
    options: &CleanOptions,
    rng: &mut StdRng,
) -> Result<Vec<FastaRecord>, RfastaError> {
    let mut processed = data;
    processed =
        deal_with_duplicate_records(processed, options.duplicate_record_action, options.verbose)?;
    processed = deal_with_duplicate_sequences(
        processed,
        options.duplicate_sequence_action,
        options.verbose,
    )?;
    processed = deal_with_invalid_sequences(processed, options)?;

    if let Some(min_len) = options.shortest_seq {
        processed.retain(|record| record.sequence.len() >= min_len);
    }
    if let Some(max_len) = options.longest_seq {
        processed.retain(|record| record.sequence.len() <= max_len);
    }
    if let Some(sample_size) = options.random_subsample {
        processed.shuffle(rng);
        processed.truncate(sample_size);
    }
    if options.remove_comma_from_header {
        for record in &mut processed {
            record.header = record.header.replace(',', ";");
        }
    }

    Ok(processed)
}

/// Cleans FASTA records using the provided options.
///
/// Duplicate-record handling follows `protfasta`: an exact duplicate record means same header and
/// same sequence. Duplicate headers are handled separately by the parser when
/// `ParseOptions::expect_unique_header` is enabled.
pub fn clean_sequences(
    data: Vec<FastaRecord>,
    options: &CleanOptions,
) -> Result<Vec<FastaRecord>, RfastaError> {
    let mut rng = StdRng::from_entropy();
    clean_sequences_with_rng(data, options, &mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_subsample_is_stable() {
        let records = vec![
            FastaRecord::new("a", "AAAA"),
            FastaRecord::new("b", "CCCC"),
            FastaRecord::new("c", "DDDD"),
        ];
        let options = CleanOptions {
            random_subsample: Some(2),
            ..CleanOptions::default()
        };

        let mut rng = StdRng::seed_from_u64(42);
        let cleaned = clean_sequences_with_rng(records, &options, &mut rng).unwrap();
        assert_eq!(cleaned.len(), 2);
    }
}
