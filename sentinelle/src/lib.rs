//! Sentinelle : analyse des journaux d'authentification (`auth.log`) et d'accès web
//! (`access.log`) à la recherche d'une intrusion.
//!
//! Les entrées analysées **empruntent** le texte du journal : aucune chaîne n'est copiée
//! entre la lecture du fichier et le rapport final.
//!
//! | Module | Rôle |
//! |---|---|
//! | [`time`] | horodatages ISO 8601 |
//! | [`auth`] | lignes de `auth.log` |
//! | [`web`] | lignes de `access.log` |
//! | [`journal`] | un fichier entier, lignes en erreur comprises |
//! | [`detect`] | les détections, une collection adaptée à chacune |
//! | [`summary`] | agrégats fusionnables, calculés en parallèle |

pub mod auth;
pub mod detect;
pub mod error;
pub mod journal;
pub mod summary;
pub mod time;
pub mod web;

// Corrigés des bonus, lus seulement avec la feature : sans elle, les fichiers peuvent
// manquer (dossier de travail).
#[cfg(feature = "bonus")]
#[path = "../../solutions/bonus-distributed/detect_distributed.rs"]
pub mod bonus_distributed;
#[cfg(feature = "bonus")]
#[path = "../../solutions/bonus-injection/detect_injection.rs"]
pub mod bonus_injection;

pub use error::ParseError;
