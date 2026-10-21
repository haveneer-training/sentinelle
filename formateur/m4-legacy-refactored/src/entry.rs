//! Les entrées analysées.
//!
//! Seuls les champs effectivement lus par les analyses sont conservés : la v0.1 en
//! copiait huit par ligne web pour n'en lire que deux.

/// Nature d'une ligne de `auth.log`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKind {
    /// `Failed password for …`, compte existant ou non.
    Failed,
    /// `Accepted … for …`.
    Accepted,
    /// `Invalid user …`.
    InvalidUser,
    /// Toute autre ligne.
    Other,
}

/// Une ligne de `auth.log`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthEntry {
    /// Nature de la ligne.
    pub kind: AuthKind,
    /// Compte concerné (vide pour [`AuthKind::Other`]).
    pub user: String,
    /// Adresse d'origine (vide pour [`AuthKind::Other`]).
    pub ip: String,
}

/// Une ligne de `access.log`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebEntry {
    /// Utilisateur authentifié, ou `-`.
    pub user: String,
    /// Taille de la réponse, ou `-`.
    pub bytes: String,
}
