use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::configs::{DEFAULT_LINE_LENGTH, MIN_LINE_LENGTH};
use crate::errors::RfastaError;

/// A parsed FASTA record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FastaRecord {
    /// FASTA header text without the leading `>`.
    pub header: String,
    /// Uppercased sequence associated with the header.
    pub sequence: String,
}

impl FastaRecord {
    /// Creates a new FASTA record.
    pub fn new(header: impl Into<String>, sequence: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            sequence: sequence.into(),
        }
    }
}

/// Options for FASTA parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseOptions {
    /// Whether duplicate headers should be rejected during parsing.
    pub expect_unique_header: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            expect_unique_header: true,
        }
    }
}

/// Options for FASTA writing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteOptions {
    /// Maximum residues per output line. `None` disables wrapping.
    pub line_length: Option<usize>,
    /// Whether output should be appended instead of replacing the destination file.
    pub append: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            line_length: Some(DEFAULT_LINE_LENGTH),
            append: false,
        }
    }
}

#[derive(Debug)]
struct ShardWriter {
    path: PathBuf,
    writer: BufWriter<File>,
    records_written: usize,
}

fn normalize_line_length(line_length: Option<usize>) -> Option<usize> {
    match line_length {
        Some(0) | None => None,
        Some(length) if length < MIN_LINE_LENGTH => Some(MIN_LINE_LENGTH),
        Some(length) => Some(length),
    }
}

fn finish_record<F>(
    header: &mut Option<String>,
    sequence: &mut String,
    seen_headers: &mut Option<HashSet<String>>,
    source_name: Option<&Path>,
    visit: &mut F,
    records_seen: &mut usize,
) -> Result<(), RfastaError>
where
    F: FnMut(FastaRecord) -> Result<(), RfastaError>,
{
    let header = header
        .take()
        .expect("finish_record is only called when a header is present");

    if sequence.is_empty() {
        return Err(RfastaError::empty_sequence(
            source_name,
            header,
            "Ensure each FASTA header is followed by at least one sequence line.",
        ));
    }

    if let Some(seen_headers) = seen_headers.as_mut() {
        if !seen_headers.insert(header.clone()) {
            return Err(RfastaError::duplicate_header(
                source_name,
                header,
                "Pass ParseOptions { expect_unique_header: false } or use --non-unique-header if repeated headers are expected.",
            ));
        }
    }

    sequence.make_ascii_uppercase();
    let record = FastaRecord::new(header, std::mem::take(sequence));
    *records_seen += 1;
    visit(record)
}

fn write_record_to_writer<W: Write>(
    writer: &mut W,
    record: &FastaRecord,
    line_length: Option<usize>,
    path: Option<&Path>,
) -> Result<(), RfastaError> {
    if record.sequence.is_empty() {
        return Err(RfastaError::empty_sequence(
            path,
            record.header.clone(),
            "Remove empty records before writing, or make sure every header has a sequence.",
        ));
    }

    writer
        .write_all(format!(">{}\n", record.header).as_bytes())
        .map_err(|source| {
            RfastaError::io(
                "write",
                path,
                source,
                "Check that the output path is writable and has enough free space.",
            )
        })?;

    match normalize_line_length(line_length) {
        Some(line_length) => {
            for chunk in record.sequence.as_bytes().chunks(line_length) {
                writer.write_all(chunk).map_err(|source| {
                    RfastaError::io(
                        "write",
                        path,
                        source,
                        "Check that the output path is writable and has enough free space.",
                    )
                })?;
                writer.write_all(b"\n").map_err(|source| {
                    RfastaError::io(
                        "write",
                        path,
                        source,
                        "Check that the output path is writable and has enough free space.",
                    )
                })?;
            }
        }
        None => {
            writer
                .write_all(record.sequence.as_bytes())
                .and_then(|_| writer.write_all(b"\n"))
                .map_err(|source| {
                    RfastaError::io(
                        "write",
                        path,
                        source,
                        "Check that the output path is writable and has enough free space.",
                    )
                })?;
        }
    }

    writer.write_all(b"\n").map_err(|source| {
        RfastaError::io(
            "write",
            path,
            source,
            "Check that the output path is writable and has enough free space.",
        )
    })
}

