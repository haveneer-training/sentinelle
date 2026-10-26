//! Exercice 6 — Annoter les durées de vie d'une fonction (corrigé)
//! Erreur attendue : E0106
//!
//! Le compilateur ne sait pas à quelle entrée la sortie de `before` est liée.
//! 1. Ajoutez les annotations : une première idée (`'a` partout) compile-t-elle avec ce
//!    `main` ? Pourquoi ?
//! 2. Trouvez l'annotation qui exprime exactement le contrat : la sortie emprunte `line`,
//!    jamais `separator`.
//!
//! # Corrigé
//!
//! Avec `fn before<'a>(line: &'a str, separator: &'a str) -> &'a str`, `'a` doit couvrir
//! les deux arguments : `separator`, détruit à la fin du bloc, ne vit pas assez longtemps
//! pour `head` (E0597). Le contrat exact ne relie la sortie qu'à `line`.

fn before<'a>(line: &'a str, separator: &str) -> &'a str {
    line.split_once(separator).map_or(line, |(head, _)| head)
}

fn main() {
    let line =
        String::from("2026-11-12T01:42:24Z srv-db-02 sudo: l.roux : COMMAND=/usr/bin/pg_dump");
    let head;
    {
        let separator = String::from(" sudo:");
        head = before(&line, &separator);
    }
    assert_eq!(head, "2026-11-12T01:42:24Z srv-db-02");
    println!("ex06 : OK");
}
