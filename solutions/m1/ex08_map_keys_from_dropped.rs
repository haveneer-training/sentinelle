//! Exercice 8 — Des clés qui empruntent une chaîne éphémère (corrigé)
//! Erreur attendue : E0597
//!
//! On compte les connexions par utilisateur. Chaque ligne est recopiée dans une `String`
//! fraîche (comme le ferait une lecture ligne à ligne), et la table garde des `&str`.
//! Corrigez de deux façons : en changeant le type des clés, puis en supprimant la copie
//! des lignes.
//!
//! # Corrigé
//!
//! Chaque `line` meurt à la fin de son tour de boucle, ses tranches avec elle.

use std::collections::HashMap;

const LOG: &str = "l.roux bastion-01\na.martin srv-web-01\nl.roux srv-db-02\n";

fn main() {
    // 1. des clés possédées : une allocation par utilisateur nouveau seulement
    let mut logins: HashMap<String, u32> = HashMap::new();
    for raw in LOG.lines() {
        let line = raw.to_string();
        let user = line.split(' ').next().unwrap();
        match logins.get_mut(user) {
            Some(count) => *count += 1,
            None => {
                logins.insert(user.to_string(), 1);
            }
        }
    }
    assert_eq!((logins["l.roux"], logins["a.martin"]), (2, 1));

    // 2. pas de copie des lignes : les clés empruntent `LOG`, qui vit toujours
    let mut logins: HashMap<&str, u32> = HashMap::new();
    for line in LOG.lines() {
        *logins.entry(line.split(' ').next().unwrap()).or_insert(0) += 1;
    }
    assert_eq!((logins["l.roux"], logins["a.martin"]), (2, 1));
    println!("ex08 : OK");
}
