//! Étape 3 : analyser un journal entier, sans s'arrêter aux lignes défectueuses.
#![cfg(feature = "step3")]

mod common;

use sentinelle::ParseError;
use sentinelle::auth::AuthEvent;
use sentinelle::journal::{parse_auth, parse_lines, parse_web};

#[test]
fn numbers_lines_from_one_and_keeps_going() {
    let text = b"1\n\nx\n4\n";
    let journal = parse_lines(text, |line: &str| line.parse::<u32>().map_err(|_| ParseError::BadNumber("n")));
    assert_eq!(journal.entries, vec![1, 4]);
    assert_eq!(journal.errors, vec![(2, ParseError::BadNumber("n")), (3, ParseError::BadNumber("n"))]);
}

#[test]
fn invalid_utf8_is_an_error_not_a_panic() {
    let text = b"ok\n\xFFko\nok\n";
    let journal = parse_lines(text, |line: &str| Ok(line.len()));
    assert_eq!(journal.entries, vec![2, 2]);
    assert_eq!(journal.errors, vec![(2, ParseError::InvalidUtf8)]);
}

#[test]
fn accounts_for_every_generated_line() {
    let data = common::dataset();
    let auth = parse_auth(&data.auth);
    let web = parse_web(&data.access);
    let counts = (data.generated.auth, data.generated.access);
    assert_eq!((auth.entries.len() + auth.errors.len()) as u64, counts.0.lines);
    assert_eq!((web.entries.len() + web.errors.len()) as u64, counts.1.lines);
    // seules des lignes corrompues peuvent être rejetées (une corruption peut passer inaperçue)
    assert!(auth.errors.len() as u64 <= counts.0.corrupted);
    assert!(web.errors.len() as u64 <= counts.1.corrupted);
}

#[test]
fn survives_invalid_utf8_in_a_real_journal() {
    let data = common::dataset_with_invalid_utf8();
    let auth = parse_auth(&data.auth);
    assert!(auth.errors.iter().any(|&(_, e)| e == ParseError::InvalidUtf8));
    let accepted = auth.entries.iter().filter(|e| matches!(e.event, AuthEvent::Accepted { .. })).count();
    assert!(accepted > 100, "{accepted} connexions réussies seulement");
}
