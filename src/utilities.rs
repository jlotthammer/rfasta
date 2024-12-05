use pyo3::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;
use std::vec::Vec;
use pyo3::exceptions::PyException;
use pyo3::PyErr;
use pyo3::types::PyDict;

use crate::configs::{get_standard_aas, get_standard_aas_with_gap, get_standard_conversion, get_standard_conversion_with_gap};

#[pyfunction]
// Function signature modified to return references
pub fn build_custom_dictionary(additional_dictionary: &PyDict) -> PyResult<HashMap<&str, &str>> {
    let mut final_dict: HashMap<&str, &str> = HashMap::new();
    let standard_conversion = get_standard_conversion();
    
    // Copy STANDARD_CONVERSION into final_dict
    for (key, value) in standard_conversion.iter() {
        final_dict.insert(key, value);
    }
    
    // Iterate over the additional dictionary and insert references into the final dictionary
    for (key, value) in additional_dictionary {
        let key_str: &str = key.extract()?;
        let value_str: &str = value.extract()?;
        final_dict.insert(key_str, value_str);
    }

    // Return the final dictionary
    Ok(final_dict)
}


#[pyfunction]
#[pyo3(signature = (seq, alignment = false, correction_dictionary = None))]
pub fn convert_to_valid(seq: &str, alignment: bool, correction_dictionary: Option<HashMap<&str, &str>>) -> PyResult<String> {
    // Choose the conversion dictionary based on alignment
    let converter: HashMap<&str, &str> = match correction_dictionary {
        Some(dict) => dict,
        None => {
            if alignment {
                get_standard_conversion_with_gap()
            } else {
                get_standard_conversion()
            }
        }
    };
    // Perform conversion
    let mut result: String = String::from(seq);
    for (key, value) in converter.iter() {
        result = result.replace(key, value);
    }
    
    let converted_seq = result.to_owned();
    Ok(converted_seq)
}

#[pyfunction]
pub fn check_sequence_is_valid(seq: &str, alignment: bool) -> (bool, char) {
    // Define standard amino acid lists
    let standard_aas: HashSet<&char> = get_standard_aas();

    let standard_aas_with_gap: HashSet<&char> = get_standard_aas_with_gap();

    // Choose valid amino acid list based on alignment flag
    let valid_aa_list: &'_ HashSet<&char> = if alignment {
        &standard_aas_with_gap
    } else {
        &standard_aas
    };
    

    // Convert sequence to a set of characters to remove duplicates
    let seq_chars: HashSet<char> = seq.chars().collect();

    // Iterate over unique characters in the sequence
    for &c in &seq_chars {
        // Check if the character is a valid amino acid
        if !valid_aa_list.contains(&c) {
            return (false, c);
        }
    }

    (true, '0') // Return (true, '0') if sequence is valid
}  

#[pyfunction]
#[pyo3(signature = (dataset, correction_dictionary = None, alignment = false))]
pub fn convert_invalid_sequences(
    mut dataset: Vec<Vec<String>>,
    correction_dictionary: Option<HashMap<&str, &str>>,
    alignment: bool,
) -> PyResult<(Vec<Vec<String>>, usize)> {
    let mut count = 0;

    for row in &mut dataset {
        let s = row[1].clone();
        let corrected = convert_to_valid(&s, alignment, correction_dictionary.clone())?;
        row[1] = corrected.clone();

        if s != corrected {
            count += 1;
        }
    }

    Ok((dataset, count))
}


#[pyfunction]
pub fn remove_invalid_sequences(
    dataset: Vec<Vec<String>>,
    alignment: bool,
) -> PyResult<Vec<Vec<String>>> {
    // Iterate through the dataset, filtering out invalid sequences
    let filtered_dataset: Vec<Vec<String>> = dataset
        .into_iter()
        .filter(|element| {
            let (is_valid, _) = check_sequence_is_valid(&element[1], alignment);
            is_valid
        })
        .collect();

    Ok(filtered_dataset)
}

#[pyfunction]
pub fn fail_on_invalid_sequences(sequences: Vec<Vec<String>>, alignment: bool) -> PyResult<()> {
    for sequence in &sequences {  // Note the & here to borrow from owned data
        let (is_valid, invalid_char) = check_sequence_is_valid(&sequence[1], alignment);
        if !is_valid {
            return Err(PyErr::new::<PyException, _>(format!(
                "Invalid character '{}' found in sequence: {}",
                invalid_char, sequence[0]
            )));
        }
    }
    Ok(())
}

#[pyfunction]
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
    return return_dict;
}

#[pyfunction]
pub fn fail_on_duplicates(dataset: Vec<(String, String)>) -> PyResult<()> {
    let mut lookup = std::collections::HashMap::new();

    for entry in dataset {
        if let Some(existing_value) = lookup.get(&entry.0) {
            if existing_value == &entry.1 {
                return Err(PyErr::new::<PyException, _>(format!(
                    "Found duplicate entries of the following record:\n>{}\n{}",
                    entry.0, entry.1
                )));
            }
        } else {
            lookup.insert(entry.0, entry.1);
        }
    }

    Ok(())
}

#[pyfunction]
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
