//! Exercice 9 — L'élision dans une méthode
//! Erreur attendue : lifetime may not live long enough
//!
//! Pour une méthode `&self`, la sortie est liée à `self` par défaut (troisième règle
//! d'élision). Ici, elle peut aussi venir de `fallback`. Annotez `display_name` pour que
//! ce soit accepté, puis vérifiez que `main` compile toujours.

use std::collections::HashMap;

struct Directory {
    names: HashMap<String, String>,
}

impl Directory {
    fn display_name(&self, login: &str, fallback: &str) -> &str {
        self.names.get(login).map(String::as_str).unwrap_or(fallback)
    }
}

fn main() {
    let directory = Directory { names: HashMap::from([("l.roux".to_string(), "Léa Roux".to_string())]) };
    let unknown = "inconnu";
    assert_eq!(directory.display_name("l.roux", unknown), "Léa Roux");
    assert_eq!(directory.display_name("x.y", unknown), "inconnu");
    println!("ex09 : OK");
}
