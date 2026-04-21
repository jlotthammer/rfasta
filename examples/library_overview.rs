use std::io::Cursor;

use rfasta::clean::{clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction};
use rfasta::parse::{parse_fasta_reader, FastaRecord, ParseOptions};
use rfasta::write::{write_fasta_writer, WriteOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = b">seq1\nacdx\n>seq2\nTTTT\n";
    let records = parse_fasta_reader(Cursor::new(input), ParseOptions::default())?;

    let cleaned = clean_sequences(
        records,
        &CleanOptions {
            invalid_sequence_action: InvalidSequenceAction::ConvertRemove,
            duplicate_record_action: DuplicateAction::Fail,
            duplicate_sequence_action: DuplicateAction::Ignore,
            ..CleanOptions::default()
        },
    )?;

    let mut output = Vec::new();
    write_fasta_writer(&mut output, &cleaned, &WriteOptions::default())?;
    let output = String::from_utf8(output)?;
    assert!(output.contains(">seq1"));
    assert!(output.contains(">seq2"));

    let extra = FastaRecord::new("seq3", "AAAA");
    assert_eq!(extra.header, "seq3");
    println!("{output}");
    Ok(())
}
