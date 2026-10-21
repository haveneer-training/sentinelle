//! Résumés parallèles (étapes 8 à 10) selon le nombre de threads.
//!
//! `cargo bench -p sentinelle-perf --bench summary`

use sentinelle::summary::{summarize, summarize_pipeline, summarize_scoped, summarize_shared};
use sentinelle_gen::{Dataset, Options};
use sentinelle_perf::Bench;
use std::sync::Arc;

fn main() {
    let data = Dataset::generate(&Options { users: 600, noise: 4.0, ..Options::default() });
    let bytes = Some(data.auth.len() as u64);
    let shared = Arc::new(data.auth.clone());
    let mut bench = Bench::from_args();
    bench.run("summary/séquentiel", bytes, || summarize(&data.auth).lines);
    for threads in [2, 4, 8] {
        bench.run(&format!("summary/scope/{threads}"), bytes, || summarize_scoped(&data.auth, threads).lines);
        bench.run(&format!("summary/pipeline/{threads}"), bytes, || {
            summarize_pipeline(data.auth.as_slice(), threads, 10_000).map(|s| s.lines).unwrap_or(0)
        });
        bench.run(&format!("summary/partagé/{threads}"), bytes, || summarize_shared(Arc::clone(&shared), threads).lines);
    }
}
