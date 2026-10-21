//! Mise en forme du rapport, identique à celle de la v0.1.

use crate::Journals;
use crate::analysis::{bytes_by_user, count_failures_by_ip, detect_bruteforce, detect_spraying};
use std::fmt::Write;

/// Le rapport complet, tel qu'affiché sur la sortie standard.
pub fn render(journals: &Journals) -> String {
    let mut out = String::new();
    // écrire dans une `String` ne peut pas échouer
    let _ = write_report(&mut out, journals);
    out
}

fn write_report(out: &mut String, journals: &Journals) -> std::fmt::Result {
    writeln!(out, "=== Sentinelle v0.1 ===")?;
    writeln!(
        out,
        "auth.log   : {} entrees ({} ignorees)",
        journals.auth.len(),
        journals.auth_skipped
    )?;
    writeln!(
        out,
        "access.log : {} entrees ({} ignorees)",
        journals.web.len(),
        journals.web_skipped
    )?;

    let failures = count_failures_by_ip(&journals.auth);
    writeln!(out, "\n--- Top 10 des IP en echec ---")?;
    for (ip, count) in failures.iter().take(10) {
        writeln!(out, "{ip:>18} {count:>6}")?;
    }
    writeln!(out, "\n--- Force brute (plus de 100 echecs) ---")?;
    for ip in detect_bruteforce(&failures) {
        writeln!(out, "{ip:>18}")?;
    }
    writeln!(out, "\n--- Password spraying (plus de 20 comptes) ---")?;
    for (ip, accounts) in detect_spraying(&journals.auth) {
        writeln!(out, "{ip:>18} {accounts:>6} comptes")?;
    }
    writeln!(out, "\n--- Top 5 des volumes web par utilisateur ---")?;
    for (user, bytes) in bytes_by_user(&journals.web).iter().take(5) {
        writeln!(out, "{user:>18} {bytes:>14} octets")?;
    }
    Ok(())
}
