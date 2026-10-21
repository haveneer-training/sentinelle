//! sentinelle-turbo — « la version rapide de Sentinelle », écrite par un collègue
//! pressé. Trois commandes :
//!
//! ```text
//! sentinelle-turbo report DIR               volumes web par utilisateur et par heure
//! sentinelle-turbo stats DIR [--threads N]  échecs SSH par adresse, en parallèle
//! sentinelle-turbo stream DIR [--workers N] la même chose, en flux
//! ```

mod load;
mod report;
mod stats;
mod stream;

use std::path::PathBuf;
use std::process::ExitCode;

fn option(args: &[String], name: &str, default: usize) -> usize {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(command), Some(dir)) = (args.first(), args.get(1)) else {
        eprintln!("usage : sentinelle-turbo report|stats|stream DIR [--threads N] [--workers N]");
        return ExitCode::FAILURE;
    };
    let dir = PathBuf::from(dir);
    match command.as_str() {
        "report" => report::run(&dir),
        "stats" => stats::run(&dir, option(&args, "--threads", 4)),
        "stream" => stream::run(&dir, option(&args, "--workers", 4)),
        _ => {
            eprintln!("commande inconnue : {command}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
