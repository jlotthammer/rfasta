use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::configs::DEFAULT_LINE_LENGTH;
use crate::errors::RfastaError;
use crate::io::{
    parse_fasta_file, split_fasta_file_round_robin, write_fasta_file, ParseOptions, WriteOptions,
};
use crate::sequence_processing::{
    clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction,
};

#[derive(Parser)]
#[command(
    author = "jlotthammer",
    version,
    about = "rfasta parses, cleans, writes, and shards protein FASTA files.",
    long_about = "rfasta is a production-ready FASTA toolkit for protein datasets. Use `clean` to standardize and validate records, and `split` to create shard files for parallel downstream processing.",
    after_help = "Examples:\n  rfasta clean proteins.fasta -o cleaned.fasta --duplicate-record remove --invalid-sequence convert-remove\n  rfasta split proteins.fasta --output-dir shards --chunks 8"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and clean a FASTA file.
    Clean {
        /// Input FASTA file.
        filename: PathBuf,

        /// Output FASTA file.
        #[arg(short = 'o')]
        output: Option<PathBuf>,

        /// Allow repeated FASTA headers during parsing.
        #[arg(long)]
        non_unique_header: bool,

        /// How to deal with exact duplicate FASTA records.
        #[arg(long, value_enum, default_value_t = DuplicateAction::Fail)]
        duplicate_record: DuplicateAction,

        /// How to deal with duplicate sequences across headers.
        #[arg(long, value_enum, default_value_t = DuplicateAction::Ignore)]
        duplicate_sequence: DuplicateAction,

        /// How to deal with invalid sequences.
        #[arg(long, value_enum, default_value_t = InvalidSequenceAction::Fail)]
        invalid_sequence: InvalidSequenceAction,

        /// Number of residues per line in the output FASTA.
        #[arg(long)]
        number_lines: Option<usize>,

        /// Shortest sequence to keep.
        #[arg(long)]
        shortest_seq: Option<usize>,

        /// Longest sequence to keep.
        #[arg(long)]
        longest_seq: Option<usize>,

        /// Randomly subsample this many sequences after filtering.
        #[arg(long)]
        random_subsample: Option<usize>,

        /// Print summary statistics for the cleaned output.
        #[arg(long)]
        print_statistics: bool,

        /// Skip writing the cleaned output file.
        #[arg(long)]
        no_outputfile: bool,

        /// Suppress informational output.
        #[arg(long)]
        silent: bool,

        /// Replace commas in FASTA headers with semicolons.
        #[arg(long)]
        remove_comma_from_header: bool,
    },
    /// Split a FASTA file into shard files.
    Split {
        /// Input FASTA file.
        filename: PathBuf,

        /// Output directory. Files are created on demand.
        #[arg(short = 'o', long)]
        output_dir: PathBuf,

        /// Number of requested shard buckets.
        #[arg(short, long)]
        chunks: usize,

        /// Line length to use in shard output.
        #[arg(long, default_value_t = DEFAULT_LINE_LENGTH)]
        line_length: usize,

        /// Skip writing output files.
        #[arg(long)]
        no_outputfiles: bool,

        /// Suppress informational output.
        #[arg(long)]
        silent: bool,
    },
}

/// Runs the rfasta command-line interface.
pub fn main(args: &[String]) -> Result<(), RfastaError> {
    let args = Args::parse_from(args);

    match args.command {
        Commands::Clean {
            filename,
            output,
            non_unique_header,
            duplicate_record,
            duplicate_sequence,
            invalid_sequence,
            number_lines,
            shortest_seq,
            longest_seq,
            random_subsample,
            print_statistics,
            no_outputfile,
            silent,
            remove_comma_from_header,
        } => {
            if !non_unique_header && matches!(duplicate_record, DuplicateAction::Ignore) {
                return Err(RfastaError::invalid_input(
                    "clean",
                    "cannot combine unique-header parsing with duplicate-record ignore mode",
                    "Pass --non-unique-header if repeated headers are expected, or use --duplicate-record fail/remove.",
                ));
            }

            let records = parse_fasta_file(
                filename,
                ParseOptions {
                    expect_unique_header: !non_unique_header,
                },
                !silent,
            )?;

            let cleaned = clean_sequences(
                records,
                &CleanOptions {
                    invalid_sequence_action: invalid_sequence,
                    duplicate_record_action: duplicate_record,
                    duplicate_sequence_action: duplicate_sequence,
                    shortest_seq,
                    longest_seq,
                    random_subsample,
                    remove_comma_from_header,
                    alignment: false,
                    verbose: !silent,
                    correction_dictionary: None,
                },
            )?;

            if print_statistics && !silent {
                println!("Total sequences: {}", cleaned.len());
                if let Some(shortest) = cleaned.iter().map(|record| record.sequence.len()).min() {
                    println!("Shortest sequence: {shortest}");
                }
                if let Some(longest) = cleaned.iter().map(|record| record.sequence.len()).max() {
                    println!("Longest sequence: {longest}");
                }
            }

            if !no_outputfile {
                if let Some(output) = output {
                    write_fasta_file(
                        &cleaned,
                        output,
                        WriteOptions {
                            line_length: number_lines.or(Some(DEFAULT_LINE_LENGTH)),
                            append: false,
                        },
                        !silent,
                    )?;
                }
            }
        }
        Commands::Split {
            filename,
            output_dir,
            chunks,
            line_length,
            no_outputfiles,
            silent,
        } => {
            if !no_outputfiles {
                split_fasta_file_round_robin(
                    filename,
                    output_dir,
                    chunks,
                    Some(line_length),
                    !silent,
                )?;
            }
        }
    }

    Ok(())
}
