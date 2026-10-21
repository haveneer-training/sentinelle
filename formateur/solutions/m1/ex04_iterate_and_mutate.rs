//! Exercice 4 — corrigé.
//!
//! Le parcours emprunte `ips` ; `remove` voudrait le modifier pendant ce temps. (Et même
//! sans le compilateur, l'indice `i` serait faux après le premier retrait.)

fn main() {
    let scanners = ["100.64.1.2", "192.0.2.67"];
    let mut ips = vec!["198.51.100.227", "100.64.1.2", "203.0.113.72", "192.0.2.67"];
    ips.retain(|ip| !scanners.contains(ip));
    assert_eq!(ips, ["198.51.100.227", "203.0.113.72"]);
    println!("ex04 : OK");
}
