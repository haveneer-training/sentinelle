//! Exercice 2 — Un tampon réutilisé
//! Erreur attendue : E0502
//!
//! Lire ligne à ligne dans un même `String` évite une allocation par ligne. Mais on
//! voudrait garder les noms d'utilisateur trouvés. Expliquez le conflit, puis proposez
//! **deux** corrections : l'une qui copie les noms, l'autre qui n'en copie aucun (en
//! changeant la façon de lire).
//!
//! Indice : <https://doc.rust-lang.org/std/str/index.html>

use std::io::BufRead;

const LOG: &str = "\
2026-11-11T22:37:02Z bastion-01 sshd[7]: Accepted password for l.roux from 198.51.100.227 port 4021 ssh2
2026-11-11T22:49:56Z srv-web-01 sshd[8]: Accepted password for l.roux from 10.0.0.10 port 4022 ssh2
2026-11-12T01:41:15Z srv-db-02 sshd[9]: Accepted password for a.martin from 10.1.0.3 port 4023 ssh2
";

fn user_of(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once(" for ")?;
    rest.split(' ').next()
}

fn main() {
    let mut reader = LOG.as_bytes();
    let mut buffer = String::new();
    let mut users: Vec<&str> = Vec::new();
    loop {
        buffer.clear();
        if reader.read_line(&mut buffer).unwrap() == 0 {
            break;
        }
        if let Some(user) = user_of(&buffer) {
            users.push(user);
        }
    }
    assert_eq!(users, ["l.roux", "l.roux", "a.martin"]);
    println!("ex02 : OK");
}
