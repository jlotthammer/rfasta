use rfasta::clean::{clean_sequences, CleanOptions, DuplicateAction, InvalidSequenceAction};
use rfasta::parse::FastaRecord;

#[test]
fn duplicate_record_removal_only_drops_exact_duplicate_records() {
    let records = vec![
        FastaRecord::new("seq1", "AAAA"),
        FastaRecord::new("seq1", "AAAA"),
        FastaRecord::new("seq1", "CCCC"),
    ];
    let cleaned = clean_sequences(
        records,
        &CleanOptions {
            duplicate_record_action: DuplicateAction::Remove,
            duplicate_sequence_action: DuplicateAction::Ignore,
            invalid_sequence_action: InvalidSequenceAction::Fail,
            ..CleanOptions::default()
        },
    )
    .unwrap();
    assert_eq!(cleaned.len(), 2);
    assert_eq!(cleaned[0].sequence, "AAAA");
    assert_eq!(cleaned[1].sequence, "CCCC");
}

#[test]
fn duplicate_sequence_failure_reports_both_headers() {
    let records = vec![
        FastaRecord::new("seq1", "AAAA"),
        FastaRecord::new("seq2", "AAAA"),
    ];
    let error = clean_sequences(
        records,
        &CleanOptions {
            duplicate_record_action: DuplicateAction::Ignore,
            duplicate_sequence_action: DuplicateAction::Fail,
            invalid_sequence_action: InvalidSequenceAction::Fail,
            ..CleanOptions::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("seq1"));
    assert!(error.to_string().contains("seq2"));
}

#[test]
fn convert_remove_fixes_known_residues_and_drops_remaining_invalid_records() {
    let records = vec![
        FastaRecord::new("seq1", "ACDX"),
        FastaRecord::new("seq2", "ACD?"),
    ];
    let cleaned = clean_sequences(
        records,
        &CleanOptions {
            invalid_sequence_action: InvalidSequenceAction::ConvertRemove,
            duplicate_record_action: DuplicateAction::Ignore,
            duplicate_sequence_action: DuplicateAction::Ignore,
            ..CleanOptions::default()
        },
    )
    .unwrap();
    assert_eq!(cleaned.len(), 1);
    assert_eq!(cleaned[0].sequence, "ACDG");
}

#[test]
fn remove_comma_and_length_filters_apply_after_cleaning() {
    let records = vec![
        FastaRecord::new("seq,1", "AAAA"),
        FastaRecord::new("seq,2", "AAAAAA"),
    ];
    let cleaned = clean_sequences(
        records,
        &CleanOptions {
            duplicate_record_action: DuplicateAction::Ignore,
            duplicate_sequence_action: DuplicateAction::Ignore,
            invalid_sequence_action: InvalidSequenceAction::Fail,
            shortest_seq: Some(5),
            remove_comma_from_header: true,
            ..CleanOptions::default()
        },
    )
    .unwrap();
    assert_eq!(cleaned.len(), 1);
    assert_eq!(cleaned[0].header, "seq;2");
}

#[test]
fn alignment_mode_accepts_gap_residues() {
    let cleaned = clean_sequences(
        vec![FastaRecord::new("seq1", "ACD-E")],
        &CleanOptions {
            alignment: true,
            invalid_sequence_action: InvalidSequenceAction::Fail,
            duplicate_record_action: DuplicateAction::Ignore,
            duplicate_sequence_action: DuplicateAction::Ignore,
            ..CleanOptions::default()
        },
    )
    .unwrap();
    assert_eq!(cleaned[0].sequence, "ACD-E");

    let error = clean_sequences(
        vec![FastaRecord::new("seq1", "ACD-E")],
        &CleanOptions {
            alignment: false,
            invalid_sequence_action: InvalidSequenceAction::Fail,
            duplicate_record_action: DuplicateAction::Ignore,
            duplicate_sequence_action: DuplicateAction::Ignore,
            ..CleanOptions::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("invalid residue"));
}
