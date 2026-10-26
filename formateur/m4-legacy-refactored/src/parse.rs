//! Lecture d'une ligne de journal.
//!
//! Le découpage par position de mot est celui de la v0.1, conservé à l'identique : une
//! ligne amputée d'un champ est encore lue de travers (`user = "from"`), sans erreur.

use crate::entry::{AuthEntry, AuthKind, WebEntry};

/// Lit une ligne de `auth.log` ; `None` si elle a moins de quatre mots.
///
/// ```
/// use sentinelle_legacy_refactored::{entry::AuthKind, parse::parse_auth_line};
///
/// let line = "2026-11-09T09:41:12Z bastion-01 sshd[7]: Failed password for root from 192.0.2.7 port 22 ssh2";
/// let entry = parse_auth_line(line).unwrap();
/// assert_eq!((entry.kind, entry.user.as_str(), entry.ip.as_str()), (AuthKind::Failed, "root", "192.0.2.7"));
/// assert_eq!(parse_auth_line("tronquée"), None);
/// ```
pub fn parse_auth_line(line: &str) -> Option<AuthEntry> {
    // le message suit les trois premiers mots ; pas de quatrième mot, pas d'entrée
    let message = line.splitn(4, ' ').nth(3)?;
    let words: Vec<&str> = message.split(' ').collect();
    // positions des mots dans le message (la v0.1 comptait depuis le début de la ligne,
    // trois mots plus tôt)
    let (kind, user_at, ip_at) = if message.starts_with("Failed password for invalid user ") {
        (AuthKind::Failed, 5, 7)
    } else if message.starts_with("Failed password for ") {
        (AuthKind::Failed, 3, 5)
    } else if message.starts_with("Accepted ") {
        (AuthKind::Accepted, 3, 5)
    } else if message.starts_with("Invalid user ") {
        (AuthKind::InvalidUser, 2, 4)
    } else {
        return Some(AuthEntry {
            kind: AuthKind::Other,
            user: String::new(),
            ip: String::new(),
        });
    };
    let word = |i: usize| words.get(i).copied().unwrap_or_default().to_string();
    Some(AuthEntry {
        kind,
        user: word(user_at),
        ip: word(ip_at),
    })
}

/// Lit une ligne de `access.log` ; `None` si elle a moins de dix mots.
///
/// ```
/// use sentinelle_legacy_refactored::parse::parse_web_line;
///
/// let line = r#"10.1.0.42 - v.michel [2026-11-09T09:43:02Z] "GET / HTTP/1.1" 200 1834 44"#;
/// let entry = parse_web_line(line).unwrap();
/// assert_eq!((entry.user.as_str(), entry.bytes), ("v.michel", Some(1834)));
/// assert_eq!(parse_web_line("10.1.0.42 - v.michel"), None);
/// ```
pub fn parse_web_line(line: &str) -> Option<WebEntry> {
    let words: Vec<&str> = line.split(' ').collect();
    if words.len() < 10 {
        return None;
    }
    Some(WebEntry {
        user: words[2].to_string(),
        bytes: words[8].parse().ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREFIX: &str = "2026-11-09T09:41:12Z bastion-01 sshd[7]:";

    #[test]
    fn invalid_user_line() {
        let entry = parse_auth_line(&format!(
            "{PREFIX} Invalid user admin from 192.0.2.7 port 22"
        ))
        .unwrap();
        assert_eq!(
            (entry.kind, entry.user.as_str(), entry.ip.as_str()),
            (AuthKind::InvalidUser, "admin", "192.0.2.7")
        );
    }

    #[test]
    fn failed_invalid_user_line() {
        let line =
            format!("{PREFIX} Failed password for invalid user admin from 192.0.2.7 port 22 ssh2");
        let entry = parse_auth_line(&line).unwrap();
        assert_eq!(
            (entry.kind, entry.user.as_str(), entry.ip.as_str()),
            (AuthKind::Failed, "admin", "192.0.2.7")
        );
    }

    #[test]
    fn other_line_has_no_user() {
        let entry =
            parse_auth_line(&format!("{PREFIX} Connection closed by 192.0.2.7 port 22")).unwrap();
        assert_eq!(
            (entry.kind, entry.user.as_str(), entry.ip.as_str()),
            (AuthKind::Other, "", "")
        );
    }

    /// **Défaut conservé (D7)**, caractérisé : sans le compte, les mots glissent d'un cran.
    /// Ce test changera avec la correction, en connaissance de cause.
    #[test]
    fn line_missing_a_field_is_misread() {
        let entry = parse_auth_line(&format!(
            "{PREFIX} Failed password for from 192.0.2.7 port 22 ssh2"
        ))
        .unwrap();
        assert_eq!(
            (entry.kind, entry.user.as_str(), entry.ip.as_str()),
            (AuthKind::Failed, "from", "port")
        );
    }

    #[test]
    fn short_web_line_is_skipped() {
        assert_eq!(
            parse_web_line("10.1.0.42 - v.michel [2026-11-09T09:43:02Z]"),
            None
        );
    }

    #[test]
    fn web_size_dash_has_no_size() {
        let line = r#"10.1.0.42 - - [2026-11-09T09:43:02Z] "GET / HTTP/1.1" 304 - 12"#;
        let entry = parse_web_line(line).unwrap();
        assert_eq!((entry.user.as_str(), entry.bytes), ("-", None));
    }
}
