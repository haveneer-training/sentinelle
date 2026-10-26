//! Comptages et détections.
//!
//! Les fonctions empruntent les entrées (`&[AuthEntry]`) : la v0.1 en recopiait le
//! journal entier à chaque appel.

use crate::entry::{AuthEntry, AuthKind, WebEntry};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

/// Seuil d'échecs au-delà duquel une adresse est suspectée de force brute.
pub const BRUTEFORCE_FAILURES: usize = 100;
/// Seuil de comptes distincts au-delà duquel une adresse est suspectée de spraying.
pub const SPRAYING_ACCOUNTS: usize = 20;

/// Échecs par adresse, par nombre décroissant ; à égalité, dans l'ordre d'apparition.
///
/// **Défaut conservé (D3)** : les lignes `Invalid user` comptent aussi, si bien qu'une
/// tentative sur un compte inexistant compte deux fois. La correction tient en un mot
/// (ne garder que [`AuthKind::Failed`]), mais elle change le rapport : elle se fait après
/// la refactorisation, en régénérant la référence du test. Elle est ici derrière la
/// feature `fix-d3`.
pub fn count_failures_by_ip(entries: &[AuthEntry]) -> Vec<(&str, usize)> {
    // un `Vec` pour l'ordre d'apparition, une table pour trouver la position en O(1)
    let mut counts: Vec<(&str, usize)> = Vec::new();
    let mut position: HashMap<&str, usize> = HashMap::new();
    for entry in entries {
        if is_failure(entry.kind) {
            let index = *position.entry(&entry.ip).or_insert_with(|| {
                counts.push((&entry.ip, 0));
                counts.len() - 1
            });
            counts[index].1 += 1;
        }
    }
    counts.sort_by_key(|&(_, count)| Reverse(count));
    counts
}

#[cfg(not(feature = "fix-d3"))]
fn is_failure(kind: AuthKind) -> bool {
    matches!(kind, AuthKind::Failed | AuthKind::InvalidUser)
}

#[cfg(feature = "fix-d3")]
fn is_failure(kind: AuthKind) -> bool {
    kind == AuthKind::Failed
}

/// Adresses de plus de [`BRUTEFORCE_FAILURES`] échecs, à partir des comptes déjà faits.
pub fn detect_bruteforce<'a>(failures: &[(&'a str, usize)]) -> Vec<&'a str> {
    failures
        .iter()
        .filter(|&&(_, count)| count > BRUTEFORCE_FAILURES)
        .map(|&(ip, _)| ip)
        .collect()
}

/// Adresses ayant échoué sur plus de [`SPRAYING_ACCOUNTS`] comptes distincts.
pub fn detect_spraying(entries: &[AuthEntry]) -> Vec<(&str, usize)> {
    let mut accounts: HashMap<&str, HashSet<&str>> = HashMap::new();
    for entry in entries.iter().filter(|e| e.kind == AuthKind::Failed) {
        accounts.entry(&entry.ip).or_default().insert(&entry.user);
    }
    let mut found: Vec<(&str, usize)> = accounts
        .into_iter()
        .map(|(ip, users)| (ip, users.len()))
        .filter(|&(_, n)| n > SPRAYING_ACCOUNTS)
        .collect();
    found.sort_by_key(|&(_, accounts)| Reverse(accounts));
    found
}

/// Volume servi par utilisateur, par volume décroissant. Une taille illisible (`-`) compte 0.
pub fn bytes_by_user(entries: &[WebEntry]) -> Vec<(&str, u64)> {
    let mut totals: HashMap<&str, u64> = HashMap::new();
    for entry in entries {
        *totals.entry(&entry.user).or_insert(0) += entry.bytes.parse::<u64>().unwrap_or(0);
    }
    let mut sorted: Vec<(&str, u64)> = totals.into_iter().collect();
    sorted.sort_by_key(|&(_, bytes)| Reverse(bytes));
    sorted
}
