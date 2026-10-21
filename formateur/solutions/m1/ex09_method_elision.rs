//! Exercice 9 — corrigé.
//!
//! La sortie peut venir de `self` ou de `fallback` : elle doit vivre moins longtemps que
//! les deux. `login` reste libre (élidé), puisqu'il n'est jamais renvoyé.

use std::collections::HashMap;

struct Directory {
    names: HashMap<String, String>,
}

impl Directory {
    fn display_name<'a>(&'a self, login: &str, fallback: &'a str) -> &'a str {
        self.names
            .get(login)
            .map(String::as_str)
            .unwrap_or(fallback)
    }
}

fn main() {
    let directory = Directory {
        names: HashMap::from([("l.roux".to_string(), "Léa Roux".to_string())]),
    };
    let unknown = "inconnu";
    assert_eq!(directory.display_name("l.roux", unknown), "Léa Roux");
    assert_eq!(directory.display_name("x.y", unknown), "inconnu");
    println!("ex09 : OK");
}
