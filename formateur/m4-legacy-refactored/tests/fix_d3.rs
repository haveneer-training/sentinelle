//! Relecture automatique de la référence corrigée (étape 7) : `expected-fix-d3.txt` ne
//! diffère de celle de la v0.1 que là où D3 intervient, et ses comptes sont ceux des
//! lignes `Failed password` du jeu de données, recomptées ici sans le parseur.

use sentinelle_gen::Dataset;
use std::path::PathBuf;

fn read(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} : {e}", path.display()))
}

/// Les lignes de la section `--- {title} ---`, jusqu'à la ligne vide suivante.
fn section<'a>(report: &'a str, title: &str) -> Vec<&'a str> {
    report
        .lines()
        .skip_while(|line| !line.starts_with(&format!("--- {title}")))
        .skip(1)
        .take_while(|line| !line.is_empty())
        .collect()
}

/// Le rapport privé des sections que D3 modifie.
fn without_failures(report: &str) -> Vec<&str> {
    let mut kept = Vec::new();
    let mut skipping = false;
    for line in report.lines() {
        if line.starts_with("--- ") {
            skipping = line.starts_with("--- Top 10 des IP") || line.starts_with("--- Force brute");
        }
        if !skipping {
            kept.push(line);
        }
    }
    kept
}

#[test]
fn only_failure_counts_change() {
    let legacy = read("../../legacy/tests/golden/expected.txt");
    let fixed = read("tests/golden/expected-fix-d3.txt");
    assert_ne!(legacy, fixed, "la correction doit changer le rapport");
    assert_eq!(without_failures(&legacy), without_failures(&fixed));
}

#[test]
fn failure_counts_match_failed_password_lines() {
    let data = Dataset::small(2026);
    let auth = String::from_utf8_lossy(&data.auth);
    let fixed = read("tests/golden/expected-fix-d3.txt");
    let top = section(&fixed, "Top 10 des IP en echec");
    assert_eq!(top.len(), 10);
    for line in top {
        let (ip, count) = line.trim().split_once(char::is_whitespace).unwrap();
        let count: usize = count.trim().parse().unwrap();
        let from = format!(" from {ip} ");
        let expected = auth
            .lines()
            .filter(|l| l.contains("]: Failed password for ") && l.contains(&from))
            .count();
        assert_eq!(count, expected, "{ip}");
    }
}

#[test]
fn bruteforce_section_follows_fixed_counts() {
    let fixed = read("tests/golden/expected-fix-d3.txt");
    let above: Vec<&str> = section(&fixed, "Top 10 des IP en echec")
        .into_iter()
        .filter(|line| {
            line.split_whitespace()
                .nth(1)
                .unwrap()
                .parse::<usize>()
                .unwrap()
                > 100
        })
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    let detected: Vec<&str> = section(&fixed, "Force brute")
        .iter()
        .map(|l| l.trim())
        .collect();
    assert_eq!(detected, above);
}
