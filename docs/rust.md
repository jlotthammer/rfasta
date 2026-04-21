# Rust Guide

The public Rust API is intentionally small and module-first.

## Modules

- `rfasta::parse`: read FASTA records from files or any `BufRead`
- `rfasta::clean`: duplicate and invalid-sequence policies
- `rfasta::write`: buffered FASTA output
- `rfasta::shard`: shard generation for parallel workflows
- `rfasta::error`: shared error type

## Parsing

Use the visitor API when you need high-throughput parse control:

```rust
use std::io::Cursor;

use rfasta::parse::{visit_fasta_reader, FastaRecord, ParseOptions};
use rfasta::RfastaError;

let mut count = 0usize;
visit_fasta_reader(Cursor::new(b">seq1\nAAAA\n>seq2\nCCCC\n"), ParseOptions::default(), |_record: FastaRecord| {
    count += 1;
    Ok::<(), RfastaError>(())
})?;
assert_eq!(count, 2);
# Ok::<(), rfasta::RfastaError>(())
```

Use `parse_fasta_reader` or `parse_fasta_file` when you want all records in a `Vec`.

## Cleaning

Cleaning applies full-record policy validation and is intended for high-confidence canonicalization
before downstream processing.

```rust
use rfasta::clean::{clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction};
use rfasta::parse::FastaRecord;

let cleaned = clean_sequences(
    vec![FastaRecord::new("seq1", "ACDX")],
    &CleanOptions {
        invalid_sequence_action: InvalidSequenceAction::ConvertRemove,
        duplicate_record_action: DuplicateAction::Fail,
        duplicate_sequence_action: DuplicateAction::Ignore,
        ..CleanOptions::default()
    },
)?;
assert_eq!(cleaned[0].sequence, "ACDG");
# Ok::<(), rfasta::RfastaError>(())
```

## Writing

`write_fasta_writer` writes to any `Write`, while `write_fasta_file` handles filesystem paths and
buffering for you.

```rust
use rfasta::parse::FastaRecord;
use rfasta::write::{write_fasta_writer, WriteOptions};

let mut out = Vec::new();
write_fasta_writer(&mut out, &[FastaRecord::new("seq1", "AAAA")], &WriteOptions::default())?;
# Ok::<(), rfasta::RfastaError>(())
```

## Sharding

The sharding API is file-oriented and built for large production datasets:

```rust
use rfasta::shard::split_fasta_file_round_robin;

split_fasta_file_round_robin("proteins.fasta", "shards", 8, Some(60), true)?;
# Ok::<(), rfasta::RfastaError>(())
```

Shards are balanced by record order, which provides stable behavior for parallel workflows.

## Memory and I/O behavior

- Parsing and writing are suitable for large-file operations.
- Cleaning resource use is driven by retained dataset size and selected policies.
- Sharding behavior is deterministic and workflow-friendly.

## API Reference

Use rustdoc for symbol-level details:

- [Rust API reference](api-reference.md)
