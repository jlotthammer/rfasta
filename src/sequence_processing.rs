use std::collections::HashMap;
use rand::seq::SliceRandom;
use crate::utilities;

/// Deals with invalid sequences based on the specified action.
pub fn deal_with_invalid_sequences(
    data: Vec<Vec<String>>,
    invalid_sequence_action: &str,
    alignment: bool,
    verbose: bool,
    correction_dictionary: Option<HashMap<String, String>>,
) -> Result<Vec<Vec<String>>, String> {
    match invalid_sequence_action {
        "fail" => {
            utilities::fail_on_invalid_sequences(data.clone(), alignment)?;
            Ok(data)
        },
        "remove" => {
            let data_len = data.len();
            let updated = utilities::remove_invalid_sequences(data, alignment);
            if verbose {
                println!(
                    "[INFO]: Removed {} of {} sequences due to invalid characters",
                    data_len - updated.len(),
                    data_len
                );
            }
            Ok(updated)
        },
        "convert" | "convert-ignore" => {
            let (updated, count) = utilities::convert_invalid_sequences(
                data,
                correction_dictionary,
                alignment,
            );
            if verbose {
                println!("[INFO]: Converted {} sequences to valid sequences", count);
            }
            if invalid_sequence_action == "convert" {
                utilities::fail_on_invalid_sequences(updated.clone(), alignment)?;
            }
            Ok(updated)
        },
        "ignore" => Ok(data),
        _ => Err(format!(
            "Invalid option for 'invalid_sequence_action': {}",
            invalid_sequence_action
        )),
    }
}

/// Deals with duplicate records based on the specified action.
pub fn deal_with_duplicate_records(
    data: Vec<Vec<String>>,
    duplicate_record_action: &str,
    verbose: bool,
) -> Result<Vec<Vec<String>>, String> {
    match duplicate_record_action {
        "ignore" => Ok(data),
        "fail" => {
            utilities::fail_on_duplicate_records(data.clone())?;
            Ok(data)
        },
        "remove" => {
            let data_len = data.len();
            let updated = utilities::remove_duplicate_records(data);
            if verbose {
                println!(
                    "[INFO]: Removed {} of {} sequences due to duplicate records",
                    data_len - updated.len(),
                    data_len
                );
            }
            Ok(updated)
        },
        _ => Err(format!(
            "Invalid option for 'duplicate_record_action': {}",
            duplicate_record_action
        )),
    }
}

/// Deals with duplicate sequences based on the specified action.
pub fn deal_with_duplicate_sequences(
    data: Vec<Vec<String>>,
    duplicate_sequence_action: &str,
    verbose: bool,
) -> Result<Vec<Vec<String>>, String> {
    match duplicate_sequence_action {
        "ignore" => Ok(data),
        "fail" => {
            utilities::fail_on_duplicate_sequences(data.clone())?;
            Ok(data)
        },
        "remove" => {
            let data_len = data.len();
            let updated = utilities::remove_duplicate_sequences(data);
            if verbose {
                println!(
                    "[INFO]: Removed {} of {} sequences due to duplicate sequences",
                    data_len - updated.len(),
                    data_len
                );
            }
            Ok(updated)
        },
        _ => Err(format!(
            "Invalid option for 'duplicate_sequence_action': {}",
            duplicate_sequence_action
        )),
    }
}

/// Processes sequences based on provided arguments.
///
/// Applies filters and transformations to the sequence data, such as handling invalid sequences,
/// filtering by length, random subsampling, and header processing.
///
/// # Arguments
///
/// * `data` - A vector of [header, sequence] pairs to process.
/// * `invalid_sequence_action` - Action to take on invalid sequences ("ignore", "fail", "remove", etc.).
/// * `duplicate_record_action` - Action to take on duplicate records.
/// * `duplicate_sequence_action` - Action to take on duplicate sequences.
/// * `shortest_seq` - Optional minimum sequence length to include.
/// * `longest_seq` - Optional maximum sequence length to include.
/// * `random_subsample` - Optional number of sequences to randomly select.
/// * `remove_comma_from_header` - Whether to replace commas with semicolons in headers.
/// * `alignment` - Whether sequences should be considered aligned.
/// * `verbose` - Enable verbose output.
///
/// # Returns
///
/// * `Ok(Vec<Vec<String>>)` - The cleaned sequences.
/// * `Err(String)` - An error message if cleaning fails.
pub fn clean_sequences(
    data: Vec<Vec<String>>,
    invalid_sequence_action: &str,
    duplicate_record_action: &str,
    duplicate_sequence_action: &str,
    shortest_seq: &Option<usize>,
    longest_seq: &Option<usize>,
    random_subsample: &Option<usize>,
    remove_comma_from_header: bool,
    alignment: bool,
    verbose: bool,
) -> Result<Vec<Vec<String>>, String> {
    let mut processed = data;

    // Deal with invalid sequences
    processed = deal_with_invalid_sequences(
        processed,
        invalid_sequence_action,
        alignment,
        verbose,
        None,
    )?;

    // Deal with duplicate records
    processed = deal_with_duplicate_records(
        processed,
        duplicate_record_action,
        verbose,
    )?;

    // Deal with duplicate sequences
    processed = deal_with_duplicate_sequences(
        processed,
        duplicate_sequence_action,
        verbose,
    )?;

    // Apply sequence length filters
    if let Some(min_len) = shortest_seq {
        processed.retain(|seq| seq[1].len() >= *min_len);
    }
    if let Some(max_len) = longest_seq {
        processed.retain(|seq| seq[1].len() <= *max_len);
    }

    // Handle random subsampling
    if let Some(sample_size) = random_subsample {
        let mut rng = rand::thread_rng();
        processed.shuffle(&mut rng);
        processed.truncate(*sample_size);
    }

    // Process headers if needed
    if remove_comma_from_header {
        for entry in &mut processed {
            entry[0] = entry[0].replace(',', ";");
        }
    }

    Ok(processed)
}