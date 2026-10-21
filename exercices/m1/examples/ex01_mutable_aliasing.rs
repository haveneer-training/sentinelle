//! Exercice 1 — Emprunt partagé puis modification
//! Erreur attendue : E0502
//!
//! On garde une référence vers l'adresse la plus active, puis on ajoute une nouvelle
//! adresse au vecteur. Pourquoi le compilateur refuse-t-il ? Que pourrait-il arriver à
//! `top` si `push` réallouait le tampon du vecteur ? Corrigez sans changer le résultat
//! affiché.

fn main() {
    let mut counts: Vec<(&str, u32)> = vec![("198.51.100.227", 445), ("203.0.113.72", 121)];
    let top = &counts[0];
    counts.push(("192.0.2.67", 25));
    assert_eq!(top.0, "198.51.100.227");
    assert_eq!(counts.len(), 3);
    println!("ex01 : OK");
}
