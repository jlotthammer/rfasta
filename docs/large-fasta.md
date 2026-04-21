# Large FASTA Workflows

This page focuses on UniRef-scale or similarly large protein FASTA inputs.

## Capacity Planning

Use the right path for the job:

- Parsing: suitable for very large files and iterative workflows
- Cleaning: applies full-dataset validation and policy rules
- Writing: produces standardized FASTA output for downstream systems
- Sharding: creates balanced shard sets for parallel processing

## Why sharding works well in production

`rfasta split` is designed to keep large ingestion jobs simple and reliable:

- avoids expensive pre-processing stages
- creates predictable shard layout for orchestrators and schedulers
- supports high-throughput workflows on large protein datasets

## What “balanced” means

Shard balancing is based on record order (not exact byte size). That means:

- record counts are usually close across shards
- shard sizes can differ when sequence lengths vary
- behavior is predictable and operationally stable

If strict byte-equal shard sizing is required, treat that as a separate post-processing strategy.

## Memory expectations

- Parsing and splitting are suitable for very large files.
- Cleaning memory usage depends on records retained after filtering.
- Plan worker memory based on dataset size and cleanup policy settings.

## Practical recommendations

- Clean once, split once, and treat the cleaned FASTA as your reproducible source artifact.
- Prefer local or sequential storage for large split jobs.
- Use shard counts that align with downstream parallelism rather than arbitrary file counts.
