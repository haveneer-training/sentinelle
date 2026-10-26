//! Exercice 3 — Deux emprunts exclusifs (corrigé)
//! Erreur attendue : E0499
//!
//! Deux adresses appartiennent au même attaquant : on reporte les échecs de la seconde
//! sur la première. Pourquoi deux `get_mut` simultanés sont-ils interdits, même sur deux
//! clés différentes ? Corrigez de deux façons : en limitant la durée des emprunts, puis
//! avec `HashMap::get_disjoint_mut` (Rust ≥ 1.86).
//!
//! # Corrigé
//!
//! `get_mut` emprunte **toute** la table de façon exclusive : le compilateur ne sait pas
//! que les deux clés diffèrent (et une insertion pourrait tout déplacer).

use std::collections::HashMap;

fn main() {
    // 1. des emprunts courts : lire, puis écrire
    let mut failures: HashMap<&str, u32> =
        HashMap::from([("198.51.100.227", 445), ("198.51.100.228", 12)]);
    let moved = std::mem::take(failures.get_mut("198.51.100.228").unwrap());
    *failures.get_mut("198.51.100.227").unwrap() += moved;
    assert_eq!(
        (failures["198.51.100.227"], failures["198.51.100.228"]),
        (457, 0)
    );

    // 2. `get_disjoint_mut` vérifie à l'exécution que les clés sont distinctes
    let mut failures: HashMap<&str, u32> =
        HashMap::from([("198.51.100.227", 445), ("198.51.100.228", 12)]);
    let [Some(main), Some(alias)] = failures.get_disjoint_mut(["198.51.100.227", "198.51.100.228"])
    else {
        panic!("adresses absentes");
    };
    *main += *alias;
    *alias = 0;
    assert_eq!(
        (failures["198.51.100.227"], failures["198.51.100.228"]),
        (457, 0)
    );
    println!("ex03 : OK");
}
