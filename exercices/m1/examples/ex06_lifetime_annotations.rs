//! Exercice 6 — Annoter les durées de vie d'une fonction
//! Erreur attendue : E0106
//!
//! Le compilateur ne sait pas à quelle entrée la sortie de `before` est liée.
//! 1. Ajoutez les annotations : une première idée (`'a` partout) compile-t-elle avec ce
//!    `main` ? Pourquoi ?
//! 2. Trouvez l'annotation qui exprime exactement le contrat : la sortie emprunte `line`,
//!    jamais `separator`.

fn before(line: &str, separator: &str) -> &str {
    line.split_once(separator).map_or(line, |(head, _)| head)
}

fn main() {
    let line = String::from("2026-11-12T01:42:24Z srv-db-02 sudo: l.roux : COMMAND=/usr/bin/pg_dump");
    let head;
    {
        let separator = String::from(" sudo:");
        head = before(&line, &separator);
    }
    assert_eq!(head, "2026-11-12T01:42:24Z srv-db-02");
    println!("ex06 : OK");
}
