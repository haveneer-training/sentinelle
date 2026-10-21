//! Générateur de journaux synthétiques pour le fil rouge Sentinelle.
//!
//! Produit `auth.log` et `access.log` d'une entreprise fictive dans lesquels se cache un
//! scénario d'intrusion. Le binaire `sentinelle-gen` écrit des fichiers ; les tests de
//! Sentinelle utilisent [`Dataset::generate`] pour disposer en mémoire des journaux et de
//! la vérité terrain correspondante.

pub mod clock;
pub mod events;
pub mod rng;
pub mod scenario;
pub mod world;

use clock::DAY;
use events::{Batch, Line};
use rng::Rng;
use scenario::Scenario;
use std::io::{self, Write};
use world::World;

pub struct Options {
    pub seed: u64,
    pub users: usize,
    pub days: i64,
    pub noise: f64,
    pub malformed: f64,
    pub invalid_utf8: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { seed: 42, users: 150, days: 7, noise: 1.0, malformed: 0.0005, invalid_utf8: false }
    }
}

/// Ce que le générateur a produit, en plus des lignes elles-mêmes.
pub struct Generated {
    pub world: World,
    pub scenario: Scenario,
    pub auth: Counts,
    pub access: Counts,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Counts {
    pub lines: u64,
    pub corrupted: u64,
}

/// Écrit les deux journaux dans `auth` et `access`.
pub fn generate(opts: &Options, auth: impl Write, access: impl Write) -> io::Result<Generated> {
    assert!(opts.days >= scenario::MIN_DAYS, "il faut au moins {} jours", scenario::MIN_DAYS);
    let mut rng = Rng::new(opts.seed);
    let world = World::new(&mut rng.fork(), opts.users, opts.days);
    let (scenario, attack) = scenario::play(&world, &mut rng.fork());
    let excluded = [scenario.spray_ip, scenario.brute_ip];

    // répartit les lignes du scénario par jour
    let mut attack_days: Vec<Batch> = (0..opts.days).map(|_| Batch::default()).collect();
    let day_of = |ts: i64| ((ts - world.start).div_euclid(DAY)).clamp(0, opts.days - 1) as usize;
    for line in attack.auth {
        attack_days[day_of(line.ts)].auth.push(line);
    }
    for line in attack.access {
        attack_days[day_of(line.ts)].access.push(line);
    }

    let mut auth = Sink::new(auth, rng.fork(), opts);
    let mut access = Sink::new(access, rng.fork(), opts);
    let mut activity = rng.fork();
    for (day, attack) in (0..opts.days).zip(attack_days) {
        let mut batch = Batch::default();
        events::normal_day(&mut batch, &world, &mut activity, day);
        events::cron_day(&mut batch, &world, &mut activity, day);
        events::noise_day(&mut batch, &world, &mut activity, day, opts.noise, &excluded);
        batch.append(attack);
        auth.write(batch.auth)?;
        access.write(batch.access)?;
    }
    auth.out.flush()?;
    access.out.flush()?;
    Ok(Generated { world, scenario, auth: auth.counts, access: access.counts })
}

/// Journaux engendrés en mémoire, avec leur vérité terrain.
pub struct Dataset {
    pub auth: Vec<u8>,
    pub access: Vec<u8>,
    pub generated: Generated,
}

impl Dataset {
    pub fn generate(opts: &Options) -> Self {
        let (mut auth, mut access) = (Vec::new(), Vec::new());
        let generated = generate(opts, &mut auth, &mut access).expect("écriture en mémoire");
        Self { auth, access, generated }
    }

    /// Petit jeu de données, rapide à engendrer et à analyser dans un test.
    pub fn small(seed: u64) -> Self {
        Self::generate(&Options { seed, users: 40, noise: 0.3, ..Options::default() })
    }
}

/// Écrit les lignes triées par instant réel, en corrompant certaines d'entre elles.
struct Sink<W: Write> {
    out: W,
    rng: Rng,
    malformed: f64,
    invalid_utf8: bool,
    counts: Counts,
}

impl<W: Write> Sink<W> {
    fn new(out: W, rng: Rng, opts: &Options) -> Self {
        Self { out, rng, malformed: opts.malformed, invalid_utf8: opts.invalid_utf8, counts: Counts::default() }
    }

    fn write(&mut self, mut lines: Vec<Line>) -> io::Result<()> {
        lines.sort_by_key(|l| l.ts);
        for line in lines {
            self.counts.lines += 1;
            let mut bytes = line.text.into_bytes();
            if !line.protected && self.rng.chance(self.malformed) {
                self.counts.corrupted += 1;
                self.corrupt(&mut bytes);
            }
            bytes.push(b'\n');
            self.out.write_all(&bytes)?;
        }
        Ok(())
    }

    fn corrupt(&mut self, bytes: &mut Vec<u8>) {
        let rng = &mut self.rng;
        match rng.range(0, if self.invalid_utf8 { 4 } else { 3 }) {
            // ligne tronquée (écriture interrompue)
            0 => bytes.truncate(rng.index(bytes.len())),
            // un champ disparaît
            1 => {
                let spaces: Vec<usize> = (0..bytes.len()).filter(|&i| bytes[i] == b' ').collect();
                if spaces.len() >= 2 {
                    let i = rng.index(spaces.len() - 1);
                    bytes.drain(spaces[i]..spaces[i + 1]);
                }
            }
            // ligne vide
            2 => bytes.clear(),
            // octet parasite : UTF-8 invalide
            _ => {
                let at = rng.index(bytes.len());
                bytes.insert(at, *rng.pick(&[0xFF, 0xC3, 0x80, 0xFE]));
            }
        }
    }
}
