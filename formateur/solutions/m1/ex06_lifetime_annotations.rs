//! Exercice 6 — corrigé.
//!
//! Avec `fn before<'a>(line: &'a str, separator: &'a str) -> &'a str`, `'a` doit couvrir
//! les deux arguments : `separator`, détruit à la fin du bloc, ne vit pas assez longtemps
//! pour `head` (E0597). Le contrat exact ne relie la sortie qu'à `line`.

fn before<'a>(line: &'a str, separator: &str) -> &'a str {
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
