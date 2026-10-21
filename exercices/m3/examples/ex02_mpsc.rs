//! Exemple TP 2 (module 3) — Communiquer par canaux (`std::sync::mpsc`)
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
    todo!("mpsc::channel, un tx.clone() par thread, drop(tx), for alert in rx")
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
