//! Exercice 8 — Des clés qui empruntent une chaîne éphémère
//! Erreur attendue : E0597
//!
//! On compte les connexions par utilisateur. Chaque ligne est recopiée dans une `String`
//! fraîche (comme le ferait une lecture ligne à ligne), et la table garde des `&str`.
//! Corrigez de deux façons : en changeant le type des clés, puis en supprimant la copie
//! des lignes.

use std::collections::HashMap;

const LOG: &str = "l.roux bastion-01\na.martin srv-web-01\nl.roux srv-db-02\n";

fn main() {
    let mut logins: HashMap<&str, u32> = HashMap::new();
    for raw in LOG.lines() {
        let line = raw.to_string();
        let user = line.split(' ').next().unwrap();
        *logins.entry(user).or_insert(0) += 1;
    }
    assert_eq!(logins["l.roux"], 2);
    assert_eq!(logins["a.martin"], 1);
    println!("ex08 : OK");
}
