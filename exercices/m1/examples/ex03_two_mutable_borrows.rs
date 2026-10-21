//! Exercice 3 — Deux emprunts exclusifs
//! Erreur attendue : E0499
//!
//! Deux adresses appartiennent au même attaquant : on reporte les échecs de la seconde
//! sur la première. Pourquoi deux `get_mut` simultanés sont-ils interdits, même sur deux
//! clés différentes ? Corrigez de deux façons : en limitant la durée des emprunts, puis
//! avec `HashMap::get_disjoint_mut` (Rust ≥ 1.86).

use std::collections::HashMap;

fn main() {
    let mut failures: HashMap<&str, u32> = HashMap::from([("198.51.100.227", 445), ("198.51.100.228", 12)]);
    let main = failures.get_mut("198.51.100.227").unwrap();
    let alias = failures.get_mut("198.51.100.228").unwrap();
    *main += *alias;
    *alias = 0;
    assert_eq!(failures["198.51.100.227"], 457);
    assert_eq!(failures["198.51.100.228"], 0);
    println!("ex03 : OK");
}
