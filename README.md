# rfasta

`rfasta` is a production-ready toolkit for parsing, cleaning, writing, and sharding protein FASTA
files.

It provides:

- A high-performance Rust library
- A Python package for scripting and notebooks
- A CLI for batch and pipeline workflows

## Highlights

- FASTA parsing from files or in-memory readers
- Configurable cleanup policies for duplicate records and invalid residues
- Deterministic shard generation for parallel processing
- Python bindings via PyO3
- CLI designed for operational workflows
- Documentation for Rust, Python, and CLI usage

## Installation

### Python (recommended for end users)

```bash
pip install rfasta
```

### Rust (library development)

Add to `Cargo.toml`:

```toml
[dependencies]
rfasta = "0.1"
```

## Quick Start

### CLI

Clean a FASTA file:

```bash
rfasta clean proteins.fasta \
  --non-unique-header \
  --duplicate-record remove \
  --invalid-sequence convert-remove \
  -o cleaned.fasta
```

Split a large FASTA file into one-pass round-robin shards:

```bash
rfasta split proteins.fasta --output-dir shards --chunks 8
```

### Python

```python
import rfasta

rows = rfasta.read_fasta("proteins.fasta", expect_unique_header=True, verbose=False)
rfasta.write_fasta(rows, "proteins.copy.fasta", line_length=60)
```

### Rust

```rust
use std::io::Cursor;

use rfasta::clean::{clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction};
use rfasta::parse::{parse_fasta_reader, ParseOptions};
use rfasta::write::{write_fasta_writer, WriteOptions};

let input = b">seq1\nacdx\n>seq2\nTTTT\n";
let records = parse_fasta_reader(Cursor::new(input), ParseOptions::default())?;
let cleaned = clean_sequences(
    records,
    &CleanOptions {
        invalid_sequence_action: InvalidSequenceAction::ConvertRemove,
        duplicate_record_action: DuplicateAction::Fail,
        ..CleanOptions::default()
    },
)?;
let mut output = Vec::new();
write_fasta_writer(&mut output, &cleaned, &WriteOptions::default())?;
# Ok::<(), rfasta::RfastaError>(())
```

## Documentation and Guides

- User guides: [`docs/`](./docs)
- API reference entry point: [`docs/api-reference.md`](./docs/api-reference.md)
- Release and PyPI publishing runbook: [`RELEASING.md`](./RELEASING.md)

Build the guide site locally:

```bash
pip install -r docs/requirements.txt
mkdocs build --strict
```

## Local Performance Check

For a repo-local benchmark driver that does not require extra benchmark crates, run:

```bash
cargo run --release --example benchmark_driver
```
