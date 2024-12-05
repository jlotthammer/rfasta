// config.rs
use std::collections::{HashMap, HashSet};

pub const STANDARD_CONVERSION: [(&str, &str); 6] = [
    ("B", "N"),
    ("U", "C"),
    ("X", "G"),
    ("Z", "Q"),
    ("*", ""),
    ("-", ""),
];

pub const STANDARD_CONVERSION_WITH_GAP: [(&str, &str); 6] = [
    ("B", "N"),
    ("U", "C"),
    ("X", "G"),
    ("Z", "Q"),
    (" ", ""),
    ("*", ""),
];

pub const STANDARD_AAS: [&char; 20] = [
    &'A', &'C', &'D', &'E', &'F', &'G', &'H', &'I', &'K', &'L', &'M', &'N', &'P', &'Q', &'R', &'S', &'T', &'V', &'W', &'Y',
];

pub const STANDARD_AAS_WITH_GAP: [&char; 21] = [
    &'A', &'C', &'D', &'E', &'F', &'G', &'H', &'I', &'K', &'L', &'M', &'N', &'P', &'Q', &'R', &'S', &'T', &'V', &'W', &'Y', &'-',
];

pub fn get_standard_aas() -> HashSet<&'static char> {
    STANDARD_AAS.iter().cloned().collect()
}

pub fn get_standard_aas_with_gap() -> HashSet<&'static char> {
    STANDARD_AAS_WITH_GAP.iter().cloned().collect()
}


// Convert the arrays into HashMap and HashSet for easy lookup
pub fn get_standard_conversion() -> HashMap<&'static str, &'static str> {
    STANDARD_CONVERSION.iter().cloned().collect()
}

pub fn get_standard_conversion_with_gap() -> HashMap<&'static str, &'static str> {
    STANDARD_CONVERSION_WITH_GAP.iter().cloned().collect()
}