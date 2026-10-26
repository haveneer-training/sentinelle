// Module 5, optimisation O1 : `parse_auth_event` sans allocation.
// Remplace le corps de `sentinelle/src/auth.rs::parse_auth_event`.
//
// Le `Vec<&str>` est remplacé par un tableau de taille fixe : aucun motif ne regarde
// plus de 8 mots, et un message plus long n'est comparé qu'à des motifs ouverts (`..`)
// ou de longueur exacte inférieure ; le comportement est donc inchangé.

// pour la mesure (perf/benches) ; inutile une fois collé dans auth.rs
use super::{AuthEvent, IpAddr, ParseError};

pub fn parse_auth_event(message: &str) -> Result<AuthEvent<'_>, ParseError> {
    let mut buffer = [""; 9];
    let mut count = 0;
    for (slot, word) in buffer.iter_mut().zip(message.split_whitespace()) {
        *slot = word;
        count += 1;
    }
    let words = &buffer[..count];
    let ip = |text: &str| text.parse::<IpAddr>().map_err(|_| ParseError::BadIp);
    let event = match *words {
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
}
