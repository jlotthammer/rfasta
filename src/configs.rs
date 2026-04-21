use std::collections::HashMap;

/// Default FASTA line length used by the CLI and examples.
pub const DEFAULT_LINE_LENGTH: usize = 60;

/// Minimum line length accepted for wrapped FASTA output.
pub const MIN_LINE_LENGTH: usize = 5;

/// Canonical amino acids for protein FASTA validation.
pub const STANDARD_AAS: [char; 20] = [
    'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W',
    'Y',
];

/// Canonical amino acids plus gap for alignment-aware validation.
pub const STANDARD_AAS_WITH_GAP: [char; 21] = [
    'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W',
    'Y', '-',
];

/// Default conversion map used when `alignment = false`.
#[cfg_attr(not(feature = "python"), allow(dead_code))]
pub const STANDARD_CONVERSION: [(&str, &str); 7] = [
    ("B", "N"),
    ("U", "C"),
    ("X", "G"),
    ("Z", "Q"),
    ("*", ""),
    ("-", ""),
    (" ", ""),
];

/// Default conversion map used when `alignment = true`.
#[cfg_attr(not(feature = "python"), allow(dead_code))]
pub const STANDARD_CONVERSION_WITH_GAP: [(&str, &str); 6] = [
    ("B", "N"),
    ("U", "C"),
    ("X", "G"),
    ("Z", "Q"),
    ("*", ""),
    (" ", ""),
];

/// Returns `true` if a residue is valid for the selected mode.
pub fn is_valid_residue(residue: char, alignment: bool) -> bool {
    let residue = residue.to_ascii_uppercase();
    if alignment {
        STANDARD_AAS_WITH_GAP.contains(&residue)
    } else {
        STANDARD_AAS.contains(&residue)
    }
}

/// Returns the standard replacement for a residue when no custom correction dictionary is provided.
pub fn standard_replacement(residue: char, alignment: bool) -> Option<&'static str> {
    match residue.to_ascii_uppercase() {
        'B' => Some("N"),
        'U' => Some("C"),
        'X' => Some("G"),
        'Z' => Some("Q"),
        '*' | ' ' => Some(""),
        '-' if !alignment => Some(""),
        _ => None,
    }
}

/// Returns the standard conversion dictionary as owned strings.
#[cfg_attr(not(feature = "python"), allow(dead_code))]
pub fn standard_conversion_map(alignment: bool) -> HashMap<String, String> {
    let entries = if alignment {
        STANDARD_CONVERSION_WITH_GAP.as_slice()
    } else {
        STANDARD_CONVERSION.as_slice()
    };
    entries
        .iter()
        .map(|(from, to)| ((*from).to_string(), (*to).to_string()))
        .collect()
}
