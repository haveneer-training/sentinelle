//! Jeux de données partagés par les tests : engendrés une fois par binaire de test.

#![allow(dead_code)] // chaque binaire de test n'utilise qu'une partie de ces aides

use sentinelle_gen::{Dataset, Options};
use std::sync::OnceLock;

/// Petit jeu de données (≈ 2 Mo), graine fixe.
pub fn dataset() -> &'static Dataset {
    static DATASET: OnceLock<Dataset> = OnceLock::new();
    DATASET.get_or_init(|| Dataset::small(2026))
}

/// Jeu de données dont les lignes corrompues contiennent de l'UTF-8 invalide.
pub fn dataset_with_invalid_utf8() -> &'static Dataset {
    static DATASET: OnceLock<Dataset> = OnceLock::new();
    DATASET.get_or_init(|| {
        Dataset::generate(&Options {
            seed: 7,
            users: 40,
            noise: 0.3,
            malformed: 0.01,
            invalid_utf8: true,
            ..Options::default()
        })
    })
}

/// Le compte compromis dans `dataset`.
pub fn victim(dataset: &Dataset) -> &str {
    &dataset.generated.world.users[dataset.generated.scenario.victim].name
}
