use clap::{Parser, Subcommand};
use crate::{io, sequence_processing};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author = "jlotthammer",
    version,
    about = "rfasta is a simple command-line tool for parsing, sanitizing, and manipulating protein-based FASTA files. This project is a port of the python protfasta package to rust+py03 bindings.",
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clean a FASTA file [feature parity with protfasta]
    Clean {
        /// Input FASTA file
        filename: PathBuf,

        /// Output fasta file (is created)
        #[arg(short = 'o')]
        output: Option<PathBuf>,

        /// Allow non-unique headers
        #[arg(long)]
        non_unique_header: bool,

        /// How to deal with duplicate records
        #[arg(long, default_value = "fail")]
        duplicate_record: String,

        /// How to deal with duplicate sequences
        #[arg(long, default_value = "ignore")]
        duplicate_sequence: String,

        /// How to deal with invalid sequences
        #[arg(long, default_value = "fail")]
        invalid_sequence: String,

        /// Number of lines for FASTA file
        #[arg(long)]
        number_lines: Option<usize>,

        /// Shortest sequence included
        #[arg(long)]
        shortest_seq: Option<usize>,

        /// Longest sequence included
        #[arg(long)]
        longest_seq: Option<usize>,

        /// Randomly sub-sample from sequences
        #[arg(long)]
        random_subsample: Option<usize>,

        /// Print information on the sequences
        #[arg(long)]
        print_statistics: bool,

        /// Prevents rfasta from writing an output file
        #[arg(long)]
        no_outputfile: bool,

        /// Generate no output at all to STDOUT
        #[arg(long)]
        silent: bool,

        /// Replace commas in FASTA headers with semicolons
        #[arg(long)]
        remove_comma_from_header: bool,
    },
    /// Split a FASTA file into N approximately equal chunks
    Split {
        /// Input FASTA file
        filename: PathBuf,

        /// Output directory (is created if it doesn't exist)
        #[arg(short = 'o', long)]
        output_dir: PathBuf,

        /// Number of chunks to split into
        #[arg(short, long)]
        chunks: usize,

        /// Prevents rfasta from writing output files
        #[arg(long)]
        no_outputfiles: bool,

        /// Generate no output at all to STDOUT
        #[arg(long)]
        silent: bool,
    },
}

// Removed helper function definitions from cli.rs

/// The main entry point for the rfasta command-line interface.
///
/// Parses command-line arguments and executes the appropriate subcommands.
///
/// # Arguments
///
/// * `args` - A slice of command-line argument strings.
///
/// # Returns
///
/// * `Ok(())` if the command executes successfully.
/// * `Err(String)` with an error message if execution fails.
pub fn main(args: &[String]) -> Result<(), String> {
    let args = Args::parse_from(args);

    match &args.command {
        Commands::Clean {
            filename,
            output,
            non_unique_header,
            duplicate_record,
            duplicate_sequence,
            invalid_sequence,
            number_lines: _,
            shortest_seq,
            longest_seq,
            random_subsample,
            print_statistics,
            no_outputfile,
            silent,
            remove_comma_from_header,
        } => {
            // Parse the FASTA file
            let data = io::internal_parse_fasta_file(
                filename.to_str().unwrap_or_default(),
                !*non_unique_header,
                None,
                !*silent,
            )?;

            // Clean the sequences using the functions from sequence_processing.rs
            let cleaned_data = sequence_processing::clean_sequences(
                data,
                invalid_sequence,
                duplicate_record,
                duplicate_sequence,
                shortest_seq,
                longest_seq,
                random_subsample,
                *remove_comma_from_header,
                false,
                !*silent,
            )?;

            // Print statistics if requested
            if *print_statistics && !*silent {
                println!("Total sequences: {}", cleaned_data.len());
                if let Some(seq) = cleaned_data.iter().map(|s| s[1].len()).min() {
                    println!("Shortest sequence: {}", seq);
                }
                if let Some(seq) = cleaned_data.iter().map(|s| s[1].len()).max() {
                    println!("Longest sequence: {}", seq);
                }
            }

            // Write output file if requested
            if !*no_outputfile {
                if let Some(output_path) = output {
                    io::write_fasta(
                        cleaned_data,
                        output_path.to_str().unwrap_or_default(),
                        Some(60), // Default line length
                        !*silent,
                        false, // Don't append by default
                    )?;
                }
            }
        },
        Commands::Split {
            filename,
            output_dir,
            chunks,
            no_outputfiles,
            silent,
        } => {
            let fasta_data = io::internal_parse_fasta_file(
                filename.to_str().unwrap_or_default(),
                true,
                None,
                !silent,
            )?;

            let split_data = io::split_fasta(fasta_data, *chunks);

            if !*no_outputfiles {
                std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
                for (i, chunk) in split_data.iter().enumerate() {
                    let filename_stem = filename.file_stem().unwrap_or_default().to_str().unwrap_or_default();
                    let chunk_filename = output_dir.join(format!("{}_{}.fasta", filename_stem, format!("{:06}", i + 1)));
                    io::write_fasta(
                        chunk.clone(),
                        chunk_filename.to_str().unwrap_or_default(),
                        Some(60),
                        !silent,
                        false,
                    )?;
                }
            }

            if !silent {
                println!("[INFO]: Split FASTA into {} chunks", split_data.len());
            }
        },
    }

    Ok(())
}