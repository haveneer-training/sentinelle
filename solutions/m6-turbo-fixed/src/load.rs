//! Chargement des journaux.

use std::path::Path;

/// Le contenu brut d'un journal.
///
/// Ticket 3 : `read_to_string` refusait le fichier entier au premier octet invalide. Les
/// octets sont lus tels quels, et chaque ligne est validée à part (voir [`lines`]).
pub fn read_log(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{} : {e}", path.display()))
}

/// Les lignes en UTF-8 valide ; les autres sont ignorées.
pub fn lines(bytes: &[u8]) -> impl Iterator<Item = &str> {
    bytes
        .split(|&b| b == b'\n')
        .filter_map(|line| std::str::from_utf8(line).ok())
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
