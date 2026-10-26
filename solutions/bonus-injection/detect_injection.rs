// Bonus « noms de compte piégés » : étapes 3 et 4 de docs/bonus/injection.md.
// À ajouter à `sentinelle/src/detect.rs` (ou dans un module à part).

use sentinelle::auth::{AuthEntry, AuthEvent};
use sentinelle::time::Timestamp;
use std::collections::HashSet;
use std::net::IpAddr;

/// Ce qui rend un nom de compte suspect, du plus grave au moins grave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oddity {
    /// `\n`, `\t`… : de quoi fabriquer une fausse ligne.
    Control,
    /// Ne s'affiche pas, ou change le sens d'affichage de la suite.
    Invisible,
    /// Lettre d'une autre écriture : homoglyphe possible (`о` cyrillique).
    NonAscii,
}

fn invisible(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
}

pub fn oddity(user: &str) -> Option<Oddity> {
    if user.chars().any(char::is_control) {
        Some(Oddity::Control)
    } else if user.chars().any(invisible) {
        Some(Oddity::Invisible)
    } else if !user.is_ascii() {
        Some(Oddity::NonAscii)
    } else {
        None
    }
}

/// Un nom de compte suspect, vu pour la première fois à `at` depuis `ip`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OddUser<'a> {
    pub user: &'a str,
    pub oddity: Oddity,
    pub ip: Option<IpAddr>,
    pub at: Timestamp,
}

/// Chaque nom suspect n'est rapporté qu'une fois, dans l'ordre de `entries`.
/// À afficher avec `{:?}` : `{}` laisserait le terminal interpréter les caractères piégés.
pub fn detect_odd_users<'a>(entries: &[AuthEntry<'a>]) -> Vec<OddUser<'a>> {
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for entry in entries {
        let Some(user) = entry.event.user() else {
            continue;
        };
        let Some(oddity) = oddity(user) else {
            continue;
        };
        if seen.insert(user) {
            let ip = match entry.event {
                AuthEvent::Accepted { ip, .. }
                | AuthEvent::Failed { ip, .. }
                | AuthEvent::InvalidUser { ip, .. } => Some(ip),
                _ => None,
            };
            found.push(OddUser {
                user,
                oddity,
                ip,
                at: entry.timestamp,
            });
        }
    }
    found
}

/// Les `Accepted` sans ouverture de session du même processus (même serveur, même PID).
/// sshd écrit toujours les deux ; une ligne forgée n'a pas sa suite.
pub fn detect_ghost_logins<'a>(entries: &[AuthEntry<'a>]) -> Vec<AuthEntry<'a>> {
    let sessions: HashSet<(&str, Option<u32>)> = entries
        .iter()
        .filter(|e| matches!(e.event, AuthEvent::SessionOpened { .. }))
        .map(|e| (e.host, e.pid))
        .collect();
    entries
        .iter()
        .filter(|e| matches!(e.event, AuthEvent::Accepted { .. }))
        .filter(|e| !sessions.contains(&(e.host, e.pid)))
        .copied()
        .collect()
}
