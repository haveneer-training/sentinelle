//! Analyse syntaxique : la v0.1 (une `String` par champ) contre le fil rouge (emprunts).
//!
//! `cargo bench -p sentinelle-perf --bench parse`

use sentinelle::auth::{parse_auth_event, parse_auth_line};
use sentinelle::journal::{parse_auth, parse_web};
use sentinelle_gen::{Dataset, Options};
use sentinelle_perf::{Bench, legacy};

fn main() {
    let data = Dataset::generate(&Options::default());
    let text = std::str::from_utf8(&data.auth).unwrap_or_default();
    let failed = "2026-11-11T22:30:00Z bastion-01 sshd[4242]: Failed password for l.roux from 198.51.100.227 port 50022 ssh2";
    let mut bench = Bench::from_args();

    bench.run("parse/ligne/v0.1", Some(failed.len() as u64), || {
        legacy::parse_auth_line(failed.to_string())
    });
    bench.run("parse/ligne/sentinelle", Some(failed.len() as u64), || {
        parse_auth_line(failed)
    });
    bench.run("parse/message/sentinelle", None, || {
        parse_auth_event("Failed password for l.roux from 198.51.100.227 port 50022 ssh2")
    });
    bench.run("parse/auth.log/v0.1", Some(data.auth.len() as u64), || {
        legacy::parse_all(text).len()
    });
    bench.run(
        "parse/auth.log/sentinelle",
        Some(data.auth.len() as u64),
        || parse_auth(&data.auth).entries.len(),
    );
    bench.run(
        "parse/access.log/sentinelle",
        Some(data.access.len() as u64),
        || parse_web(&data.access).entries.len(),
    );
}
