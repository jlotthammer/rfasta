use std::fs;
use std::io::Cursor;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use rfasta::clean::{clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction};
use rfasta::parse::{parse_fasta_reader, ParseOptions};
use rfasta::shard::split_fasta_file_round_robin;
use rfasta::write::{write_fasta_writer, WriteOptions};

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    dir.push(format!("{prefix}_{}_{}", std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("create temp benchmark dir");
    dir
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = include_str!("../sequences.fasta");

    let start = Instant::now();
    let records = parse_fasta_reader(Cursor::new(input.as_bytes()), ParseOptions::default())?;
    println!("parse_fasta_reader: {:?}", start.elapsed());

    let start = Instant::now();
    let cleaned = clean_sequences(
        records.clone(),
        &CleanOptions {
            invalid_sequence_action: InvalidSequenceAction::Fail,
            duplicate_record_action: DuplicateAction::Fail,
            duplicate_sequence_action: DuplicateAction::Ignore,
            ..CleanOptions::default()
        },
    )?;
    println!("clean_sequences: {:?}", start.elapsed());

    let start = Instant::now();
    let mut sink = Vec::new();
    write_fasta_writer(&mut sink, &cleaned, &WriteOptions::default())?;
    println!("write_fasta_writer: {:?}", start.elapsed());

    let tempdir = unique_temp_dir("rfasta_benchmark_driver");
    let input_path = tempdir.join("input.fasta");
    let output_dir = tempdir.join("shards");
    fs::write(&input_path, input)?;

    let start = Instant::now();
    split_fasta_file_round_robin(&input_path, output_dir, 4, Some(60), false)?;
    println!("split_fasta_file_round_robin: {:?}", start.elapsed());

    fs::remove_dir_all(tempdir)?;
    Ok(())
}
