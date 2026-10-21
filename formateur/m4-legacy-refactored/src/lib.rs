//! Sentinelle v0.2 : la v0.1 refactorisée, **à comportement constant**.
//!
//! La sortie est identique octet pour octet à celle de la v0.1 (le test
//! `tests/golden.rs` le vérifie), défauts compris : corriger un comportement est une
//! étape distincte, qui modifie la référence en connaissance de cause (voir
//! [`analysis::count_failures_by_ip`]).
//!
//! | Module | Rôle |
//! |---|---|
//! | [`entry`] | les entrées analysées |
//! | [`parse`] | lecture d'une ligne |
//! | [`analysis`] | comptages et détections |
//! | [`report`] | mise en forme du rapport |
//! | [`error`] | erreurs de chargement |

#![warn(missing_docs)]

pub mod analysis;
pub mod entry;
pub mod error;
pub mod parse;
pub mod report;

use std::path::Path;

/// Les deux journaux analysés.
#[derive(Debug, Default)]
pub struct Journals {
    /// Entrées de `auth.log`.
    pub auth: Vec<entry::AuthEntry>,
    /// Lignes de `auth.log` ignorées.
    pub auth_skipped: usize,
    /// Entrées de `access.log`.
    pub web: Vec<entry::WebEntry>,
    /// Lignes de `access.log` ignorées.
    pub web_skipped: usize,
}

/// Charge et analyse `dir/auth.log` et `dir/access.log`.
///
/// # Errors
///
/// [`error::LoadError`] si un fichier est illisible ou n'est pas de l'UTF-8 valide.
pub fn load(dir: &Path) -> Result<Journals, error::LoadError> {
    let auth_text = error::read_text(&dir.join("auth.log"))?;
    let web_text = error::read_text(&dir.join("access.log"))?;
    let mut journals = Journals::default();
    for line in auth_text.lines() {
        match parse::parse_auth_line(line) {
            Some(entry) => journals.auth.push(entry),
            None => journals.auth_skipped += 1,
        }
    }
    for line in web_text.lines() {
        match parse::parse_web_line(line) {
            Some(entry) => journals.web.push(entry),
            None => journals.web_skipped += 1,
        }
    }
    Ok(journals)
}
