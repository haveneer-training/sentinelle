//! Exemple TP 1 (module 3) — corrigé.

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

fn count_with_join() -> usize {
    // les fragments sont `&'static str` : `spawn` peut les emporter
    let handles: Vec<_> = FRAGMENTS
        .into_iter()
        .map(|fragment| thread::spawn(move || failures(fragment)))
        .collect();
    handles
        .into_iter()
        .map(|h| h.join().expect("thread paniqué"))
        .sum()
}

fn count_with_mutex() -> usize {
    let total = Arc::new(Mutex::new(0));
    let handles: Vec<_> = FRAGMENTS
        .into_iter()
        .map(|fragment| {
            let total = Arc::clone(&total);
            thread::spawn(move || *total.lock().unwrap() += failures(fragment))
        })
        .collect();
    for handle in handles {
        handle.join().expect("thread paniqué");
    }
    *total.lock().unwrap()
}

fn main() {
    assert_eq!(count_with_join(), 6);
    assert_eq!(count_with_mutex(), 6);
    println!("m3/ex01 : OK");
}
