//! Étape 10 (bonus) : état partagé (`Arc<Mutex<_>>`) et compteurs atomiques.
#![cfg(feature = "step10")]

mod common;

use sentinelle::summary::{
    Summary, chunk_ranges, summarize, summarize_shared, summarize_with_progress,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
fn ranges_cover_the_text() {
    let text = common::dataset().auth.as_slice();
    let ranges = chunk_ranges(text, 5);
    assert_eq!(ranges.first().map(|r| r.start), Some(0));
    assert_eq!(ranges.last().map(|r| r.end), Some(text.len()));
    assert!(ranges.windows(2).all(|w| w[0].end == w[1].start));
}

#[test]
fn shared_summary_equals_sequential() {
    for data in [common::dataset(), common::dataset_with_invalid_utf8()] {
        let expected = owned(summarize(&data.auth));
        let text = Arc::new(data.auth.clone());
        for threads in [1, 4] {
            assert_eq!(
                summarize_shared(Arc::clone(&text), threads),
                expected,
                "{threads} threads"
            );
        }
        assert_eq!(
            Arc::strong_count(&text),
            1,
            "les threads ont rendu leurs Arc"
        );
    }
}

#[test]
fn progress_is_monotonic_and_ends_with_the_total() {
    let data = common::dataset();
    let reports = Mutex::new(Vec::new());
    let summary = summarize_with_progress(&data.auth, 4, Duration::from_millis(1), |n| {
        reports.lock().unwrap().push(n)
    });
    assert_eq!(summary, summarize(&data.auth));
    let reports = reports.into_inner().unwrap();
    assert!(reports.windows(2).all(|w| w[0] <= w[1]), "{reports:?}");
    assert_eq!(reports.last(), Some(&summary.lines));
}
