//! Lignes de `auth.log` : `TIMESTAMP HOST PROCESS[PID]: MESSAGE`.
//!
//! Une [`AuthEntry`] ne possède aucune chaîne : ses champs textuels sont des tranches de
//! la ligne analysée, d'où le paramètre de durée de vie `'a`.

use crate::ParseError;
use crate::time::Timestamp;
#[cfg_attr(not(feature = "step1"), allow(unused_imports))] // trou étape 1
use crate::time::parse_timestamp;
use std::net::IpAddr;

/// Une ligne de `auth.log`, analysée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthEntry<'a> {
    /// Heure affichée par le serveur (son horloge peut dériver).
    pub timestamp: Timestamp,
    pub host: &'a str,
    pub process: &'a str,
    /// Absent pour certains processus (`sudo`).
    pub pid: Option<u32>,
    pub event: AuthEvent<'a>,
}

/// Ce que raconte le message d'une ligne de `auth.log`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthEvent<'a> {
    /// `Accepted METHOD for USER from IP port PORT ssh2`
    Accepted {
        user: &'a str,
        ip: IpAddr,
        method: &'a str,
    },
    /// `Failed password for [invalid user] USER from IP port PORT ssh2`
    Failed {
        user: &'a str,
        ip: IpAddr,
        invalid_user: bool,
    },
    /// `Invalid user USER from IP port PORT`, toujours suivi d'un `Failed` sur le même compte.
    InvalidUser { user: &'a str, ip: IpAddr },
    /// `pam_unix(sshd:session): session opened for user USER(uid=UID) by (uid=0)`
    SessionOpened { user: &'a str },
    /// `pam_unix(sshd:session): session closed for user USER`
    SessionClosed { user: &'a str },
    /// `USER : TTY=… ; PWD=… ; USER=root ; COMMAND=CMD`
    Sudo { user: &'a str, command: &'a str },
    /// Tout le reste (cron, connexions coupées…), conservé tel quel.
    Other(&'a str),
}

impl<'a> AuthEvent<'a> {
    /// Le compte concerné, s'il y en a un.
    pub fn user(&self) -> Option<&'a str> {
        match *self {
            AuthEvent::Accepted { user, .. }
            | AuthEvent::Failed { user, .. }
            | AuthEvent::InvalidUser { user, .. }
            | AuthEvent::SessionOpened { user }
            | AuthEvent::SessionClosed { user }
            | AuthEvent::Sudo { user, .. } => Some(user),
            AuthEvent::Other(_) => None,
        }
    }
}

/// Analyse une ligne de `auth.log`.
///
/// La ligne n'est pas copiée : l'entrée renvoyée en emprunte des tranches et ne peut donc
/// pas lui survivre.
#[cfg_attr(not(feature = "step1"), allow(unused_variables))] // trou étape 1
pub fn parse_auth_line(line: &str) -> Result<AuthEntry<'_>, ParseError> {
    // TODO-STEP:1 Découper l'en-tête avec `split_once` : horodatage, serveur, puis
    // `PROCESS[PID]` ou `PROCESS` jusqu'au premier « : ». Le message est confié à
    // `parse_auth_event`. Chaque champ absent donne une `ParseError` précise.
    // SOLUTION-BEGIN
    if line.is_empty() {
        return Err(ParseError::Empty);
    }
    let (timestamp, rest) = line
        .split_once(' ')
        .ok_or(ParseError::MissingField("host"))?;
    let timestamp = parse_timestamp(timestamp).ok_or(ParseError::BadTimestamp)?;
    let (host, rest) = rest
        .split_once(' ')
        .ok_or(ParseError::MissingField("process"))?;
    let (tag, message) = rest
        .split_once(": ")
        .ok_or(ParseError::MissingField("message"))?;
    let (process, pid) = match tag.split_once('[') {
        Some((process, pid)) => {
            let pid = pid.strip_suffix(']').and_then(|pid| pid.parse().ok());
            (process, Some(pid.ok_or(ParseError::BadNumber("pid"))?))
        }
        None => (tag, None),
    };
    Ok(AuthEntry {
        timestamp,
        host,
        process,
        pid,
        event: parse_auth_event(message)?,
    })
    // SOLUTION-END
}

/// Analyse le message d'une ligne de `auth.log` (ce qui suit `PROCESS[PID]: `).
///
/// Un message inconnu n'est pas une erreur : il devient [`AuthEvent::Other`]. En revanche,
/// un message reconnu dont l'adresse est illisible en est une.
#[cfg_attr(not(feature = "step1"), allow(unused_variables))] // trou étape 1
pub fn parse_auth_event(message: &str) -> Result<AuthEvent<'_>, ParseError> {
    // TODO-STEP:1 Découper le message en mots et reconnaître chaque forme par un motif de
    // tranche (`match words[..] { ["Accepted", method, "for", user, "from", ip, ..] => … }`).
    // Les lignes `sudo` commencent par des espaces ; leur commande, qui en contient aussi,
    // est tout ce qui suit `COMMAND=`.
    // SOLUTION-BEGIN
    let words: Vec<&str> = message.split_whitespace().collect();
    let ip = |text: &str| text.parse::<IpAddr>().map_err(|_| ParseError::BadIp);
    let event = match words[..] {
        ["Accepted", method, "for", user, "from", addr, ..] => AuthEvent::Accepted {
            user,
            ip: ip(addr)?,
            method,
        },
        [
            "Failed",
            "password",
            "for",
            "invalid",
            "user",
            user,
            "from",
            addr,
            ..,
        ] => AuthEvent::Failed {
            user,
            ip: ip(addr)?,
            invalid_user: true,
        },
        ["Failed", "password", "for", user, "from", addr, ..] => AuthEvent::Failed {
            user,
            ip: ip(addr)?,
            invalid_user: false,
        },
        ["Invalid", "user", user, "from", addr, ..] => AuthEvent::InvalidUser {
            user,
            ip: ip(addr)?,
        },
        [
            "pam_unix(sshd:session):",
            "session",
            "opened",
            "for",
            "user",
            user_uid,
            ..,
        ] => {
            let user = user_uid.split_once('(').map_or(user_uid, |(user, _)| user);
            AuthEvent::SessionOpened { user }
        }
        [
            "pam_unix(sshd:session):",
            "session",
            "closed",
            "for",
            "user",
            user,
        ] => AuthEvent::SessionClosed { user },
        [user, ":", tty, ..] if tty.starts_with("TTY=") => {
            let (_, command) = message
                .split_once("COMMAND=")
                .ok_or(ParseError::MissingField("COMMAND"))?;
            AuthEvent::Sudo { user, command }
        }
        _ => AuthEvent::Other(message),
    };
    Ok(event)
    // SOLUTION-END
}
