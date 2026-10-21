//! Exercice 7 — Une structure qui survit à ses données
//! Erreur attendue : E0597
//!
//! `Report` emprunte le texte du journal. Le rapport est construit dans un bloc, puis
//! utilisé après. Lisez le message d'erreur : quelle valeur est détruite trop tôt, et
//! où ? Corrigez sans rien copier, puis en copiant ; quand préférer l'un ou l'autre ?

struct Report<'a> {
    worst_line: &'a str,
}

fn load() -> String {
    "ok\nFailed password for root from 192.0.2.67\nok\n".to_string()
}

fn main() {
    let report;
    {
        let text = load();
        let worst_line = text.lines().find(|l| l.starts_with("Failed")).unwrap_or("");
        report = Report { worst_line };
    }
    assert_eq!(
        report.worst_line,
        "Failed password for root from 192.0.2.67"
    );
    println!("ex07 : OK");
}
