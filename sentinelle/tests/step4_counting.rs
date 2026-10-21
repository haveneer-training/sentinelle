//! Étape 4 : compter avec une `HashMap`, garder les meilleurs avec un `BinaryHeap`.
#![cfg(feature = "step4")]

mod common;

use sentinelle::auth::parse_auth_line;
use sentinelle::detect::{failures_by_ip, top_n};
use sentinelle::journal::parse_auth;
use std::collections::HashMap;
use std::net::IpAddr;

#[test]
fn counts_an_invalid_user_attempt_once() {
    let lines = [
        "2026-11-09T00:00:01Z bastion-01 sshd[1]: Invalid user oracle from 192.0.2.7 port 1",
        "2026-11-09T00:00:03Z bastion-01 sshd[1]: Failed password for invalid user oracle from 192.0.2.7 port 1 ssh2",
        "2026-11-09T00:00:09Z bastion-01 sshd[1]: Failed password for root from 192.0.2.7 port 1 ssh2",
        "2026-11-09T00:00:12Z bastion-01 sshd[2]: Accepted password for bob from 192.0.2.8 port 2 ssh2",
    ];
    let entries: Vec<_> = lines.iter().map(|l| parse_auth_line(l).unwrap()).collect();
    let counts = failures_by_ip(&entries);
    let ip: IpAddr = "192.0.2.7".parse().unwrap();
    assert_eq!(counts, HashMap::from([(ip, 2)]));
}

#[test]
fn top_n_orders_by_value_then_key() {
    let counts = HashMap::from([("d", 1), ("b", 5), ("a", 3), ("c", 5), ("e", 3)]);
    assert_eq!(top_n(counts.clone(), 3), vec![("b", 5), ("c", 5), ("a", 3)]);
    assert_eq!(top_n(&counts, 1), vec![(&"b", &5)]);
    assert_eq!(top_n(counts.clone(), 0), vec![]);
    assert_eq!(top_n(counts, 10).len(), 5);
}

#[test]
fn top_n_matches_a_full_sort() {
    let values: Vec<(u32, u32)> = (0..1000).map(|i| (i, (i * 7919) % 97)).collect();
    let mut sorted = values.clone();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    sorted.truncate(25);
    assert_eq!(top_n(values, 25), sorted);
}

#[test]
fn brute_force_address_comes_first() {
    let data = common::dataset();
    let journal = parse_auth(&data.auth);
    let top = top_n(failures_by_ip(&journal.entries), 2);
    let scenario = &data.generated.scenario;
    assert_eq!(top[0], (IpAddr::V4(scenario.brute_ip), scenario.brute.2 as usize));
    assert_eq!(top[1].0, IpAddr::V4(scenario.spray_ip));
}
