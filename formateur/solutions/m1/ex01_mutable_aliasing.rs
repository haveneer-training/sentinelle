//! Exercice 1 — Emprunt partagé puis modification (corrigé)
//! Erreur attendue : E0502
//!
//! On garde une référence vers l'adresse la plus active, puis on ajoute une nouvelle
//! adresse au vecteur. Pourquoi le compilateur refuse-t-il ? Que pourrait-il arriver à
//! `top` si `push` réallouait le tampon du vecteur ? Corrigez sans changer le résultat
//! affiché.
//!
//! # Corrigé
//!
//! `push` peut réallouer le tampon : `top` pointerait alors dans une mémoire libérée.
//! Solutions : copier ce qu'on veut garder (les éléments sont `Copy`), ou terminer
//! l'emprunt avant la modification.

fn main() {
    let mut counts: Vec<(&str, u32)> = vec![("198.51.100.227", 445), ("203.0.113.72", 121)];
    let top = counts[0]; // copie : plus aucun emprunt de `counts`
    counts.push(("192.0.2.67", 25));
    assert_eq!(top.0, "198.51.100.227");
    assert_eq!(counts.len(), 3);
    // variante : prendre la référence après le `push`, quand on en a besoin
    let top = &counts[0];
    assert_eq!(top.0, "198.51.100.227");
    println!("ex01 : OK");
}
