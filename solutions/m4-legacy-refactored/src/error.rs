//! Erreurs de chargement des journaux.

use std::fmt;
use std::path::{Path, PathBuf};

/// Un journal n'a pas pu être chargé.
#[derive(Debug)]
pub enum LoadError {
    /// Fichier illisible.
    Io {
        /// Le fichier en cause.
        path: PathBuf,
        /// L'erreur du système.
        source: std::io::Error,
    },
    /// Le fichier n'est pas de l'UTF-8 valide.
    InvalidUtf8 {
        /// Le fichier en cause.
        path: PathBuf,
        /// Position du premier octet invalide.
        offset: usize,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, source } => write!(f, "{} : {source}", path.display()),
            LoadError::InvalidUtf8 { path, offset } => {
                write!(f, "{} : UTF-8 invalide à l'octet {offset}", path.display())
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io { source, .. } => Some(source),
            LoadError::InvalidUtf8 { .. } => None,
        }
    }
}

/// Lit un fichier texte en signalant précisément ce qui échoue.
pub(crate) fn read_text(path: &Path) -> Result<String, LoadError> {
    let bytes = std::fs::read(path).map_err(|source| LoadError::Io {
        path: path.to_owned(),
        source,
    })?;
    String::from_utf8(bytes).map_err(|e| LoadError::InvalidUtf8 {
        path: path.to_owned(),
        offset: e.utf8_error().valid_up_to(),
    })
}
