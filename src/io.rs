use std::collections::HashMap;
use std::io::Write;
use std::fs;
use pyo3::PyErr;
use pyo3::prelude::*;

pub fn check_inputs(
    expect_unique_header: bool,
    header_parser: Option<Box<dyn Fn(String) -> Result<String, String>>>,
    check_header_parser: bool,
    duplicate_record_action: &str,
    duplicate_sequence_action: &str,
    invalid_sequence_action: &str,
    alignment: bool,
    return_list: bool,
    output_filename: Option<String>,
    verbose: bool,
    correction_dictionary: Option<HashMap<String, String>>,
) -> Result<(), String> {
    
    // Check expect_unique_header
    if !expect_unique_header {
        return Err("keyword 'expect_unique_header' must be a boolean".to_string());
    }

    // Validate header_parser
    if check_header_parser {
        if let Some(header_parser) = &header_parser {
            let tst_string: String = "this test string should work".to_string();
            return match header_parser(tst_string.clone()) {
                Ok(a) if a.is_empty() => Ok(()),
                Ok(_) => Err("Something went wrong when testing the header_parser function.\nFunction completed but return value was not a string".to_string()),
                Err(e) => Err(format!("Something went wrong when testing the header_parser function using string: {}.\nMaybe you should set check_header_parser to False? \nException: {}", tst_string, e)),
            };
        }
    }

    // Check duplicate_record_action
    match duplicate_record_action {
        "ignore" | "fail" | "remove" => (),
        _ => return Err("keyword 'duplicate_record_action' must be one of 'ignore','fail','remove'".to_string()),
    }

    // Check duplicate_sequence_action
    match duplicate_sequence_action {
        "ignore" | "fail" | "remove" => (),
        _ => return Err("keyword 'duplicate_sequence_action' must be one of 'ignore','fail', 'remove'".to_string()),
    }

    // Check invalid_sequence_action
    match invalid_sequence_action {
        "ignore" | "fail" | "remove" | "convert" | "convert-ignore" | "convert-remove" => (),
        _ => return Err("keyword 'invalid_sequence_action' must be one of 'ignore','fail','remove','convert','convert-ignore', 'convert-remove'".to_string()),
    }

    // Check return_list
    if !return_list {
        return Err("keyword 'return_list' must be a boolean".to_string());
    }

    // Check output_filename
    if let Some(output_filename) = &output_filename {
        if output_filename.is_empty() {
            return Err("keyword 'output_filename' must be a non-empty string".to_string());
        }
    }

    // Check verbose
    if !verbose {
        return Err("keyword 'verbose' must be a boolean".to_string());
    }

    // Check alignment
    if !alignment {
        return Err("keyword 'alignment' must be a boolean".to_string());
    }

    // Check if expect_unique_header is true while duplicate_record_action is 'ignore'
    if duplicate_record_action == "ignore" && expect_unique_header {
        return Err("Cannot expect unique headers and ignore duplicate records".to_string());
    }

    // Check correction_dictionary
    if let Some(correction_dictionary) = &correction_dictionary {
        if correction_dictionary.is_empty() {
            return Err("If provided, keyword 'correction_dictionary' must be a non-empty dictionary".to_string());
        }
    }

    Ok(())
}

pub fn internal_parse_fasta_file(
    filename: &str,
    expect_unique_header: bool,
    header_parser: Option<Box<dyn Fn(String) -> String>>,
    verbose: bool,
) -> Vec<Vec<String>> {
    // Read in the file...
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(_) => panic!("Unable to find file: {}", filename),
    };

    let lines: Vec<String> = content.lines().map(|s: &str| s.to_string()).collect();

    if verbose {
        println!("[INFO]: Read in file with {} lines", lines.len());
    }

    // Call _parse_fasta_all to parse the content
    _parse_fasta_all(lines, expect_unique_header, header_parser, verbose)
}

fn _parse_fasta_all(
    content: Vec<String>,
    expect_unique_header: bool,
    header_parser: Option<Box<dyn Fn(String) -> String>>,
    verbose: bool,
) -> Vec<Vec<String>> {
    let mut return_data: Vec<Vec<String>> = Vec::new();
    let mut all_headers: HashMap<String, bool> = HashMap::new();
    let mut seq: String = String::new();
    let mut header: String = String::new();

    fn update(header: &str, seq: &str, all_headers: &mut HashMap<String, bool>, expect_unique_header: bool, return_data: &mut Vec<Vec<String>>) {
        if all_headers.contains_key(header) {
            if expect_unique_header {
                panic!("Found duplicate header ({})", header);
            }
        } else {
            all_headers.insert(header.to_string(), true);
        }
        return_data.push(vec![header.to_string(), seq.to_uppercase()]);
    }

    for line in content {
        let sline: &str = line.trim();

        if sline.is_empty() {
            continue;
        }

        if sline.starts_with('>') {
            let h: String = sline[1..].to_string();

            if !seq.is_empty() {
                update(&header, &seq, &mut all_headers, expect_unique_header, &mut return_data);
            }

            header = if let Some(header_parser) = &header_parser {
                header_parser(h)
            } else {
                h
            };
            seq.clear();
        } else {
            seq.push_str(sline);
        }
    }

    if !seq.is_empty() {
        update(&header, &seq, &mut all_headers, expect_unique_header, &mut return_data);
    }

    if verbose {
        println!("[INFO]: Parsed file to recover {} sequences", return_data.len());
    }

    return_data
}

#[pyfunction]
#[pyo3(signature = (fasta_data, filename, line_length = None, verbose = true, append_to_fasta = false))]
pub fn write_fasta(
    fasta_data: Vec<Vec<String>>,
    filename: &str,
    line_length: Option<usize>,
    verbose: bool,
    append_to_fasta: bool,
) -> PyResult<()> {
    // Validate line length
    let line_length = match line_length {
        Some(len) if len >= 5 => Some(len),
        Some(_) => Some(5),
        None => None,
    };

    let data_len = fasta_data.len();  // Store length before processing
    
    // Open file with appropriate mode
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(append_to_fasta)
        .truncate(!append_to_fasta)
        .open(filename)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    for entry in &fasta_data {  // Use reference to avoid moving fasta_data
        if entry.len() != 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Each entry must contain exactly two elements: header and sequence",
            ));
        }

        let (header, seq) = (&entry[0], &entry[1]);
        
        if seq.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Sequence associated with [{}] is empty", header)
            ));
        }

        // Write header
        writeln!(file, ">{}", header)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        // Write sequence with line breaks
        match line_length {
            Some(len) => {
                for chunk in seq.as_bytes().chunks(len) {
                    file.write_all(chunk)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
                    file.write_all(b"\n")
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
                }
            }
            None => {
                writeln!(file, "{}", seq)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            }
        }

        // Add extra newline between sequences
        file.write_all(b"\n")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    }

    if verbose {
        println!("[INFO]: Wrote {} sequences to {}", data_len, filename);
    }

    Ok(())
}

