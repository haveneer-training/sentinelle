//! Étape 6 : corréler et apprendre des habitudes.
#![cfg(feature = "step6")]

mod common;

use sentinelle::auth::parse_auth_line;
use sentinelle::detect::{Config, detect_compromise, detect_new_access, detect_unusual_sudo};
use sentinelle::journal::parse_auth;
use std::net::IpAddr;

const LINES: &[&str] = &[
    "2026-11-09T09:00:00Z srv-web-01 sshd[1]: Accepted password for alice from 10.1.0.1 port 1 ssh2",
    "2026-11-09T09:05:00Z srv-web-01 sudo:   alice : TTY=pts/0 ; PWD=/home/alice ; USER=root ; COMMAND=/usr/bin/df",
    "2026-11-10T09:00:00Z srv-web-01 sshd[2]: Accepted password for alice from 10.1.0.1 port 1 ssh2",
    "2026-11-10T09:01:00Z srv-db-01 sshd[3]: Accepted password for alice from 10.1.0.1 port 1 ssh2",
    "2026-11-10T09:02:00Z srv-db-01 sshd[4]: Accepted password for alice from 10.1.0.1 port 1 ssh2",
    "2026-11-10T09:03:00Z srv-db-01 sudo:   alice : TTY=pts/0 ; PWD=/home/alice ; USER=root ; COMMAND=/usr/bin/ls",
    "2026-11-10T09:04:00Z srv-db-01 sudo:   bob : TTY=pts/1 ; PWD=/home/bob ; USER=root ; COMMAND=/usr/bin/id",
];

#[test]
fn learns_then_reports_new_pairs_once() {
    let entries: Vec<_> = LINES.iter().map(|l| parse_auth_line(l).unwrap()).collect();
    let learn_until = entries[2].timestamp; // la journée du 9 sert d'apprentissage
    let found = detect_new_access(&entries, learn_until);
    let pairs: Vec<_> = found.iter().map(|a| (a.user, a.host)).collect();
    assert_eq!(pairs, vec![("alice", "srv-db-01")]);
}

#[test]
fn sudo_by_an_unusual_account() {
    let entries: Vec<_> = LINES.iter().map(|l| parse_auth_line(l).unwrap()).collect();
    let found = detect_unusual_sudo(&entries, entries[2].timestamp);
    let commands: Vec<_> = found.iter().map(|s| (s.user, s.command)).collect();
    assert_eq!(commands, vec![("bob", "/usr/bin/id")]);
}

#[test]
fn reconstructs_the_intrusion() {
    let data = common::dataset();
    let scenario = &data.generated.scenario;
    let victim = common::victim(data);
    let journal = parse_auth(&data.auth);
    let config = Config::starting_at(journal.entries[0].timestamp);

    let compromises = detect_compromise(&journal.entries, config.compromise_failures);
    assert_eq!(compromises.len(), 1, "{compromises:?}");
    let compromise = &compromises[0];
    assert_eq!((compromise.user, compromise.host), (victim, "bastion-01"));
    assert_eq!(compromise.ip, IpAddr::V4(scenario.brute_ip));
    assert_eq!(compromise.at, scenario.compromise);

    let new_access = detect_new_access(&journal.entries, config.learn_until);
    for host in ["bastion-01", "srv-db-02"] {
        assert!(
            new_access
                .iter()
                .any(|a| a.user == victim && a.host == host),
            "rebond vers {host} manqué"
        );
    }

    let sudo = detect_unusual_sudo(&journal.entries, config.learn_until);
    assert!(
        sudo.iter().all(|s| s.user == victim),
        "faux positifs : {sudo:?}"
    );
    assert!(
        sudo.iter()
            .any(|s| s.host == "srv-db-02" && s.command.contains("pg_dump"))
    );
}
