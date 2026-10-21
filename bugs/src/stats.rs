//! `stats` : échecs par adresse et comptes visés, calculés par plusieurs threads qui
//! partagent les mêmes tables.

use crate::load::{failed_or_accepted, read_log};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;
use std::thread;

#[derive(Default)]
struct Shared {
    /// adresse -> échecs consécutifs
    failures: Mutex<HashMap<String, usize>>,
    /// adresse -> comptes visés
    accounts: Mutex<HashMap<String, HashSet<String>>>,
}

impl Shared {
    fn failed(&self, ip: &str, user: &str) {
        let mut failures = self.failures.lock().unwrap();
        *failures.entry(ip.to_string()).or_insert(0) += 1;
        let mut accounts = self.accounts.lock().unwrap();
        accounts
            .entry(ip.to_string())
            .or_default()
            .insert(user.to_string());
    }

    /// Un succès blanchit l'adresse si elle n'a visé que ce compte.
    fn accepted(&self, ip: &str, user: &str) {
        let accounts = self.accounts.lock().unwrap();
        let only_this_user = accounts
            .get(ip)
            .is_none_or(|users| users.len() == 1 && users.contains(user));
        if only_this_user {
            self.failures.lock().unwrap().remove(ip);
        }
    }
}

pub fn run(dir: &Path, threads: usize) {
    let text = read_log(&dir.join("auth.log"));
    let lines: Vec<&str> = text.lines().collect();
    let shared = Shared::default();
    let chunk = lines.len().div_ceil(threads.max(1)).max(1);
    thread::scope(|scope| {
        for part in lines.chunks(chunk) {
            let shared = &shared;
            scope.spawn(move || {
                for line in part {
                    match failed_or_accepted(line) {
                        Some((true, user, ip)) => shared.failed(ip, user),
                        Some((false, user, ip)) => shared.accepted(ip, user),
                        None => {}
                    }
                }
            });
        }
    });
    let failures = shared.failures.into_inner().unwrap();
    let accounts = shared.accounts.into_inner().unwrap();
    let mut top: Vec<(&String, &usize)> = failures.iter().collect();
    top.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (ip, count) in top.iter().take(5) {
        println!(
            "{ip:>18} {count:>6} échecs, {} comptes",
            accounts.get(*ip).map_or(0, HashSet::len)
        );
    }
}
