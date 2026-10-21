//! Passage à l'échelle des résumés parallèles (étapes 8 à 10).
//!
//! ```shell
//! cargo run --release -p sentinelle-gen -- --out data-800M --users 20000 --noise 50
//! cargo run --release -p sentinelle --example scaling -- data-800M
//! ```
//!
//! Pour chaque nombre de threads, affiche la durée et l'accélération par rapport à la
//! version séquentielle. À interpréter : quelle part du travail reste séquentielle
//! (loi d'Amdahl) ? que coûte le verrou partagé ?

use sentinelle::journal::load;
use sentinelle::summary::{summarize, summarize_pipeline, summarize_scoped, summarize_shared};
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let started = Instant::now();
    let value = f();
    (value, started.elapsed())
}

fn main() -> std::io::Result<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "data".into()));
    let path = dir.join("auth.log");
    let (text, read) = timed(|| load(&path));
    let text = text?;
    println!(
        "{} : {:.1} Mo lus en {read:.1?}\n",
        path.display(),
        text.len() as f64 / 1e6
    );

    let (reference, sequential) = timed(|| summarize(&text));
    println!(
        "séquentiel : {sequential:.1?} ({} lignes)\n",
        reference.lines
    );
    println!(
        "{:>8} {:>12} {:>12} {:>12}",
        "threads", "scope", "pipeline", "partagé"
    );

    let max = std::thread::available_parallelism().map_or(8, |n| n.get());
    let shared_text = Arc::new(text.clone());
    let mut threads = 1;
    while threads <= max {
        let (_, scoped) = timed(|| summarize_scoped(&text, threads));
        let (_, pipeline) = timed(|| {
            summarize_pipeline(BufReader::new(std::fs::File::open(&path)?), threads, 10_000)
        });
        let (_, shared) = timed(|| summarize_shared(Arc::clone(&shared_text), threads));
        let ratio = |d: Duration| format!("×{:.2}", sequential.as_secs_f64() / d.as_secs_f64());
        println!(
            "{threads:>8} {:>12} {:>12} {:>12}",
            ratio(scoped),
            ratio(pipeline),
            ratio(shared)
        );
        threads *= 2;
    }
    Ok(())
}
