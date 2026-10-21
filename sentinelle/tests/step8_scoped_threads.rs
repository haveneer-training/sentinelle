//! Étape 8 : `thread::scope`, des threads qui empruntent le texte.
#![cfg(feature = "step8")]

mod common;

use sentinelle::summary::{Summary, split_chunks, summarize, summarize_scoped};
use std::collections::{HashMap, HashSet};

#[test]
fn chunks_cover_the_text_on_line_boundaries() {
    let text = common::dataset().auth.as_slice();
    for parts in [1, 2, 3, 7, 64] {
        let chunks = split_chunks(text, parts);
        assert!(
            !chunks.is_empty() && chunks.len() <= parts,
            "{} morceaux pour {parts}",
            chunks.len()
        );
        assert_eq!(chunks.concat(), text);
        for chunk in &chunks[..chunks.len() - 1] {
            assert_eq!(chunk.last(), Some(&b'\n'));
        }
    }
}

#[test]
fn chunks_of_small_or_odd_texts() {
    assert!(split_chunks(b"", 4).is_empty());
    assert_eq!(split_chunks(b"a\nb\nc", 2), vec![&b"a\nb\n"[..], &b"c"[..]]);
    assert_eq!(
        split_chunks(b"one long line", 3),
        vec![&b"one long line"[..]]
    );
    assert_eq!(split_chunks(b"a\nb\n", 10).concat(), b"a\nb\n");
}

#[test]
fn merge_adds_counts_and_unites_accounts() {
    let ip = "192.0.2.1".parse().unwrap();
    let mut left: Summary<&str> = Summary {
        lines: 3,
        errors: 1,
        failures_by_ip: HashMap::from([(ip, 2)]),
        accounts_by_ip: HashMap::from([(ip, HashSet::from(["a", "b"]))]),
    };
    let right = Summary {
        lines: 4,
        errors: 0,
        failures_by_ip: HashMap::from([(ip, 1)]),
        accounts_by_ip: HashMap::from([(ip, HashSet::from(["b", "c"]))]),
    };
    left.merge(right);
    assert_eq!(
        (left.lines, left.errors, left.failures_by_ip[&ip]),
        (7, 1, 3)
    );
    assert_eq!(left.accounts_by_ip[&ip], HashSet::from(["a", "b", "c"]));
}

#[test]
fn parallel_summary_equals_sequential() {
    for data in [common::dataset(), common::dataset_with_invalid_utf8()] {
        let expected = summarize(&data.auth);
        for threads in [1, 3, 8] {
            assert_eq!(
                summarize_scoped(&data.auth, threads),
                expected,
                "{threads} threads"
            );
        }
    }
}
