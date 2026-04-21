mod common;

use std::fs;
use std::io::Cursor;

use rfasta::parse::{parse_fasta_file, parse_fasta_reader, FastaRecord, ParseOptions};
use rfasta::shard::split_fasta_file_round_robin;
use rfasta::write::{write_fasta_file, write_fasta_writer, WriteOptions};

#[test]
fn parse_reader_handles_multiline_sequences_and_blank_lines() {
    let input = b">seq1\nac\n\nDE\n>seq2\nTT\n";
    let records = parse_fasta_reader(Cursor::new(input), ParseOptions::default()).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0], FastaRecord::new("seq1", "ACDE"));
    assert_eq!(records[1], FastaRecord::new("seq2", "TT"));
}

#[test]
fn parse_file_rejects_sequence_before_header() {
    let dir = common::unique_temp_dir("rfasta_io_parse");
    let input = common::write_text_file(&dir, "bad.fasta", "ACDE\n>seq1\nAAAA\n");
    let error = parse_fasta_file(input, ParseOptions::default(), false).unwrap_err();
    assert!(error.to_string().contains("line 1"));
    assert!(error.to_string().contains("help:"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn parse_reader_rejects_duplicate_headers_when_requested() {
    let input = b">seq1\nAAAA\n>seq1\nCCCC\n";
    let error = parse_fasta_reader(Cursor::new(input), ParseOptions::default()).unwrap_err();
    assert!(error.to_string().contains("duplicate header"));
}

#[test]
fn write_writer_wraps_output_lines() {
    let records = vec![FastaRecord::new("seq1", "ACDEFGHIK")];
    let mut output = Vec::new();
    write_fasta_writer(
        &mut output,
        &records,
        &WriteOptions {
            line_length: Some(4),
            append: false,
        },
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("ACDEF\nGHIK\n") || output.contains("ACDE\nFGHI\nK\n"));
}

#[test]
fn split_file_round_robin_writes_non_empty_shards() {
    let dir = common::unique_temp_dir("rfasta_io_split");
    let input = common::write_text_file(
        &dir,
        "input.fasta",
        ">seq1\nAAAA\n>seq2\nCCCC\n>seq3\nDDDD\n",
    );
    let output_dir = dir.join("shards");

    let written = split_fasta_file_round_robin(input, &output_dir, 5, Some(60), false).unwrap();
    assert_eq!(written, 3);
    assert!(output_dir.join("input_000001.fasta").exists());
    assert!(output_dir.join("input_000002.fasta").exists());
    assert!(output_dir.join("input_000003.fasta").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn write_file_rejects_empty_sequence() {
    let dir = common::unique_temp_dir("rfasta_io_write");
    let output = dir.join("out.fasta");
    let error = write_fasta_file(
        &[FastaRecord::new("seq1", "")],
        output,
        WriteOptions::default(),
        false,
    )
    .unwrap_err();
    assert!(error.to_string().contains("empty"));
    fs::remove_dir_all(dir).unwrap();
}
