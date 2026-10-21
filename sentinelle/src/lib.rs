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

pub use error::ParseError;
