//! Exercice 4 — corrigé.
//!
//! Le parcours emprunte `ips` ; `remove` voudrait le modifier pendant ce temps. (Et même
//! sans le compilateur, l'indice `i` serait faux après le premier retrait.)
//! Deux temps : décider pendant le parcours, modifier après. `retain`, vu au module 2
//! (m2/ex01 y revient), fait les deux en une ligne.

fn main() {
    let scanners = ["100.64.1.2", "192.0.2.67"];
    let original = vec!["198.51.100.227", "100.64.1.2", "203.0.113.72", "192.0.2.67"];
    let expected = ["198.51.100.227", "203.0.113.72"];

    // 1. noter les positions pendant le parcours, retirer ensuite, de la fin vers le
    //    début pour que les positions restantes restent justes
    let mut ips = original.clone();
    let doomed: Vec<usize> = ips
        .iter()
        .enumerate()
        .filter(|(_, ip)| scanners.contains(ip))
        .map(|(i, _)| i)
        .collect();
    for &i in doomed.iter().rev() {
        ips.remove(i);
    }
    assert_eq!(ips, expected);

    // 2. reconstruire un vecteur filtré : `into_iter` consomme l'ancien
    let ips: Vec<&str> = original
        .clone()
        .into_iter()
        .filter(|ip| !scanners.contains(ip))
        .collect();
    assert_eq!(ips, expected);

    // 3. (module 2) filtrer en place, en une passe
    let mut ips = original;
    ips.retain(|ip| !scanners.contains(ip));
    assert_eq!(ips, expected);
    println!("ex04 : OK");
}