/// Streams FASTA records from any buffered reader and invokes `visit` for each record.
///
/// This is the lowest-allocation API for large inputs. The callback is invoked one record at a
/// time, allowing callers to process or shard data without first collecting the full dataset.
///
/// # Example
/// ```
/// use std::io::Cursor;
///
/// use rfasta::parse::{visit_fasta_reader, FastaRecord, ParseOptions};
/// use rfasta::RfastaError;
///
/// let data = b">seq1\nacgt\n>seq2\nTTTT\n";
/// let mut headers = Vec::new();
/// visit_fasta_reader(Cursor::new(data), ParseOptions::default(), |record: FastaRecord| {
///     headers.push(record.header);
///     Ok::<(), RfastaError>(())
/// })?;
/// assert_eq!(headers, vec!["seq1".to_string(), "seq2".to_string()]);
/// # Ok::<(), RfastaError>(())
/// ```
pub fn visit_fasta_reader<R, F>(
    mut reader: R,
    options: ParseOptions,
    mut visit: F,
) -> Result<usize, RfastaError>
where
    R: BufRead,
    F: FnMut(FastaRecord) -> Result<(), RfastaError>,
{
    visit_fasta_reader_with_source(&mut reader, options, None, &mut visit)
}

fn visit_fasta_reader_with_source<R, F>(
    reader: &mut R,
    options: ParseOptions,
    source_name: Option<&Path>,
    visit: &mut F,
) -> Result<usize, RfastaError>
where
    R: BufRead,
    F: FnMut(FastaRecord) -> Result<(), RfastaError>,
{
    let mut line = String::new();
    let mut current_header = None;
    let mut current_sequence = String::new();
    let mut line_number = 0usize;
    let mut records_seen = 0usize;
    let mut seen_headers = options.expect_unique_header.then(HashSet::new);

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).map_err(|source| {
            RfastaError::io(
                "parse",
                source_name,
                source,
                "Check that the input file exists and is readable.",
            )
        })?;

        if bytes_read == 0 {
            break;
        }

        line_number += 1;
        let stripped = line.trim();

        if stripped.is_empty() {
            continue;
        }

        if let Some(header) = stripped.strip_prefix('>') {
            if current_header.is_some() {
                finish_record(
                    &mut current_header,
                    &mut current_sequence,
                    &mut seen_headers,
                    source_name,
                    visit,
                    &mut records_seen,
                )?;
            }
            current_header = Some(header.to_string());
            continue;
        }

        if current_header.is_none() {
            return Err(RfastaError::malformed_fasta(
                source_name,
                line_number,
                "found sequence data before the first FASTA header",
                "Make sure the file starts with a header line beginning with `>`.",
            ));
        }

        current_sequence.push_str(stripped);
    }

    if current_header.is_some() {
        finish_record(
            &mut current_header,
            &mut current_sequence,
            &mut seen_headers,
            source_name,
            visit,
            &mut records_seen,
        )?;
    }

    Ok(records_seen)
}

/// Parses all FASTA records from a buffered reader into memory.
///
/// Use [`visit_fasta_reader`] when you want streaming behavior for very large inputs.
///
/// # Example
/// ```
/// use std::io::Cursor;
///
/// use rfasta::parse::{parse_fasta_reader, ParseOptions};
///
/// let data = b">seq1\nacgt\n>seq2\naaaa\n";
/// let records = parse_fasta_reader(Cursor::new(data), ParseOptions::default())?;
/// assert_eq!(records[0].sequence, "ACGT");
/// assert_eq!(records.len(), 2);
/// # Ok::<(), rfasta::RfastaError>(())
/// ```
pub fn parse_fasta_reader<R: BufRead>(
    reader: R,
    options: ParseOptions,
) -> Result<Vec<FastaRecord>, RfastaError> {
    let mut records = Vec::new();
    visit_fasta_reader(reader, options, |record| {
        records.push(record);
        Ok(())
    })?;
    Ok(records)
}

