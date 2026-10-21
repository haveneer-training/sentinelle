//! Exercice 2 — corrigé.
//!
//! `users` garde des tranches de `buffer`, que `clear` puis `read_line` modifient.
//! Deux solutions : posséder les noms (`String`), ou emprunter au texte complet, qui
//! vit plus longtemps que la table (c'est le choix de Sentinelle).

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
    // 1. copier les noms : une allocation par nom trouvé, mais le tampon reste réutilisé
    let mut reader = LOG.as_bytes();
    let mut buffer = String::new();
    let mut owned: Vec<String> = Vec::new();
    loop {
        buffer.clear();
        if reader.read_line(&mut buffer).unwrap() == 0 {
            break;
        }
        if let Some(user) = user_of(&buffer) {
            owned.push(user.to_string());
        }
    }
    assert_eq!(owned, ["l.roux", "l.roux", "a.martin"]);

    // 2. aucune copie : on emprunte au texte entier, qui survit à `users`
    let users: Vec<&str> = LOG.lines().filter_map(user_of).collect();
    assert_eq!(users, ["l.roux", "l.roux", "a.martin"]);
    println!("ex02 : OK");
}
