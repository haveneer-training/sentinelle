//! Exemple TP 1 (module 3) — Créer des threads et les synchroniser
//!
//! Quatre fragments de journal, un thread par fragment.
//! 1. `count_with_join` : chaque thread **renvoie** son compte, récupéré par `join`.
//! 2. `count_with_mutex` : les threads ajoutent leur compte à un total partagé.

use std::sync::{Arc, Mutex};
use std::thread;

const FRAGMENTS: [&str; 4] = [
    "Failed password for root\nAccepted password for l.roux\n",
    "Failed password for admin\nFailed password for oracle\n",
    "session opened for user l.roux\n",
    "Failed password for l.roux\nFailed password for l.roux\nFailed password for l.roux\n",
];

fn failures(fragment: &str) -> usize {
    fragment.lines().filter(|l| l.starts_with("Failed")).count()
}

/// Un `thread::spawn` par fragment, puis la somme des résultats des `join()`.
fn count_with_join() -> usize {
    todo!("spawn(move || failures(fragment)), puis join")
}

/// Un total `Arc<Mutex<usize>>` partagé ; chaque thread y ajoute son compte.
fn count_with_mutex() -> usize {
    todo!("Arc::clone pour chaque thread, lock() pour ajouter, join avant de lire")
}

fn main() {
    assert_eq!(count_with_join(), 6);
    assert_eq!(count_with_mutex(), 6);
    println!("m3/ex01 : OK");
}
