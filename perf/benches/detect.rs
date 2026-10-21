//! Comptage des échecs : recherche linéaire (v0.1) contre table de hachage, à volume
//! croissant. Le rapport des durées entre deux tailles dit la complexité.
//!
//! `cargo bench -p sentinelle-perf --bench detect`

use sentinelle::detect::{Config, analyze, failures_by_ip};
use sentinelle::journal::{parse_auth, parse_web};
use sentinelle_gen::{Dataset, Options};
use sentinelle_perf::{Bench, legacy};

fn main() {
    let mut bench = Bench::from_args();
    for noise in [1.0, 2.0, 4.0] {
        let data = Dataset::generate(&Options {
            noise,
            ..Options::default()
        });
        let text = std::str::from_utf8(&data.auth).unwrap_or_default();
        let old = legacy::parse_all(text);
        let new = parse_auth(&data.auth);
        let size = format!("bruit ×{noise}");
        bench.run(&format!("detect/échecs/v0.1/{size}"), None, || {
            legacy::count_failures_by_ip(old.clone()).len()
        });
        bench.run(
            &format!("detect/copie-du-journal/v0.1/{size}"),
            None,
            || old.clone().len(),
        );
        bench.run(&format!("detect/échecs/sentinelle/{size}"), None, || {
            failures_by_ip(&new.entries).len()
        });
    }

    let data = Dataset::generate(&Options::default());
    let auth = parse_auth(&data.auth);
    let web = parse_web(&data.access);
    let config = Config::starting_at(auth.entries[0].timestamp);
    bench.run("detect/analyse-complète/sentinelle", None, || {
        analyze(&auth.entries, &web.entries, &config).len()
    });
}
