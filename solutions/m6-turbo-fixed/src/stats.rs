//! `stats` : échecs par adresse et comptes visés, calculés par plusieurs threads.

use crate::load::{failed_or_accepted, lines, read_log};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::thread;

/// Les comptes d'une partie du journal.
///
/// Ticket 4 : deux `Mutex` pris dans un ordre différent par `failed` et `accepted`
/// s'interbloquaient. Chaque thread a désormais ses tables, fusionnées à la fin : plus
/// aucun verrou (comme à l'étape 8 du fil rouge).
#[derive(Default)]
struct Tables {
    /// adresse -> échecs
    failures: HashMap<String, usize>,
    /// adresse -> comptes visés
    accounts: HashMap<String, HashSet<String>>,
}

impl Tables {
    fn failed(&mut self, ip: &str, user: &str) {
        *self.failures.entry(ip.to_string()).or_insert(0) += 1;
        self.accounts
            .entry(ip.to_string())
            .or_default()
            .insert(user.to_string());
    }

    fn merge(&mut self, other: Tables) {
        for (ip, n) in other.failures {
            *self.failures.entry(ip).or_insert(0) += n;
        }
        for (ip, users) in other.accounts {
            self.accounts.entry(ip).or_default().extend(users);
        }
    }
}

pub fn run(dir: &Path, threads: usize) {
    let text = read_log(&dir.join("auth.log"));
    let lines: Vec<&str> = lines(&text).collect();
    let chunk = lines.len().div_ceil(threads.max(1)).max(1);
    let mut total = Tables::default();
    thread::scope(|scope| {
        let handles: Vec<_> = lines
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    let mut tables = Tables::default();
                    for line in part {
                        // Ticket bonus : un succès ne « blanchit » plus l'adresse. Effacer
                        // ses échecs masquait précisément une force brute réussie.
                        if let Some((true, user, ip)) = failed_or_accepted(line) {
                            tables.failed(ip, user);
                        }
                    }
                    tables
                })
            })
            .collect();
        for handle in handles {
            total.merge(handle.join().unwrap());
        }
    });
    let Tables { failures, accounts } = total;
    let mut top: Vec<(&String, &usize)> = failures.iter().collect();
    top.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (ip, count) in top.iter().take(5) {
        println!(
            "{ip:>18} {count:>6} échecs, {} comptes",
            accounts.get(*ip).map_or(0, HashSet::len)
        );
    }
}
