//! Exemple TP 1 (module 2) — Manipuler un vecteur (corrigé)
//!
//! Les instants des échecs d'une adresse, gardés **triés** dans un `Vec<i64>`.
//! Remplacez chaque `todo!()` ; `main` vérifie le résultat.

/// Ajoute un échec, plus récent que tous les autres.
fn record(failures: &mut Vec<i64>, ts: i64) {
    failures.push(ts);
}

/// Une ligne arrivée en retard : l'insérer à sa place, sans retrier tout le vecteur.
fn insert_late(failures: &mut Vec<i64>, ts: i64) {
    // `Err(i)` : absent, à insérer en `i` ; `Ok(i)` : déjà présent en `i`
    let position = failures.binary_search(&ts).unwrap_or_else(|i| i);
    failures.insert(position, ts);
}

/// Oublie les échecs antérieurs à `limit`.
fn forget_before(failures: &mut Vec<i64>, limit: i64) {
    // trié : tout ce qui est à retirer est au début
    let keep_from = failures.partition_point(|&ts| ts < limit);
    failures.drain(..keep_from);
}

/// Retire les adresses des scanners connus : la boucle refusée de m1/ex04, cette fois
/// en une ligne.
fn forget_scanners(ips: &mut Vec<&str>, scanners: &[&str]) {
    ips.retain(|ip| !scanners.contains(ip));
}

/// Supprime les doublons (deux lignes identiques au même instant).
fn deduplicate(failures: &mut Vec<i64>) {
    failures.dedup();
}

/// Les `n` échecs les plus récents (tous s'il y en a moins).
fn latest(failures: &[i64], n: usize) -> &[i64] {
    &failures[failures.len().saturating_sub(n)..]
}

/// Le plus petit écart entre deux échecs consécutifs, s'il y en a au moins deux.
fn shortest_gap(failures: &[i64]) -> Option<i64> {
    failures.windows(2).map(|w| w[1] - w[0]).min()
}

fn main() {
    let mut failures = Vec::new();
    for ts in [100, 102, 110, 110, 140] {
        record(&mut failures, ts);
    }
    insert_late(&mut failures, 105);
    assert_eq!(failures, [100, 102, 105, 110, 110, 140]);
    deduplicate(&mut failures);
    assert_eq!(failures, [100, 102, 105, 110, 140]);
    assert_eq!(shortest_gap(&failures), Some(2));
    assert_eq!(latest(&failures, 2), [110, 140]);
    assert_eq!(latest(&failures, 10).len(), 5);
    forget_before(&mut failures, 105);
    assert_eq!(failures, [105, 110, 140]);
    assert_eq!(shortest_gap(&failures[..1]), None);
    let mut ips = vec!["198.51.100.227", "100.64.1.2", "203.0.113.72", "192.0.2.67"];
    forget_scanners(&mut ips, &["100.64.1.2", "192.0.2.67"]);
    assert_eq!(ips, ["198.51.100.227", "203.0.113.72"]);
    println!("m2/ex01 : OK");
}
