mod common;

use std::fs;
use std::process::Command;

#[test]
fn cli_help_mentions_streaming_split() {
    let output = Command::new(env!("CARGO_BIN_EXE_rfasta"))
        .arg("--help")
        .output()
        .expect("run cli");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("stream"));
    assert!(stdout.contains("split"));
}

#[test]
fn cli_clean_rejects_incompatible_unique_header_and_ignore_mode() {
    let dir = common::unique_temp_dir("rfasta_cli_clean");
    let input = common::write_text_file(&dir, "input.fasta", ">seq1\nAAAA\n");

    let output = Command::new(env!("CARGO_BIN_EXE_rfasta"))
        .args([
            "clean",
            input.to_str().unwrap(),
            "--duplicate-record",
            "ignore",
        ])
        .output()
        .expect("run clean");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("help:"));

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_split_writes_round_robin_shards() {
    let dir = common::unique_temp_dir("rfasta_cli_split");
    let input = common::write_text_file(
        &dir,
        "input.fasta",
        ">seq1\nAAAA\n>seq2\nCCCC\n>seq3\nDDDD\n>seq4\nEEEE\n",
    );
    let output_dir = dir.join("shards");

    let output = Command::new(env!("CARGO_BIN_EXE_rfasta"))
        .args([
            "split",
            input.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--chunks",
            "2",
            "--silent",
        ])
        .output()
        .expect("run split");

    assert!(output.status.success());
    let shard1 = fs::read_to_string(output_dir.join("input_000001.fasta")).unwrap();
    let shard2 = fs::read_to_string(output_dir.join("input_000002.fasta")).unwrap();
    assert!(shard1.contains(">seq1"));
    assert!(shard1.contains(">seq3"));
    assert!(shard2.contains(">seq2"));
    assert!(shard2.contains(">seq4"));

    fs::remove_dir_all(dir).unwrap();
}
