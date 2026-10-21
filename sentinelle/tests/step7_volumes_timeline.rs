//! Étape 7 : `BTreeMap`, requêtes par intervalle, chronologie.
#![cfg(feature = "step7")]

mod common;

use sentinelle::detect::{Alert, Config, analyze, bytes_by_user_hour, timeline, user_volume};
use sentinelle::journal::{parse_auth, parse_web};
use sentinelle::time::HOUR;
use sentinelle::web::parse_web_line;
use std::collections::BTreeMap;

#[test]
fn sums_by_user_and_hour() {
    let lines = [
        r#"10.1.0.1 - alice [2026-11-09T09:00:00Z] "GET / HTTP/1.1" 200 100 1"#,
        r#"10.1.0.1 - alice [2026-11-09T09:59:59Z] "GET / HTTP/1.1" 200 10 1"#,
        r#"10.1.0.1 - alice [2026-11-09T10:00:00Z] "GET / HTTP/1.1" 200 1 1"#,
        r#"10.1.0.1 - alice [2026-11-09T10:00:00Z] "GET /x HTTP/1.1" 304 - 1"#,
        r#"10.1.0.1 - - [2026-11-09T10:00:00Z] "POST /login HTTP/1.1" 302 90 1"#,
    ];
    let entries: Vec<_> = lines.iter().map(|l| parse_web_line(l).unwrap()).collect();
    let volumes = bytes_by_user_hour(&entries);
    let nine = entries[0].timestamp;
    assert_eq!(volumes, BTreeMap::from([(("alice", nine), 110), (("alice", nine + HOUR), 1)]));
}

#[test]
fn range_queries_are_half_open() {
    let volumes = BTreeMap::from([(("a", 0), 1), (("a", HOUR), 2), (("a", 2 * HOUR), 4), (("b", HOUR), 8)]);
    assert_eq!(user_volume(&volumes, "a", 0, 2 * HOUR), 3);
    assert_eq!(user_volume(&volumes, "a", HOUR, 10 * HOUR), 6);
    assert_eq!(user_volume(&volumes, "b", 0, 10 * HOUR), 8);
    assert_eq!(user_volume(&volumes, "c", 0, 10 * HOUR), 0);
}

#[test]
fn timeline_is_sorted_and_stable() {
    let data = common::dataset();
    let auth = parse_auth(&data.auth);
    let web = parse_web(&data.access);
    let config = Config::starting_at(auth.entries[0].timestamp);
    let alerts = analyze(&auth.entries, &web.entries, &config);
    assert!(alerts.windows(2).all(|w| w[0].at() <= w[1].at()));

    // à instant égal, la compromission (liste 3) précède l'accès inédit (liste 4)
    let position = |pred: fn(&Alert) -> bool| alerts.iter().position(pred).unwrap();
    assert!(position(|a| matches!(a, Alert::Compromise(_))) < position(|a| matches!(a, Alert::NewAccess(n) if n.host == "bastion-01")));

    let reversed = timeline(vec![alerts.iter().rev().cloned().collect()]);
    assert_eq!(reversed.len(), alerts.len());
    assert!(reversed.windows(2).all(|w| w[0].at() <= w[1].at()));
}

#[test]
fn finds_the_exfiltration() {
    let data = common::dataset();
    let (start, end, _, total) = data.generated.scenario.exfil;
    let victim = common::victim(data);
    let web = parse_web(&data.access);
    let volumes = bytes_by_user_hour(&web.entries);
    let during = user_volume(&volumes, victim, start.div_euclid(HOUR) * HOUR, end + 1);
    assert!(during >= total && during < total + 50_000_000, "{during} octets pour {total} exfiltrés");
    assert!(during > u64::from(u32::MAX), "le volume dépasse u32::MAX : u64 obligatoire");

    let auth = parse_auth(&data.auth);
    let config = Config::starting_at(auth.entries[0].timestamp);
    let spikes: Vec<_> = analyze(&auth.entries, &web.entries, &config)
        .into_iter()
        .filter_map(|a| match a {
            Alert::VolumeSpike(spike) => Some(spike.user),
            _ => None,
        })
        .collect();
    assert!(!spikes.is_empty() && spikes.iter().all(|&user| user == victim), "{spikes:?}");
}
