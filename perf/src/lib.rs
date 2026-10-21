//! Un harnais de mesure minimal, sans dépendance.
//!
//! Il fait ce que fait criterion, en moins bien mais en lisible : échauffement,
//! calibrage du nombre d'itérations, échantillons, médiane. `std::hint::black_box`
//! empêche le compilateur d'éliminer un calcul dont le résultat ne servirait à rien.
//!
//! ```no_run
//! use sentinelle_perf::Bench;
//!
//! let mut bench = Bench::from_args();
//! bench.run("somme", Some(8_000), || (0..1_000u64).sum::<u64>());
//! ```

pub mod legacy;

use std::hint::black_box;
use std::time::{Duration, Instant};

/// Durée visée pour un échantillon : assez longue pour que l'horloge soit précise.
const SAMPLE_TARGET: Duration = Duration::from_millis(10);
const SAMPLES: usize = 25;
/// Pour une mesure lente (plus de 100 ms par appel), moins d'échantillons suffisent.
const SLOW_SAMPLES: usize = 5;

/// Une série de mesures, filtrée par les arguments de la ligne de commande
/// (`cargo bench -p sentinelle-perf -- parse` ne lance que les mesures dont le nom
/// contient « parse »).
pub struct Bench {
    filters: Vec<String>,
}

/// Le résultat d'une mesure.
#[derive(Debug, Clone, Copy)]
pub struct Measure {
    pub median: Duration,
    pub min: Duration,
}

impl Bench {
    pub fn from_args() -> Self {
        // cargo passe `--bench` aux harnais personnalisés : on ignore les options
        let filters = std::env::args()
            .skip(1)
            .filter(|a| !a.starts_with("--"))
            .collect();
        println!(
            "{:<44} {:>12} {:>12} {:>12}",
            "mesure", "médiane", "min", "débit"
        );
        Self { filters }
    }

    /// Mesure `f`. `bytes` (facultatif) : volume traité par appel, pour afficher un débit.
    pub fn run<T>(
        &mut self,
        name: &str,
        bytes: Option<u64>,
        mut f: impl FnMut() -> T,
    ) -> Option<Measure> {
        if !self.filters.is_empty()
            && !self
                .filters
                .iter()
                .any(|filter| name.contains(filter.as_str()))
        {
            return None;
        }
        // échauffement et calibrage : doubler jusqu'à atteindre la durée visée
        let mut iterations: u32 = 1;
        let calibration = loop {
            let started = Instant::now();
            for _ in 0..iterations {
                black_box(f());
            }
            let elapsed = started.elapsed();
            if elapsed >= SAMPLE_TARGET || iterations >= 1 << 24 {
                break elapsed / iterations;
            }
            iterations *= 2;
        };
        let count = if calibration > Duration::from_millis(100) {
            SLOW_SAMPLES
        } else {
            SAMPLES
        };
        let mut samples: Vec<Duration> = (0..count)
            .map(|_| {
                let started = Instant::now();
                for _ in 0..iterations {
                    black_box(f());
                }
                started.elapsed() / iterations
            })
            .collect();
        samples.sort_unstable();
        let measure = Measure {
            median: samples[count / 2],
            min: samples[0],
        };
        let throughput = bytes.map_or(String::new(), |b| {
            format!("{:.0} Mo/s", b as f64 / measure.median.as_secs_f64() / 1e6)
        });
        println!(
            "{name:<44} {:>12.2?} {:>12.2?} {throughput:>12}",
            measure.median, measure.min
        );
        Some(measure)
    }
}
