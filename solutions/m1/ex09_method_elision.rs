//! Exercice 9 — L'élision dans une méthode (corrigé)
//! Erreur attendue : lifetime may not live long enough
//!
//! Pour une méthode `&self`, la sortie est liée à `self` par défaut (troisième règle
//! d'élision). Ici, elle peut aussi venir de `fallback`. Annotez `display_name` pour que
//! ce soit accepté, puis vérifiez que `main` compile toujours.
//!
//! # Corrigé
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
