//! `report` : volumes servis par utilisateur et par heure, au-delà d'un seuil.

use crate::load::read_log;
use std::collections::BTreeMap;
use std::path::Path;

/// Au-delà, le volume horaire est signalé.
const THRESHOLD: u64 = 2_000_000_000;

pub fn run(dir: &Path) {
    let text = read_log(&dir.join("access.log"));
    // (utilisateur, jour et heure) -> octets
    let mut volumes: BTreeMap<(&str, &str), u32> = BTreeMap::new();
    for line in text.lines() {
        let Some(open) = line.find('[') else { continue };
        // `[2026-11-13T03:04:47Z]` : jour et heure = les 13 caractères après le crochet
        let hour = &line[open + 1..open + 14];
        let mut fields = line.split(' ');
        let user = fields.nth(2).unwrap_or("-");
        let bytes: u64 = fields.nth(5).and_then(|b| b.parse().ok()).unwrap_or(0);
        if user != "-" {
            *volumes.entry((user, hour)).or_insert(0) += bytes as u32;
        }
    }
    println!("{} couples (utilisateur, heure)", volumes.len());
    let mut alerts = 0;
    for ((user, hour), bytes) in &volumes {
        if u64::from(*bytes) >= THRESHOLD {
            println!("ALERTE {user} {hour}h : {bytes} octets");
            alerts += 1;
        }
    }
    println!("{alerts} alerte(s)");
}
