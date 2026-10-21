//! `sentinelle-v0 [DIR]` : rapport sur `DIR/auth.log` et `DIR/access.log`.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "data".to_string()));
    let started = Instant::now();
    match sentinelle_legacy_refactored::load(&dir) {
        Ok(journals) => {
            print!("{}", sentinelle_legacy_refactored::report::render(&journals));
            eprintln!("\nAnalyse terminee en {:?}", started.elapsed());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("erreur : {error}");
            ExitCode::FAILURE
        }
    }
}
