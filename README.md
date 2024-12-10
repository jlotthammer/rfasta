# rfasta: Rust-powered protein FASTA parser library and CLI.

**rfasta** is designed for bioinformaticians and protein scientists who need a fast,
reliable tool for parsing, cleaning, and manipulating protein sequence FASTA files.

rfasta is a direct port of the python package [protfasta](https://github.com/holehouse-lab/protfasta) into rust. This greatly improves the performance of parsing, cleaning, and manipulation of LARGE protein sequence fasta files such as those from uniref.

## Example Usage

### **Sanitize fasta file - removes duplicate sequences, invalid sequences [in this case all sequences with noncanonical amino acids]**
```bash
rfasta clean --non-unique-header --duplicate-record remove --invalid-sequence remove  test.fasta -o output.fasta
```
#### Output
```markdown
[INFO]: Read in file with 100005 lines
[INFO]: Parsed file to recover 11085 sequences
[INFO]: Removed 68 of 11085 sequences due to invalid characters
[INFO]: Removed 1 of 11017 sequences due to duplicate records
[INFO]: Wrote 11016 sequences to output.fasta
```


### **Shard fasta file into smaller chunkers**
```bash
rfasta split --output-dir . --chunks 3 output.fasta
```
#### Output
```markdown
[INFO]: Read in file with 110670 lines
[INFO]: Parsed file to recover 11016 sequences
[INFO]: Wrote 3672 sequences to ./output_000001.fasta
[INFO]: Wrote 3672 sequences to ./output_000002.fasta
[INFO]: Wrote 3672 sequences to ./output_000003.fasta
[INFO]: Split FASTA into 3 chunks
```

## Changelog

# v0.1.0-beta (Initial Release)
# Initial beta release of rfasta.
- Core functionality for:
  - Parsing: Read and interpret protein FASTA files efficiently.
  - Cleaning: Remove invalid entries and ensure sequences conform to biological standards.
  - Manipulation: Efficient fasta sharding operations on large protein sequence fasta files.
- Rust CLI integration for command-line use cases.
- Python bindings via PyO3 for seamless Python library integration.
- High performance with optimized parsing for large-scale FASTA files (e.g., UniRef datasets).
- Early-stage development—additional features, documentation, and pypi deployment to follow in subsequent releases.
