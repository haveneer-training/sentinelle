//! Exercice 5 — Renvoyer une référence vers une variable locale (corrigé)
//! Erreur attendue : E0515
//!
//! `label` fabrique une étiquette « utilisateur@serveur ». Pourquoi ne peut-elle pas
//! renvoyer un `&str` ? Corrigez la signature. Puis écrivez `user_part`, qui, elle,
//! peut renvoyer une référence : dans quoi pointe-t-elle ?
//!
//! # Corrigé
//!
//! L'étiquette est une chaîne **nouvelle** : elle doit être possédée par l'appelant.
//! `user_part`, elle, renvoie une tranche de `session.user` : l'élision la lie à `session`.

struct Session {
    user: String,
    host: String,
}

fn label(session: &Session) -> String {
    format!("{}@{}", session.user, session.host)
}

fn user_part(session: &Session) -> &str {
    session.user.split('.').next_back().unwrap_or(&session.user)
}

fn main() {
    let session = Session {
        user: "l.roux".into(),
        host: "srv-db-02".into(),
    };
    assert_eq!(label(&session), "l.roux@srv-db-02");
    assert_eq!(user_part(&session), "roux");
    println!("ex05 : OK");
}
