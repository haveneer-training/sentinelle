//! Étape 5 : ensembles (`HashSet`) et fenêtre glissante (`VecDeque`).
#![cfg(feature = "step5")]

mod common;

use sentinelle::auth::{AuthEntry, AuthEvent};
use sentinelle::detect::{detect_bruteforce, detect_spraying};
use sentinelle::journal::parse_auth;
use std::net::IpAddr;

fn failure(timestamp: i64, ip: &str, user: &'static str) -> AuthEntry<'static> {
    AuthEntry {
        timestamp,
        host: "bastion-01",
        process: "sshd",
        pid: Some(1),
        event: AuthEvent::Failed {
            user,
            ip: ip.parse().unwrap(),
            invalid_user: false,
        },
    }
}

#[test]
fn spraying_counts_distinct_accounts() {
    let entries = [
        failure(10, "192.0.2.1", "a"),
        failure(20, "192.0.2.1", "a"),
        failure(30, "192.0.2.1", "b"),
        failure(5, "192.0.2.1", "c"),
        failure(40, "192.0.2.2", "a"),
    ];
    let found = detect_spraying(&entries, 3);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].accounts, found[0].first, found[0].last),
        (3, 5, 30)
    );
    assert_eq!(detect_spraying(&entries, 4), vec![]);
}

#[test]
fn bruteforce_needs_the_threshold_within_the_window() {
    // 5 échecs espacés de 10 s : rafale pour une fenêtre de 60 s, pas de 30 s
    let entries: Vec<_> = (0..5)
        .map(|i| failure(100 + 10 * i, "192.0.2.1", "a"))
        .collect();
    assert_eq!(detect_bruteforce(&entries, 30, 5), vec![]);
    let found = detect_bruteforce(&entries, 60, 5);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].first, found[0].last, found[0].failures),
        (100, 140, 5)
    );
}

#[test]
fn a_burst_is_reported_once_and_a_pause_splits_it() {
    let mut entries: Vec<_> = (0..20).map(|i| failure(i, "192.0.2.1", "a")).collect();
    entries.extend((0..20).map(|i| failure(1000 + i, "192.0.2.1", "a")));
    entries.push(failure(5, "192.0.2.9", "z"));
    let found = detect_bruteforce(&entries, 60, 10);
    let summary: Vec<_> = found
        .iter()
        .map(|b| (b.first, b.last, b.failures))
        .collect();
    assert_eq!(summary, vec![(0, 19, 20), (1000, 1019, 20)]);
}

#[test]
fn finds_the_scenario_spraying_and_bruteforce() {
    let data = common::dataset();
    let scenario = &data.generated.scenario;
    let journal = parse_auth(&data.auth);

    let spraying = detect_spraying(&journal.entries, 20);
    assert_eq!(spraying.len(), 1, "{spraying:?}");
    assert_eq!(spraying[0].ip, IpAddr::V4(scenario.spray_ip));
    assert_eq!(spraying[0].accounts, scenario.spray.2);

    let bursts = detect_bruteforce(&journal.entries, 300, 20);
    assert_eq!(bursts.len(), 1, "{bursts:?}");
    assert_eq!(bursts[0].ip, IpAddr::V4(scenario.brute_ip));
    assert_eq!(bursts[0].first, scenario.brute.0);
    assert_eq!(bursts[0].failures, scenario.brute.2 as usize);
}
