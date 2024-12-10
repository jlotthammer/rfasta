use std::collections::HashMap;
use std::io::Write;
use std::fs;

/// Checks the validity of input parameters.
///
/// This function validates the provided input parameters to ensure they meet the expected requirements.
///
/// # Arguments
///
/// * `expect_unique_header` - A boolean indicating whether headers are expected to be unique.
/// * `header_parser` - An optional function to parse headers.
/// * `check_header_parser` - A boolean to check the header parser function.
/// * `duplicate_record_action` - Action to take on duplicate records ("ignore", "fail", "remove").
/// * `duplicate_sequence_action` - Action to take on duplicate sequences ("ignore", "fail", "remove").
/// * `invalid_sequence_action` - Action to take on invalid sequences.
/// * `alignment` - A boolean indicating if sequences should be aligned.
/// * `return_list` - A boolean indicating if results should be returned as a list.
/// * `output_filename` - An optional output filename.
/// * `verbose` - A boolean for verbose output.
/// * `correction_dictionary` - An optional dictionary for sequence corrections.
///
/// # Returns
///
/// * `Ok(())` if all inputs are valid, or an `Err(String)` with an error message.
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

/// Parses a FASTA file and returns the data as a vector of sequences.
///
/// This function reads a FASTA file and parses its content into a vector of [header, sequence] pairs.
///
/// # Arguments
///
/// * `filename` - The path to the FASTA file.
/// * `expect_unique_header` - Whether to expect unique headers in the FASTA file.
/// * `header_parser` - An optional function to parse header lines.
/// * `verbose` - Whether to enable verbose output.
///
/// # Returns
///
/// * `Result<Vec<Vec<String>>, String>` - A vector where each element is a vector containing the header and sequence, or an error message.
pub fn internal_parse_fasta_file(
    filename: &str,
    expect_unique_header: bool,
    header_parser: Option<Box<dyn Fn(String) -> String>>,
    verbose: bool,
) -> Result<Vec<Vec<String>>, String> {
    // Read in the file...
    let content = fs::read_to_string(filename)
        .map_err(|_| format!("Unable to find or read file: {}", filename))?;

    let lines: Vec<String> = content.lines().map(|s: &str| s.to_string()).collect();

    if verbose {
        println!("[INFO]: Read in file with {} lines", lines.len());
    }

    // Call _parse_fasta_all to parse the content
    Ok(_parse_fasta_all(lines, expect_unique_header, header_parser, verbose))
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

/// Writes FASTA data to a file.
///
/// This function takes FASTA data and writes it to a specified file in FASTA format.
///
/// # Arguments
///
/// * `fasta_data` - A vector of [header, sequence] pairs to write.
/// * `filename` - The output file path.
/// * `line_length` - An optional line length for wrapping sequences.
/// * `verbose` - Whether to enable verbose output.
/// * `append_to_fasta` - Whether to append to the file instead of overwriting.
///
/// # Returns
///
/// * `Ok(())` if writing is successful, or an `Err(String)` with an error message.
pub fn write_fasta(
    fasta_data: Vec<Vec<String>>,
    filename: &str,
    line_length: Option<usize>,
    verbose: bool,
    append_to_fasta: bool,
) -> Result<(), String> {
    let line_length = match line_length {
        Some(len) if len >= 5 => Some(len),
        Some(_) => Some(5),
        None => None,
    };
    let data_len = fasta_data.len();
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(append_to_fasta)
        .truncate(!append_to_fasta)
        .open(filename)
        .map_err(|e| e.to_string())?;
    for entry in &fasta_data {
        if entry.len() != 2 {
            return Err("Each entry must contain exactly two elements: header and sequence".to_string());
        }
        let (header, seq) = (&entry[0], &entry[1]);
        if seq.is_empty() {
            return Err(format!("Sequence associated with [{}] is empty", header));
        }
        writeln!(file, ">{}", header).map_err(|e| e.to_string())?;
        match line_length {
            Some(len) => {
                for chunk in seq.as_bytes().chunks(len) {
                    file.write_all(chunk).map_err(|e| e.to_string())?;
                    file.write_all(b"\n").map_err(|e| e.to_string())?;
                }
            }
            None => {
                writeln!(file, "{}", seq).map_err(|e| e.to_string())?;
            }
        }
        file.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    if verbose {
        println!("[INFO]: Wrote {} sequences to {}", data_len, filename);
    }
    Ok(())
}

/// Splits FASTA data into approximately equal chunks.
///
/// This function splits a vector of FASTA sequences into `n` chunks, ensuring that header and sequence pairs are kept together.
///
/// # Arguments
///
/// * `fasta_data` - A vector of [header, sequence] pairs.
/// * `n` - The number of chunks to split the data into.
///
/// # Returns
///
/// * `Vec<Vec<Vec<String>>>` - A vector of chunks, where each chunk is a vector of [header, sequence] pairs.
pub fn split_fasta(
    fasta_data: Vec<Vec<String>>,
    n: usize,
) -> Vec<Vec<Vec<String>>> {
    let total_sequences = fasta_data.len();
    let chunk_size = (total_sequences + n - 1) / n; // Ceiling division to ensure all sequences are included
    let mut chunks: Vec<Vec<Vec<String>>> = Vec::new();
    let mut current_chunk: Vec<Vec<String>> = Vec::new();
    let mut current_size = 0;

    for entry in fasta_data {
        if current_size >= chunk_size && chunks.len() < n - 1 {
            chunks.push(current_chunk);
            current_chunk = Vec::new();
            current_size = 0;
        }
        current_chunk.push(entry);
        current_size += 1;
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}