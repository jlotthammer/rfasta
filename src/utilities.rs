use std::collections::HashMap;
use std::collections::HashSet;
use std::vec::Vec;

use crate::configs::{get_standard_aas, get_standard_aas_with_gap, get_standard_conversion, get_standard_conversion_with_gap};

pub fn build_custom_dictionary(additional_dictionary: HashMap<String, String>) -> HashMap<String, String> {
    let mut final_dict: HashMap<String, String> = get_standard_conversion()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    for (key, value) in additional_dictionary {
        final_dict.insert(key, value);
    }
    final_dict
}

pub fn convert_to_valid(seq: &str, alignment: bool, correction_dictionary: Option<HashMap<String, String>>) -> String {
    let converter: HashMap<String, String> = match correction_dictionary {
        Some(dict) => dict,
        None => {
            if alignment {
                get_standard_conversion_with_gap()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            } else {
                get_standard_conversion()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            }
        }
    };
    let mut result = String::from(seq);
    for (key, value) in converter.iter() {
        result = result.replace(key, value);
    }
    result
}

pub fn check_sequence_is_valid(seq: &str, alignment: bool) -> (bool, char) {
    let standard_aas = get_standard_aas();
    let standard_aas_with_gap = get_standard_aas_with_gap();
    let valid_aa_list = if alignment {
        &standard_aas_with_gap
    } else {
        &standard_aas
    };
    let seq_chars: HashSet<char> = seq.chars().collect();
    for &c in &seq_chars {
        if !valid_aa_list.contains(&c) {
            return (false, c);
        }
    }
    (true, '0')
}

pub fn convert_invalid_sequences(
    mut dataset: Vec<Vec<String>>,
    correction_dictionary: Option<HashMap<String, String>>,
    alignment: bool,
) -> (Vec<Vec<String>>, usize) {
    let mut count = 0;
    for row in &mut dataset {
        let s = row[1].clone();
        let corrected = convert_to_valid(&s, alignment, correction_dictionary.clone());
        row[1] = corrected.clone();
        if s != corrected {
            count += 1;
        }
    }
    (dataset, count)
}

pub fn remove_invalid_sequences(
    dataset: Vec<Vec<String>>,
    alignment: bool,
) -> Vec<Vec<String>> {
    dataset
        .into_iter()
        .filter(|element| {
            let (is_valid, _) = check_sequence_is_valid(&element[1], alignment);
            is_valid
        })
        .collect()
}

pub fn fail_on_invalid_sequences(sequences: Vec<Vec<String>>, alignment: bool) -> Result<(), String> {
    for sequence in &sequences {
        let (is_valid, invalid_char) = check_sequence_is_valid(&sequence[1], alignment);
        if !is_valid {
            return Err(format!(
                "Invalid character '{}' found in sequence: {}",
                invalid_char, sequence[0]
            ));
        }
    }
    Ok(())
}

pub fn convert_list_to_dictionary(raw_list: Vec<Vec<String>>, verbose: bool) -> HashMap<String, String> {
    let mut return_dict = HashMap::new();
    if verbose {
        let mut warning_count = 0;
        for entry in &raw_list {
            if let Some(_) = return_dict.insert(entry[0].clone(), entry[1].clone()) {
                warning_count += 1;
                println!("[WARNING]: Overwriting entry [count = {}]", warning_count);
            }
        }
        if warning_count > 0 {
            println!("[INFO] If you want to avoid overwriting duplicate headers set return_list=True");
        } else {
            println!("[INFO]: All processed sequences uniquely added to the returning dictionary");
        }
    } else {
        for entry in &raw_list {
            return_dict.insert(entry[0].clone(), entry[1].clone());
        }
    }
    return_dict
}

pub fn fail_on_duplicates(dataset: Vec<(String, String)>) -> Result<(), String> {
    let mut lookup = HashMap::new();
    for entry in dataset {
        if let Some(existing_value) = lookup.get(&entry.0) {
            if existing_value == &entry.1 {
                return Err(format!(
                    "Found duplicate entries of the following record:\n>{}\n{}",
                    entry.0, entry.1
                ));
            }
        } else {
            lookup.insert(entry.0, entry.1);
        }
    }
    Ok(())
}

pub fn remove_duplicates(dataset: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut lookup: HashMap<String, Vec<String>> = HashMap::new();
    let mut updated: Vec<(String, String)> = Vec::new();
    for entry in dataset {
        if !lookup.contains_key(&entry.0) {
            lookup.insert(entry.0.clone(), vec![entry.1.clone()]);
            updated.push(entry);
        } else {
            let mut found_dupe = false;
            for d in &lookup[&entry.0] {
                if d == &entry.1 {
                    found_dupe = true;
                    break;
                }
            }
            if !found_dupe {
                lookup.get_mut(&entry.0).unwrap().push(entry.1.clone());
                updated.push(entry);
            }
        }
    }
    updated
}

/// Fails if duplicate headers are found.
pub fn fail_on_duplicate_records(dataset: Vec<Vec<String>>) -> Result<(), String> {
    let mut headers = std::collections::HashSet::new();
    for entry in dataset {
        if !headers.insert(entry[0].clone()) {
            return Err(format!("Duplicate header found: {}", entry[0]));
        }
    }
    Ok(())
}

/// Removes entries with duplicate headers.
pub fn remove_duplicate_records(dataset: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    dataset
        .into_iter()
        .filter(|entry| seen.insert(entry[0].clone()))
        .collect()
}

/// Fails if duplicate sequences are found.
pub fn fail_on_duplicate_sequences(dataset: Vec<Vec<String>>) -> Result<(), String> {
    let mut sequences = std::collections::HashSet::new();
    for entry in dataset {
        if !sequences.insert(entry[1].clone()) {
            return Err(format!("Duplicate sequence found for header: {}", entry[0]));
        }
    }
    Ok(())
}

/// Removes entries with duplicate sequences.
pub fn remove_duplicate_sequences(dataset: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    dataset
        .into_iter()
        .filter(|entry| seen.insert(entry[1].clone()))
        .collect()
}