/// Streams FASTA records from a file path.
///
/// This is the file-based counterpart to [`visit_fasta_reader`]. It is useful when you want
/// bounded-memory processing over a large FASTA without manually opening a reader.
pub fn visit_fasta_file<P, F>(
    path: P,
    options: ParseOptions,
    verbose: bool,
    mut visit: F,
) -> Result<usize, RfastaError>
where
    P: AsRef<Path>,
    F: FnMut(FastaRecord) -> Result<(), RfastaError>,
{
    let path = path.as_ref();
    let file = File::open(path).map_err(|source| {
        RfastaError::io(
            "parse",
            Some(path),
            source,
            "Check that the input file exists and is readable.",
        )
    })?;
    let mut reader = BufReader::new(file);
    let records = visit_fasta_reader_with_source(&mut reader, options, Some(path), &mut visit)?;
    if verbose {
        println!(
            "[INFO]: Parsed file to recover {records} sequences from {}",
            path.display()
        );
    }
    Ok(records)
}

/// Parses all FASTA records from a file path into memory.
///
/// # Example
/// ```no_run
/// use rfasta::parse::{parse_fasta_file, ParseOptions};
///
/// let records = parse_fasta_file("proteins.fasta", ParseOptions::default(), true)?;
/// println!("Loaded {} records", records.len());
/// # Ok::<(), rfasta::RfastaError>(())
/// ```
pub fn parse_fasta_file<P: AsRef<Path>>(
    path: P,
    options: ParseOptions,
    verbose: bool,
) -> Result<Vec<FastaRecord>, RfastaError> {
    let path = path.as_ref();
    let mut records = Vec::new();
    visit_fasta_file(path, options, verbose, |record| {
        records.push(record);
        Ok(())
    })?;
    Ok(records)
}

/// Writes FASTA records to any writer.
///
/// # Example
/// ```
/// use rfasta::parse::FastaRecord;
/// use rfasta::write::{write_fasta_writer, WriteOptions};
///
/// let records = vec![
///     FastaRecord::new("seq1", "ACDEFG"),
///     FastaRecord::new("seq2", "TTTT"),
/// ];
/// let mut buffer = Vec::new();
/// write_fasta_writer(&mut buffer, &records, &WriteOptions::default())?;
/// let output = String::from_utf8(buffer).unwrap();
/// assert!(output.contains(">seq1"));
/// # Ok::<(), rfasta::RfastaError>(())
/// ```
pub fn write_fasta_writer<W: Write>(
    writer: &mut W,
    records: &[FastaRecord],
    options: &WriteOptions,
) -> Result<(), RfastaError> {
    for record in records {
        write_record_to_writer(writer, record, options.line_length, None)?;
    }
    writer.flush().map_err(|source| {
        RfastaError::io(
            "write",
            None,
            source,
            "Check that the output writer is still open and writable.",
        )
    })
}

/// Writes FASTA records to a file.
///
/// This helper opens a buffered writer, applies the requested line wrapping, and emits a blank
/// line between records for compatibility with the existing CLI and Python interfaces.
pub fn write_fasta_file<P: AsRef<Path>>(
    records: &[FastaRecord],
    path: P,
    options: WriteOptions,
    verbose: bool,
) -> Result<(), RfastaError> {
    let path = path.as_ref();
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(options.append)
        .truncate(!options.append)
        .open(path)
        .map_err(|source| {
            RfastaError::io(
                "write",
                Some(path),
                source,
                "Check that the output path is writable and that parent directories exist.",
            )
        })?;
    let mut writer = BufWriter::new(file);
    for record in records {
        write_record_to_writer(&mut writer, record, options.line_length, Some(path))?;
    }
    writer.flush().map_err(|source| {
        RfastaError::io(
            "write",
            Some(path),
            source,
            "Check that the output path is writable and has enough free space.",
        )
    })?;
    if verbose {
        println!(
            "[INFO]: Wrote {} sequences to {}",
            records.len(),
            path.display()
        );
    }
    Ok(())
}

