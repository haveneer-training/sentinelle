//! Erreurs d'analyse d'une ligne de journal.

use std::fmt;

/// Pourquoi une ligne n'a pas pu être analysée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Ligne vide.
    Empty,
    /// La ligne n'est pas de l'UTF-8 valide.
    InvalidUtf8,
    /// Un champ attendu manque (ligne tronquée ou amputée).
    MissingField(&'static str),
    /// Horodatage illisible.
    BadTimestamp,
    /// Adresse IP illisible.
    BadIp,
    /// Champ numérique illisible.
    BadNumber(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "ligne vide"),
            ParseError::InvalidUtf8 => write!(f, "UTF-8 invalide"),
            ParseError::MissingField(field) => write!(f, "champ manquant : {field}"),
            ParseError::BadTimestamp => write!(f, "horodatage illisible"),
            ParseError::BadIp => write!(f, "adresse IP illisible"),
            ParseError::BadNumber(field) => write!(f, "nombre illisible : {field}"),
        }
    }
}

impl std::error::Error for ParseError {}
