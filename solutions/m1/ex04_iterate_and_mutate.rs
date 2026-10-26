//! Exercice 4 — Modifier une collection pendant qu'on la parcourt (corrigé)
//! Erreur attendue : E0502
//!
//! On veut retirer les entrées des scanners connus. En C++, effacer pendant un parcours
//! invalide l'itérateur ; ici, le compilateur refuse. Expliquez pourquoi, puis corrigez :
//! aucun emprunt de `ips` ne doit être vivant au moment où on le modifie.
//!
//! # Corrigé
//!
//! Le parcours emprunte `ips` ; `remove` voudrait le modifier pendant ce temps. (Et même
//! sans le compilateur, l'indice `i` serait faux après le premier retrait.)
//! Le plus simple à ce stade du cours : ne pas modifier le vecteur parcouru, mais en
//! construire un autre (1), ou parcourir par indice pour que chaque emprunt (`ips[i]`)
//! se termine avant la modification (2, 3). Les itérateurs (4) et `retain` (5, vu au
//! module 2 ; m2/ex01 y revient) viennent plus tard dans le cours.

fn main() {
    let scanners = ["100.64.1.2", "192.0.2.67"];
    let original = vec!["198.51.100.227", "100.64.1.2", "203.0.113.72", "192.0.2.67"];
    let expected = ["198.51.100.227", "203.0.113.72"];

    // 1. garder ce qui reste dans un nouveau vecteur : la boucle consomme `ips` (move),
    //    qu'on réaffecte ensuite ; rien n'est emprunté pendant les `push`
    let mut ips = original.clone();
    let mut kept = Vec::new();
    for ip in ips {
        if !scanners.contains(&ip) {
            kept.push(ip);
        }
    }
    ips = kept;
    assert_eq!(ips, expected);

    // 2. une boucle sur l'indice : on n'avance que si l'on n'a rien retiré, puisque
    //    `remove` décale les suivants d'une case
    let mut ips = original.clone();
    let mut i = 0;
    while i < ips.len() {
        if scanners.contains(&ips[i]) {
            ips.remove(i);
        } else {
            i += 1;
        }
    }
    assert_eq!(ips, expected);

    // 3. noter les positions pendant le parcours, retirer ensuite, de la fin vers le
    //    début pour que les positions restantes restent justes
    let mut ips = original.clone();
    let mut doomed = Vec::new();
    #[allow(clippy::needless_range_loop)] // par indice exprès : les itérateurs viennent en 4
    for i in 0..ips.len() {
        if scanners.contains(&ips[i]) {
            doomed.push(i);
        }
    }
    while let Some(i) = doomed.pop() {
        ips.remove(i);
    }
    assert_eq!(ips, expected);

    // 4. (itérateurs) la solution 1 en une expression : un vecteur filtré, construit en
    //    consommant l'ancien (`into_iter`)
    let ips: Vec<&str> = original
        .clone()
        .into_iter()
        .filter(|ip| !scanners.contains(ip))
        .collect();
    assert_eq!(ips, expected);

    // 5. (module 2) filtrer en place, en une passe
    let mut ips = original;
    ips.retain(|ip| !scanners.contains(ip));
    assert_eq!(ips, expected);
    println!("ex04 : OK");
}
