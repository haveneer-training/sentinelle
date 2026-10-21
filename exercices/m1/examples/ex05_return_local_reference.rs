//! Exercice 5 — Renvoyer une référence vers une variable locale
//! Erreur attendue : E0515
//!
//! `label` fabrique une étiquette « utilisateur@serveur ». Pourquoi ne peut-elle pas
//! renvoyer un `&str` ? Corrigez la signature. Puis écrivez `user_part`, qui, elle,
//! peut renvoyer une référence : dans quoi pointe-t-elle ?

struct Session {
    user: String,
    host: String,
}

fn label(session: &Session) -> &str {
    let text = format!("{}@{}", session.user, session.host);
    &text
}

fn main() {
    let session = Session { user: "l.roux".into(), host: "srv-db-02".into() };
    assert_eq!(label(&session), "l.roux@srv-db-02");
    println!("ex05 : OK");
}
