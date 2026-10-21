//! Étape 9 : un pipeline de threads reliés par des canaux `mpsc`.
#![cfg(feature = "step9")]

mod common;

use sentinelle::summary::{Summary, summarize, summarize_pipeline};
use std::collections::HashSet;
use std::io::Cursor;
use std::time::{Duration, Instant};

/// Le résumé séquentiel, avec des noms de compte possédés.
fn owned(summary: Summary<&str>) -> Summary<String> {
    Summary {
        lines: summary.lines,
        errors: summary.errors,
        failures_by_ip: summary.failures_by_ip,
        accounts_by_ip: summary
            .accounts_by_ip
            .into_iter()
            .map(|(ip, accounts)| {
                (
                    ip,
                    accounts
                        .into_iter()
                        .map(String::from)
                        .collect::<HashSet<_>>(),
                )
            })
            .collect(),
    }
}

#[test]
fn pipeline_equals_sequential() {
    for data in [common::dataset(), common::dataset_with_invalid_utf8()] {
        let expected = owned(summarize(&data.auth));
        for (workers, lines_per_block) in [(1, 1_000), (4, 500), (8, 7)] {
            let summary =
                summarize_pipeline(Cursor::new(&data.auth), workers, lines_per_block).unwrap();
            assert_eq!(
                summary, expected,
                "{workers} travailleurs, blocs de {lines_per_block} lignes"
            );
        }
    }
}

#[test]
fn empty_input_terminates() {
    // un pipeline qui oublie de fermer un canal ne se termine jamais : on borne l'attente
    let started = Instant::now();
    let summary = summarize_pipeline(Cursor::new(Vec::new()), 4, 100).unwrap();
    assert_eq!(summary, Summary::default());
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn read_errors_are_reported() {
    struct Failing;
    impl std::io::Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disque débranché"))
        }
    }
    let result = summarize_pipeline(std::io::BufReader::new(Failing), 2, 10);
    assert_eq!(result.unwrap_err().to_string(), "disque débranché");
}
