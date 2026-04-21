# Getting Started

This page shows the fastest way to use `rfasta` from Rust, Python, and the CLI.

## Build the Rust crate

```bash
cargo build
```

Run the test suite:

```bash
cargo test
```

## Install the Python module locally

From a Python environment with Rust installed:

```bash
maturin develop --features python-module,cli
```

This installs the extension module and the `rfasta` command into the active environment.

## Rust example

```rust
use std::io::Cursor;

use rfasta::clean::{clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction};
use rfasta::parse::{parse_fasta_reader, ParseOptions};
use rfasta::write::{write_fasta_writer, WriteOptions};

let input = b">seq1\nACDX\n>seq2\nTTTT\n";
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
# Ok::<(), rfasta::RfastaError>(())
```

Use Rust when you want direct integration into services, jobs, or internal libraries with full
control over parsing, cleaning, writing, and sharding behavior.

## Python example

```python
import rfasta

rows = rfasta.read_fasta("proteins.fasta", expect_unique_header=True, verbose=False)
rfasta.write_fasta(rows, "proteins.copy.fasta", line_length=60)
```

Use Python when you want simple file-oriented workflows or compatibility with existing Python-based
analysis notebooks and scripts.

## CLI example

```bash
rfasta clean proteins.fasta \
  --non-unique-header \
  --duplicate-record remove \
  --invalid-sequence convert-remove \
  -o cleaned.fasta

rfasta split cleaned.fasta --output-dir shards --chunks 8
```

Use the CLI for repeatable shell workflows, pipeline steps, or operational batch jobs.

## Which interface should I choose?

- Choose Rust for embedding `rfasta` in software with full API-level control.
- Choose Python for notebook and scripting workflows.
- Choose the CLI for automation, shell pipelines, and production batch jobs.
