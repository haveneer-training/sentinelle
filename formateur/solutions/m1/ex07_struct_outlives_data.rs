//! Exercice 7 — corrigé.
//!
//! `text` est détruit à la fin du bloc alors que `report` l'emprunte encore. Sans copie :
//! faire vivre `text` aussi longtemps que le rapport. Avec copie : `Report` possède sa
//! ligne (`String`), au prix d'une allocation, mais il devient indépendant du texte.

struct Report<'a> {
    worst_line: &'a str,
}

struct OwnedReport {
    worst_line: String,
}

fn load() -> String {
    "ok\nFailed password for root from 192.0.2.67\nok\n".to_string()
}

fn main() {
    let text = load();
    let report = Report {
        worst_line: text.lines().find(|l| l.starts_with("Failed")).unwrap_or(""),
    };
    assert_eq!(
        report.worst_line,
        "Failed password for root from 192.0.2.67"
    );

    let owned;
    {
        let text = load();
        owned = OwnedReport {
            worst_line: text
                .lines()
                .find(|l| l.starts_with("Failed"))
                .unwrap_or("")
                .to_string(),
        };
    }
    assert_eq!(owned.worst_line, "Failed password for root from 192.0.2.67");
    println!("ex07 : OK");
}