/// Splits records into round-robin shards in memory.
///
/// This helper is convenient for tests or small library workflows. For UniRef-scale files, prefer
/// [`split_fasta_file_round_robin`] so records are streamed directly into shard writers.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn split_records_round_robin(
    records: Vec<FastaRecord>,
    chunks: usize,
) -> Result<Vec<Vec<FastaRecord>>, RfastaError> {
    if chunks == 0 {
        return Err(RfastaError::invalid_chunk_count(
            chunks,
            "Pass a chunk count greater than zero.",
        ));
    }

    let mut split = std::iter::repeat_with(Vec::new)
        .take(chunks)
        .collect::<Vec<_>>();
    for (index, record) in records.into_iter().enumerate() {
        split[index % chunks].push(record);
    }
    split.retain(|chunk| !chunk.is_empty());
    Ok(split)
}

/// Streams a FASTA file into round-robin shard files without first collecting all records.
///
/// Shards are balanced by record order, not exact byte size. This avoids rereading the input file
/// and keeps memory bounded for very large inputs such as UniRef FASTA distributions.
///
/// # Example
/// ```no_run
/// use rfasta::shard::split_fasta_file_round_robin;
///
/// let written = split_fasta_file_round_robin("proteins.fasta", "shards", 8, Some(60), true)?;
/// println!("Wrote {written} shard files");
/// # Ok::<(), rfasta::RfastaError>(())
/// ```
pub fn split_fasta_file_round_robin<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_dir: Q,
    chunks: usize,
    line_length: Option<usize>,
    verbose: bool,
) -> Result<usize, RfastaError> {
    if chunks == 0 {
        return Err(RfastaError::invalid_chunk_count(
            chunks,
            "Pass a chunk count greater than zero.",
        ));
    }

    let input_path = input_path.as_ref();
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).map_err(|source| {
        RfastaError::io(
            "split",
            Some(output_dir),
            source,
            "Check that the output directory is writable, or create it before running split.",
        )
    })?;

    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("rfasta_shard")
        .to_string();

    let mut shard_index = 0usize;
    let mut writers: Vec<Option<ShardWriter>> =
        std::iter::repeat_with(|| None).take(chunks).collect();

    visit_fasta_file(
        input_path,
        ParseOptions {
            expect_unique_header: false,
        },
        false,
        |record| {
            let target = shard_index % chunks;
            if writers[target].is_none() {
                let path = output_dir.join(format!("{stem}_{:06}.fasta", target + 1));
                let file = File::create(&path).map_err(|source| {
                    RfastaError::io(
                        "split",
                        Some(&path),
                        source,
                        "Check that the output directory is writable and has enough free space.",
                    )
                })?;
                writers[target] = Some(ShardWriter {
                    path,
                    writer: BufWriter::new(file),
                    records_written: 0,
                });
            }

            let shard = writers[target]
                .as_mut()
                .expect("writer is initialized before use");
            write_record_to_writer(&mut shard.writer, &record, line_length, Some(&shard.path))?;
            shard.records_written += 1;
            shard_index += 1;
            Ok(())
        },
    )?;

    let mut files_written = 0usize;
    for shard in writers.iter_mut().flatten() {
        shard.writer.flush().map_err(|source| {
            RfastaError::io(
                "split",
                Some(&shard.path),
                source,
                "Check that the output directory is writable and has enough free space.",
            )
        })?;
        files_written += 1;
        if verbose {
            println!(
                "[INFO]: Wrote {} sequences to {}",
                shard.records_written,
                shard.path.display()
            );
        }
    }

    if verbose {
        println!("[INFO]: Split FASTA into {files_written} chunks");
    }

    Ok(files_written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_records_round_robin_distributes_records_by_order() {
        let records = vec![
            FastaRecord::new("seq1", "AAAA"),
            FastaRecord::new("seq2", "CCCC"),
            FastaRecord::new("seq3", "DDDD"),
            FastaRecord::new("seq4", "EEEE"),
        ];
        let chunks = split_records_round_robin(records, 2).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0][0].header, "seq1");
        assert_eq!(chunks[0][1].header, "seq3");
        assert_eq!(chunks[1][0].header, "seq2");
        assert_eq!(chunks[1][1].header, "seq4");
    }
}
