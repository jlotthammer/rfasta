# rfasta

`rfasta` is a guide-first toolkit for parsing, cleaning, writing, and sharding protein FASTA
files. It is designed for large sequential workloads such as UniRef-style datasets while still
supporting Python and CLI workflows that feel close to `protfasta`.

[Get Started](getting-started.md){ .md-button .md-button--primary }
[Rust API Reference](api-reference.md){ .md-button }

## Choose an Interface

<div class="grid cards" markdown>

-   :material-language-rust: __Use from Rust__

    Use the small module-first API:

    - `rfasta::parse`
    - `rfasta::clean`
    - `rfasta::write`
    - `rfasta::shard`

    ```rust
    use std::io::Cursor;

    use rfasta::parse::{parse_fasta_reader, ParseOptions};

    let records = parse_fasta_reader(Cursor::new(b">seq1\nAAAA\n"), ParseOptions::default())?;
    # Ok::<(), rfasta::RfastaError>(())
    ```

    [Rust guide](rust.md)

-   :material-language-python: __Use from Python__

    Use the PyO3 extension module for file-oriented read/write helpers and sequence utilities.

    ```python
    import rfasta

    rows = rfasta.read_fasta("proteins.fasta", expect_unique_header=True)
    rfasta.write_fasta(rows, "copy.fasta")
    ```

    [Python guide](python.md)

-   :material-console: __Use from the CLI__

    Use the executable when you want one-off cleanup or sharding jobs.

    ```bash
    rfasta clean proteins.fasta -o cleaned.fasta --invalid-sequence convert-remove
    rfasta split cleaned.fasta --output-dir shards --chunks 8
    ```

    [CLI guide](cli.md)

</div>

## Why rfasta

- Handles very large protein FASTA inputs reliably
- Clear CLI, Python, and Rust interfaces for different teams and workflows
- Configurable duplicate and invalid-residue cleanup policies
- Consistent FASTA writing with configurable line wrapping
- Documentation designed for onboarding and production operations

## Large FASTA Operations

For large sequential FASTA workloads, `rfasta split` is designed to keep throughput high and
operational complexity low. Use it to create shard sets for parallel pipelines without adding
extra preprocessing steps.

[Large FASTA guidance](large-fasta.md)
