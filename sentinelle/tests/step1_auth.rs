//! Étape 1 : analyser une ligne de `auth.log` sans la copier.
#![cfg(feature = "step1")]

use sentinelle::ParseError;
use sentinelle::auth::{AuthEntry, AuthEvent, parse_auth_event, parse_auth_line};
use sentinelle::time::parse_timestamp;
use std::net::IpAddr;

fn ip(text: &str) -> IpAddr {
    text.parse().unwrap()
}

#[test]
fn parses_header_with_pid() {
    let line = "2026-11-09T09:41:12Z srv-web-02 sshd[48213]: Accepted publickey for v.michel from 10.1.0.42 port 51022 ssh2";
    let entry = parse_auth_line(line).unwrap();
    assert_eq!(
        entry,
        AuthEntry {
            timestamp: parse_timestamp("2026-11-09T09:41:12Z").unwrap(),
            host: "srv-web-02",
            process: "sshd",
            pid: Some(48213),
            event: AuthEvent::Accepted { user: "v.michel", ip: ip("10.1.0.42"), method: "publickey" },
        }
    );
}

#[test]
fn borrows_from_the_line() {
    let line = String::from("2026-11-09T09:41:12Z srv-web-02 sshd[1]: Failed password for root from 192.0.2.1 port 1 ssh2");
    let entry = parse_auth_line(&line).unwrap();
    let range = line.as_bytes().as_ptr_range();
    assert!(range.contains(&entry.host.as_ptr()), "host doit être une tranche de la ligne, pas une copie");
}

#[test]
fn parses_sudo_without_pid() {
    let line = "2026-11-09T09:41:50Z srv-web-02 sudo:   v.michel : TTY=pts/6 ; PWD=/home/v.michel ; USER=root ; COMMAND=/usr/bin/df -h";
    let entry = parse_auth_line(line).unwrap();
    assert_eq!(entry.process, "sudo");
    assert_eq!(entry.pid, None);
    assert_eq!(entry.event, AuthEvent::Sudo { user: "v.michel", command: "/usr/bin/df -h" });
}

#[test]
fn recognizes_every_message() {
    let cases = [
        ("Failed password for alice from 192.0.2.7 port 4242 ssh2", AuthEvent::Failed { user: "alice", ip: ip("192.0.2.7"), invalid_user: false }),
        (
            "Failed password for invalid user oracle from 100.64.3.9 port 4242 ssh2",
            AuthEvent::Failed { user: "oracle", ip: ip("100.64.3.9"), invalid_user: true },
        ),
        ("Invalid user oracle from 100.64.3.9 port 4242", AuthEvent::InvalidUser { user: "oracle", ip: ip("100.64.3.9") }),
        ("Accepted password for bob from 10.0.0.10 port 1 ssh2", AuthEvent::Accepted { user: "bob", ip: ip("10.0.0.10"), method: "password" }),
        ("pam_unix(sshd:session): session opened for user bob(uid=1003) by (uid=0)", AuthEvent::SessionOpened { user: "bob" }),
        ("pam_unix(sshd:session): session closed for user bob", AuthEvent::SessionClosed { user: "bob" }),
        ("Connection closed by 192.0.2.7 port 4242 [preauth]", AuthEvent::Other("Connection closed by 192.0.2.7 port 4242 [preauth]")),
        (
            "pam_unix(cron:session): session opened for user root(uid=0) by (uid=0)",
            AuthEvent::Other("pam_unix(cron:session): session opened for user root(uid=0) by (uid=0)"),
        ),
    ];
    for (message, expected) in cases {
        assert_eq!(parse_auth_event(message), Ok(expected), "message : {message}");
    }
}

#[test]
fn reports_precise_errors() {
    assert_eq!(parse_auth_line(""), Err(ParseError::Empty));
    assert_eq!(parse_auth_line("2026-11-09T09:41:12Z"), Err(ParseError::MissingField("host")));
    assert_eq!(parse_auth_line("2026-11-09T09:41:12Z srv-web-02"), Err(ParseError::MissingField("process")));
    assert_eq!(parse_auth_line("2026-11-09T09:41:12Z srv-web-02 sshd[12]"), Err(ParseError::MissingField("message")));
    assert_eq!(parse_auth_line("2026-11-09 srv-web-02 sshd[12]: x"), Err(ParseError::BadTimestamp));
    assert_eq!(parse_auth_line("2026-11-09T09:41:12Z srv-web-02 sshd[1x]: x"), Err(ParseError::BadNumber("pid")));
    assert_eq!(parse_auth_event("Failed password for alice from port 4242 ssh2"), Err(ParseError::BadIp));
    assert_eq!(parse_auth_event("  alice : TTY=pts/0 ; PWD=/home/alice"), Err(ParseError::MissingField("COMMAND")));
}

#[test]
fn a_truncated_message_is_not_mistaken_for_an_event() {
    // le compte a disparu : « from » ne doit pas être pris pour un nom d'utilisateur
    let event = parse_auth_event("Failed password for from 192.0.2.7 port 4242 ssh2");
    assert!(!matches!(event, Ok(AuthEvent::Failed { .. })), "obtenu : {event:?}");
}
