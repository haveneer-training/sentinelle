//! Chargement des journaux.

use std::path::Path;

/// Le contenu d'un journal.
pub fn read_log(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

/// L'adresse et le compte d'une ligne `Failed password` ou `Accepted`.
pub fn failed_or_accepted(line: &str) -> Option<(bool, &str, &str)> {
    let (_, message) = line.split_once("]: ")?;
    let words: Vec<&str> = message.split_whitespace().collect();
    match words[..] {
        [
            "Failed",
            "password",
            "for",
            "invalid",
            "user",
            user,
            "from",
            ip,
            ..,
        ] => Some((true, user, ip)),
        ["Failed", "password", "for", user, "from", ip, ..] => Some((true, user, ip)),
        ["Accepted", _, "for", user, "from", ip, ..] => Some((false, user, ip)),
        _ => None,
    }
}
