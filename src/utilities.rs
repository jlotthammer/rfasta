use std::collections::{HashMap, HashSet};

use crate::configs::{is_valid_residue, standard_conversion_map, standard_replacement};
use crate::errors::RfastaError;
use crate::io::FastaRecord;

/// Python-compatible correction dictionary type.
pub type CorrectionDictionary = HashMap<String, String>;

enum ConversionStrategy {
    Standard { alignment: bool },
    SingleChar(HashMap<char, String>),
    MultiPattern(Vec<(String, String)>),
}

impl ConversionStrategy {
    fn apply(&self, seq: &str) -> String {
        match self {
            Self::Standard { alignment } => {
                let mut converted = String::with_capacity(seq.len());
                for residue in seq.chars() {
                    if let Some(replacement) = standard_replacement(residue, *alignment) {
                        converted.push_str(replacement);
                    } else {
                        converted.push(residue.to_ascii_uppercase());
                    }
                }
                converted
            }
            Self::SingleChar(map) => {
                let mut converted = String::with_capacity(seq.len());
                for residue in seq.chars() {
                    let residue = residue.to_ascii_uppercase();
                    if let Some(replacement) = map.get(&residue) {
                        converted.push_str(replacement);
                    } else {
                        converted.push(residue);
                    }
                }
                converted
            }
            Self::MultiPattern(patterns) => {
                let mut converted = seq.to_ascii_uppercase();
                for (from, to) in patterns {
                    converted = converted.replace(from, to);
                }
                converted
            }
        }
    }
}

fn conversion_strategy(
    correction_dictionary: Option<CorrectionDictionary>,
    alignment: bool,
) -> ConversionStrategy {
    match correction_dictionary {
        Some(dictionary) => {
            if dictionary.keys().all(|key| key.chars().count() == 1) {
                let mut map = HashMap::with_capacity(dictionary.len());
                for (key, value) in dictionary {
                    let residue = key
                        .chars()
                        .next()
                        .expect("single-character dictionary keys are non-empty")
                        .to_ascii_uppercase();
                    map.insert(residue, value.to_ascii_uppercase());
                }
                ConversionStrategy::SingleChar(map)
            } else {
                ConversionStrategy::MultiPattern(
                    dictionary
                        .into_iter()
                        .map(|(from, to)| (from.to_ascii_uppercase(), to.to_ascii_uppercase()))
                        .collect(),
                )
            }
        }
        None => ConversionStrategy::Standard { alignment },
    }
}

/// Builds a correction dictionary by overlaying custom entries on the standard non-alignment map.
#[cfg_attr(not(feature = "python"), allow(dead_code))]
pub fn build_custom_dictionary(
    additional_dictionary: CorrectionDictionary,
) -> CorrectionDictionary {
    let mut final_dict = standard_conversion_map(false);
    for (key, value) in additional_dictionary {
        final_dict.insert(key, value);
    }
    final_dict
}

/// Converts a sequence according to either the default conversions or a custom correction dictionary.
#[cfg_attr(not(feature = "python"), allow(dead_code))]
pub fn convert_to_valid(
    seq: &str,
    alignment: bool,
    correction_dictionary: Option<CorrectionDictionary>,
) -> String {
    conversion_strategy(correction_dictionary, alignment).apply(seq)
}

/// Validates a protein sequence and returns the first invalid residue when present.
pub fn check_sequence_is_valid(seq: &str, alignment: bool) -> (bool, char) {
    for residue in seq.chars() {
        let residue = residue.to_ascii_uppercase();
        if !is_valid_residue(residue, alignment) {
            return (false, residue);
        }
    }
    (true, '0')
}

/// Converts invalid sequences in place and returns the number of changed records.
pub fn convert_invalid_sequences(
    mut dataset: Vec<FastaRecord>,
    correction_dictionary: Option<CorrectionDictionary>,
    alignment: bool,
) -> (Vec<FastaRecord>, usize) {
    let strategy = conversion_strategy(correction_dictionary, alignment);
    let mut converted_count = 0;

    for record in &mut dataset {
        let updated = strategy.apply(&record.sequence);
        if updated != record.sequence {
            converted_count += 1;
            record.sequence = updated;
        }
    }

    (dataset, converted_count)
}

