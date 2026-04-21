# Python Guide

The Python module is intended for file-oriented workflows, scripting, and sequence utility tasks.
This guide covers the supported user-facing Python API.

## Install locally

```bash
maturin develop --features python-module,cli
```

## Read and write FASTA

```python
import rfasta

rows = rfasta.read_fasta(
    "proteins.fasta",
    expect_unique_header=True,
    verbose=False,
)

rfasta.write_fasta(
    rows,
    "proteins.copy.fasta",
    line_length=60,
    verbose=False,
)
```

The return shape for `read_fasta` is a list of `[header, sequence]` pairs to stay close to
existing `protfasta` expectations.

## Utility helpers

The Python module also exposes low-level sequence utilities such as:

- `convert_to_valid`
- `check_sequence_is_valid`
- `convert_invalid_sequences`
- `remove_invalid_sequences`

These are useful when your data is already in Python and you want consistent sequence policy
behavior across scripting and pipeline jobs.

## Compatibility with `protfasta`

`rfasta` keeps the broad workflow model of `protfasta`:

- duplicate handling policies
- invalid sequence conversion/removal policies
- file-oriented read/write helpers

But there are some intentional differences:

- errors include more context and explicit recovery hints
- Rust and CLI interfaces expose a broader set of workflow controls
- Python keeps a concise, task-focused surface for scripting and notebooks

## Current limitations

- The Python surface is narrower than the full Rust API.
- Sharding workflows are currently documented primarily through Rust and CLI guides.
- Python users still receive the underlying `rfasta` help text in exceptions, but the module does
  not yet expose every Rust entry point directly.
