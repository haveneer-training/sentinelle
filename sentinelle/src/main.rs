//! `sentinelle [DIR] [--threads N]` : analyse `DIR/auth.log` et `DIR/access.log`.

use sentinelle::detect::{Config, analyze, failures_by_ip, top_n};
use sentinelle::journal::{load, parse_auth, parse_web};
use sentinelle::summary::summarize_scoped;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let mut dir = PathBuf::from("data");
    let mut threads = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--threads" => match args.next().and_then(|n| n.parse::<usize>().ok()) {
                Some(n) if n > 0 => threads = Some(n),
                _ => {
                    eprintln!("--threads attend un entier strictement positif");
                    return ExitCode::FAILURE;
                }
            },
            _ => dir = PathBuf::from(arg),
        }
    }

    let started = Instant::now();
    let (auth_text, web_text) = match (load(&dir.join("auth.log")), load(&dir.join("access.log"))) {
        (Ok(auth), Ok(web)) => (auth, web),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("lecture des journaux de {} impossible : {e}", dir.display());
            return ExitCode::FAILURE;
        }
    };
    let auth = parse_auth(&auth_text);
    let web = parse_web(&web_text);
    let parsed = started.elapsed();

    println!("=== Sentinelle ===");
    println!(
        "auth.log   : {} entrées ({} rejetées)",
        auth.entries.len(),
        auth.errors.len()
    );
    println!(
        "access.log : {} entrées ({} rejetées)",
        web.entries.len(),
        web.errors.len()
    );

    println!("\n--- Adresses en échec (top 10) ---");
    for (ip, failures) in top_n(&failures_by_ip(&auth.entries), 10) {
        println!("{ip:>18} {failures:>6}");
    }

    let Some(first) = auth.entries.first() else {
        println!("\nAucune entrée : rien à analyser.");
        return ExitCode::SUCCESS;
    };
    let config = Config::starting_at(first.timestamp);
    println!("\n--- Chronologie des alertes ---");
    for alert in analyze(&auth.entries, &web.entries, &config) {
        println!("{alert}");
    }
    eprintln!(
        "\nlecture et analyse syntaxique : {parsed:.1?}, total : {:.1?}",
        started.elapsed()
    );

    if let Some(threads) = threads {
        let started = Instant::now();
        let summary = summarize_scoped(&auth_text, threads);
        eprintln!(
            "résumé sur {threads} threads : {} lignes en {:.1?}",
            summary.lines,
            started.elapsed()
        );
    }
    ExitCode::SUCCESS
}
