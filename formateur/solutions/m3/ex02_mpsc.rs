//! Exemple TP 2 (module 3) — Communiquer par canaux (`std::sync::mpsc`) (corrigé)
//!
//! Trois capteurs (threads) surveillent chacun un serveur et **envoient** une alerte par
//! échec repéré ; le thread principal les **reçoit** et les range.

use std::sync::mpsc;
use std::thread;

const SERVERS: [(&str, &str); 3] = [
    (
        "bastion-01",
        "Failed password for root\nFailed password for l.roux\n",
    ),
    ("srv-web-01", "Accepted password for l.roux\n"),
    ("srv-db-02", "Failed password for postgres\n"),
];

/// Toutes les alertes `"serveur : ligne"`, triées.
///
/// Un `Sender` cloné par capteur. Attention : la boucle de réception ne se termine que
/// quand **tous** les émetteurs ont disparu.
fn collect_alerts() -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    for (server, log) in SERVERS {
        let tx = tx.clone();
        thread::spawn(move || {
            for line in log.lines().filter(|l| l.starts_with("Failed")) {
                tx.send(format!("{server} : {line}"))
                    .expect("récepteur disparu");
            }
        });
    }
    // sans ce `drop`, le `for` ci-dessous attendrait indéfiniment l'émetteur d'origine
    drop(tx);
    let mut alerts: Vec<String> = rx.into_iter().collect();
    alerts.sort();
    alerts
}

fn main() {
    let alerts = collect_alerts();
    assert_eq!(
        alerts,
        [
            "bastion-01 : Failed password for l.roux",
            "bastion-01 : Failed password for root",
            "srv-db-02 : Failed password for postgres"
        ]
    );
    println!("m3/ex02 : OK");
}
