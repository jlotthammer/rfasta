use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Error type used across parsing, cleaning, writing, and splitting operations.
#[derive(Debug)]
pub enum RfastaError {
    /// Underlying filesystem or stream I/O failure.
    Io {
        /// High-level operation being attempted.
        operation: &'static str,
        /// Path involved in the failure, when known.
        path: Option<PathBuf>,
        /// Original I/O error.
        source: io::Error,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Malformed FASTA structure such as sequence data before a header.
    MalformedFasta {
        /// Path being parsed, when known.
        path: Option<PathBuf>,
        /// 1-based line number where parsing failed.
        line_number: usize,
        /// Specific parse failure message.
        message: String,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Duplicate header encountered when unique headers were required.
    DuplicateHeader {
        /// Path being parsed, when known.
        path: Option<PathBuf>,
        /// Duplicate header text.
        header: String,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Empty sequence encountered while parsing or writing.
    EmptySequence {
        /// Path involved in the failure, when known.
        path: Option<PathBuf>,
        /// Header associated with the empty sequence.
        header: String,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Invalid residue encountered during cleaning or validation.
    InvalidSequence {
        /// Header associated with the invalid sequence.
        header: String,
        /// First invalid residue encountered.
        invalid_char: char,
        /// Whether alignment mode was enabled.
        alignment: bool,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Exact duplicate record encountered during cleaning.
    DuplicateRecord {
        /// Header associated with the duplicate record.
        header: String,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Duplicate sequence encountered across multiple headers.
    DuplicateSequence {
        /// Header for the first record containing the sequence.
        first_header: String,
        /// Header for the later duplicate sequence.
        duplicate_header: String,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Invalid shard count requested for a split operation.
    InvalidChunkCount {
        /// Invalid chunk count value.
        chunks: usize,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// Invalid user-provided record shape.
    InvalidRecord {
        /// Specific validation failure.
        message: String,
        /// User-facing recovery hint.
        hint: &'static str,
    },
    /// General invalid input error for a high-level operation.
    InvalidInput {
        /// High-level operation being attempted.
        operation: &'static str,
        /// Specific validation failure.
        message: String,
        /// User-facing recovery hint.
        hint: &'static str,
    },
}

impl RfastaError {
    pub fn io(
        operation: &'static str,
        path: Option<&Path>,
        source: io::Error,
        hint: &'static str,
    ) -> Self {
        Self::Io {
            operation,
            path: path.map(Path::to_path_buf),
            source,
            hint,
        }
    }

    pub fn malformed_fasta(
        path: Option<&Path>,
        line_number: usize,
        message: impl Into<String>,
        hint: &'static str,
    ) -> Self {
        Self::MalformedFasta {
            path: path.map(Path::to_path_buf),
            line_number,
            message: message.into(),
            hint,
        }
    }

    pub fn duplicate_header(
        path: Option<&Path>,
        header: impl Into<String>,
        hint: &'static str,
    ) -> Self {
        Self::DuplicateHeader {
            path: path.map(Path::to_path_buf),
            header: header.into(),
            hint,
        }
    }

    pub fn empty_sequence(
        path: Option<&Path>,
        header: impl Into<String>,
        hint: &'static str,
    ) -> Self {
        Self::EmptySequence {
            path: path.map(Path::to_path_buf),
            header: header.into(),
            hint,
        }
    }

    pub fn invalid_record(message: impl Into<String>, hint: &'static str) -> Self {
        Self::InvalidRecord {
            message: message.into(),
            hint,
        }
    }

    pub fn invalid_input(
        operation: &'static str,
        message: impl Into<String>,
        hint: &'static str,
    ) -> Self {
        Self::InvalidInput {
            operation,
            message: message.into(),
            hint,
        }
    }

    pub fn invalid_chunk_count(chunks: usize, hint: &'static str) -> Self {
        Self::InvalidChunkCount { chunks, hint }
    }
}

impl fmt::Display for RfastaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
                hint,
            } => {
                write!(f, "rfasta {operation} error")?;
                if let Some(path) = path {
                    write!(f, " for {}", path.display())?;
                }
                write!(f, ": {source}\nhelp: {hint}")
            }
            Self::MalformedFasta {
                path,
                line_number,
                message,
                hint,
            } => {
                write!(f, "rfasta parse error")?;
                if let Some(path) = path {
                    write!(f, " for {}", path.display())?;
                }
                write!(f, " at line {line_number}: {message}\nhelp: {hint}")
            }
            Self::DuplicateHeader { path, header, hint } => {
                write!(f, "rfasta parse error")?;
                if let Some(path) = path {
                    write!(f, " for {}", path.display())?;
                }
                write!(f, ": found duplicate header `{header}`\nhelp: {hint}")
            }
            Self::EmptySequence { path, header, hint } => {
                write!(f, "rfasta parse/write error")?;
                if let Some(path) = path {
                    write!(f, " for {}", path.display())?;
                }
                write!(f, ": sequence for header `{header}` is empty\nhelp: {hint}")
            }
            Self::InvalidSequence {
                header,
                invalid_char,
                alignment,
                hint,
            } => {
                write!(
                    f,
                    "rfasta clean error: invalid residue `{invalid_char}` found in header `{header}` (alignment mode: {alignment})\nhelp: {hint}"
                )
            }
            Self::DuplicateRecord { header, hint } => write!(
                f,
                "rfasta clean error: found duplicate record for header `{header}`\nhelp: {hint}"
            ),
            Self::DuplicateSequence {
                first_header,
                duplicate_header,
                hint,
            } => write!(
                f,
                "rfasta clean error: duplicate sequence found for headers `{first_header}` and `{duplicate_header}`\nhelp: {hint}"
            ),
            Self::InvalidChunkCount { chunks, hint } => write!(
                f,
                "rfasta split error: invalid chunk count `{chunks}`\nhelp: {hint}"
            ),
            Self::InvalidRecord { message, hint } => {
                write!(f, "rfasta record error: {message}\nhelp: {hint}")
            }
            Self::InvalidInput {
                operation,
                message,
                hint,
            } => write!(f, "rfasta {operation} error: {message}\nhelp: {hint}"),
        }
    }
}

impl Error for RfastaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