/// Removes records that contain invalid residues.
pub fn remove_invalid_sequences(dataset: Vec<FastaRecord>, alignment: bool) -> Vec<FastaRecord> {
    dataset
        .into_iter()
        .filter(|record| check_sequence_is_valid(&record.sequence, alignment).0)
        .collect()
}

/// Fails on the first record that contains an invalid residue.
pub fn fail_on_invalid_sequences(
    sequences: &[FastaRecord],
    alignment: bool,
) -> Result<(), RfastaError> {
    for record in sequences {
        let (is_valid, invalid_char) = check_sequence_is_valid(&record.sequence, alignment);
        if !is_valid {
            return Err(RfastaError::InvalidSequence {
                header: record.header.clone(),
                invalid_char,
                alignment,
                hint: "Use InvalidSequenceAction::Convert, InvalidSequenceAction::ConvertRemove, or InvalidSequenceAction::Remove if you want rfasta to sanitize invalid residues.",
            });
        }
    }
    Ok(())
}

/// Converts a list of records into a dictionary keyed by header.
#[cfg_attr(not(feature = "python"), allow(dead_code))]
pub fn convert_records_to_dictionary(
    records: &[FastaRecord],
    verbose: bool,
) -> HashMap<String, String> {
    let mut return_dict = HashMap::new();
    let mut warning_count = 0;

    for record in records {
        if return_dict
            .insert(record.header.clone(), record.sequence.clone())
            .is_some()
        {
            warning_count += 1;
            if verbose {
                println!("[WARNING]: Overwriting entry [count = {warning_count}]");
            }
        }
    }

    if verbose {
        if warning_count > 0 {
            println!("[INFO]: If you want to avoid overwriting duplicate headers, request list-style output.");
        } else {
            println!("[INFO]: All processed sequences uniquely added to the returning dictionary");
        }
    }

    return_dict
}

/// Fails when an exact duplicate record appears more than once.
pub fn fail_on_duplicates(dataset: &[FastaRecord]) -> Result<(), RfastaError> {
    let mut seen: HashSet<(String, String)> = HashSet::with_capacity(dataset.len());
    for record in dataset {
        let key = (record.header.clone(), record.sequence.clone());
        if !seen.insert(key) {
            return Err(RfastaError::DuplicateRecord {
                header: record.header.clone(),
                hint: "Use DuplicateAction::Remove to keep the first occurrence, or enable unique headers during parsing if duplicates are unexpected.",
            });
        }
    }
    Ok(())
}

/// Removes duplicate records while keeping the first occurrence.
pub fn remove_duplicates(dataset: Vec<FastaRecord>) -> Vec<FastaRecord> {
    let mut seen: HashSet<(String, String)> = HashSet::with_capacity(dataset.len());
    let mut updated = Vec::with_capacity(dataset.len());

    for record in dataset {
        let key = (record.header.clone(), record.sequence.clone());
        if seen.insert(key) {
            updated.push(record);
        }
    }

    updated
}

/// Fails when the same sequence appears for multiple headers.
pub fn fail_on_duplicate_sequences(dataset: &[FastaRecord]) -> Result<(), RfastaError> {
    let mut sequences = HashMap::with_capacity(dataset.len());
    for record in dataset {
        if let Some(first_header) = sequences.insert(record.sequence.clone(), record.header.clone())
        {
            return Err(RfastaError::DuplicateSequence {
                first_header,
                duplicate_header: record.header.clone(),
                hint: "Use DuplicateAction::Remove to keep the first sequence occurrence, or DuplicateAction::Ignore to preserve all matching sequences.",
            });
        }
    }
    Ok(())
}

/// Removes duplicate sequences while keeping the first occurrence.
pub fn remove_duplicate_sequences(dataset: Vec<FastaRecord>) -> Vec<FastaRecord> {
    let mut seen = HashSet::with_capacity(dataset.len());
    let mut updated = Vec::with_capacity(dataset.len());

    for record in dataset {
        if seen.insert(record.sequence.clone()) {
            updated.push(record);
        }
    }

    updated
}
