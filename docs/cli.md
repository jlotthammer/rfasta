# CLI Guide

The `rfasta` executable is intended for operational cleanup and sharding jobs.

## Clean a FASTA file

```bash
rfasta clean proteins.fasta \
  --non-unique-header \
  --duplicate-record remove \
  --duplicate-sequence ignore \
  --invalid-sequence convert-remove \
  -o cleaned.fasta
```

Useful flags:

- `--non-unique-header`: allow repeated headers during parsing
- `--duplicate-record {ignore,fail,remove}`
- `--duplicate-sequence {ignore,fail,remove}`
- `--invalid-sequence {ignore,fail,remove,convert,convert-ignore,convert-remove}`
- `--shortest-seq` / `--longest-seq`
- `--random-subsample`
- `--remove-comma-from-header`

## Split a FASTA file

```bash
rfasta split proteins.fasta --output-dir shards --chunks 8
```

The split command is built for production pipelines:

- predictable shard distribution for parallel workers
- stable file naming for downstream orchestration
- efficient processing for large FASTA inputs

## Large-file workflow

For very large inputs, a common pattern is:

1. Clean once into a canonical FASTA.
2. Split the cleaned FASTA into shard files.
3. Run downstream jobs against shard inputs in parallel.

Example:

```bash
rfasta clean raw.fasta -o cleaned.fasta --invalid-sequence convert-remove
rfasta split cleaned.fasta --output-dir shards --chunks 32
```

## Failure modes

CLI errors are intended to be actionable. They include:

- the operation that failed
- relevant file or header context when known
- a `help:` hint with the suggested next step

Examples:

- duplicate header during parse
- invalid residue during clean
- invalid chunk count during split
- unwritable output path during write
