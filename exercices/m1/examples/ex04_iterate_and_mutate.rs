//! Exercice 4 — Modifier une collection pendant qu'on la parcourt
//! Erreur attendue : E0502
//!
//! On veut retirer les entrées des scanners connus. En C++, effacer pendant un parcours
//! invalide l'itérateur ; ici, le compilateur refuse. Corrigez avec la méthode prévue
//! pour cela, en une ligne.

fn main() {
    let scanners = ["100.64.1.2", "192.0.2.67"];
    let mut ips = vec!["198.51.100.227", "100.64.1.2", "203.0.113.72", "192.0.2.67"];
    for (i, ip) in ips.iter().enumerate() {
        if scanners.contains(ip) {
            ips.remove(i);
        }
    }
    assert_eq!(ips, ["198.51.100.227", "203.0.113.72"]);
    println!("ex04 : OK");
}
