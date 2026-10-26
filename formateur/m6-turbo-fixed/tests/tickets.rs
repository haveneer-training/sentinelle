//! Les tickets du module 6, une fois corrigés, sur les jeux de `docs/modules/module-6.md`.
//! Les symptômes de la version boguée, eux, sont vérifiés par `scripts/check-bugs.sh`.

use sentinelle_gen::{Dataset, Options};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Au-delà, la commande est considérée comme bloquée (tickets 4 et 5).
const TIMEOUT: Duration = Duration::from_secs(60);

/// Un jeu écrit une fois pour tous les tests, avec sa vérité terrain.
struct Data {
    dir: PathBuf,
    brute_ip: String,
}

fn write(name: &str, opts: &Options) -> Data {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let data = Dataset::generate(opts);
    std::fs::write(dir.join("auth.log"), &data.auth).unwrap();
    std::fs::write(dir.join("access.log"), &data.access).unwrap();
    Data {
        dir,
        brute_ip: data.generated.scenario.brute_ip.to_string(),
    }
}

/// `sentinelle-gen --out data`
fn data() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| write("data", &Options::default()))
}

/// `sentinelle-gen --out data-propre --malformed 0`
fn data_propre() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| {
        let opts = Options {
            malformed: 0.0,
            ..Options::default()
        };
        write("data-propre", &opts)
    })
}

/// `sentinelle-gen --out data-abime --invalid-utf8 --malformed 0.01`
fn data_abime() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| {
        let opts = Options {
            invalid_utf8: true,
            malformed: 0.01,
            ..Options::default()
        };
        write("data-abime", &opts)
    })
}

/// Lance `sentinelle-turbo`, en échouant s'il ne rend pas la main avant [`TIMEOUT`].
fn turbo(args: &[&str], dir: &Path) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sentinelle-turbo"))
        .arg(args[0])
        .arg(dir)
        .args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("lancement de sentinelle-turbo");
    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > TIMEOUT {
            child.kill().ok();
            panic!("{args:?} bloqué depuis plus de {TIMEOUT:?}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let Output {
        status,
        stdout,
        stderr,
    } = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&stderr);
    assert!(status.success(), "{args:?} : {stderr}");
    String::from_utf8(stdout).expect("sortie en UTF-8")
}

/// (adresse, échecs) des lignes de `stats` ou de `stream`.
fn failures(output: &str) -> Vec<(String, usize)> {
    output
        .lines()
        .map(|line| {
            let mut words = line.split_whitespace();
            let ip = words.next().unwrap().to_string();
            (ip, words.next().unwrap().parse().unwrap())
        })
        .collect()
}

#[test]
fn ticket_1_large_volumes_raise_an_alert() {
    // le binaire de test est compilé en debug : un débordement paniquerait
    let out = turbo(&["report"], &data_propre().dir);
    let alerts: Vec<u64> = out
        .lines()
        .filter(|l| l.starts_with("ALERTE "))
        .map(|l| l.split_whitespace().nth(4).unwrap().parse().unwrap())
        .collect();
    assert!(!alerts.is_empty(), "aucune alerte :\n{out}");
    assert!(alerts.iter().all(|&b| b >= 2_000_000_000), "{alerts:?}");
    assert!(
        alerts.iter().any(|&b| b > u64::from(u32::MAX)),
        "volume tronqué : {alerts:?}"
    );
    assert!(
        out.ends_with(&format!("{} alerte(s)\n", alerts.len())),
        "{out}"
    );
}

#[test]
fn ticket_2_truncated_lines_are_skipped() {
    let out = turbo(&["report"], &data().dir);
    assert!(out.contains("alerte(s)"), "{out}");
}

#[test]
fn ticket_3_invalid_utf8_lines_are_skipped() {
    let dir = &data_abime().dir;
    let bytes = std::fs::read(dir.join("auth.log")).unwrap();
    assert!(
        std::str::from_utf8(&bytes).is_err(),
        "le jeu doit contenir de l'UTF-8 invalide"
    );
    turbo(&["report"], dir);
    assert_eq!(failures(&turbo(&["stats"], dir)).len(), 5);
}

#[test]
fn ticket_4_stats_terminates_and_ignores_thread_count() {
    let dir = &data().dir;
    let one = turbo(&["stats", "--threads", "1"], dir);
    for threads in ["2", "8", "32"] {
        assert_eq!(
            turbo(&["stats", "--threads", threads], dir),
            one,
            "--threads {threads}"
        );
    }
}

#[test]
fn ticket_5_stream_terminates_and_agrees_with_stats() {
    let dir = &data().dir;
    let stats = failures(&turbo(&["stats"], dir));
    for workers in ["1", "4", "16"] {
        let stream = failures(&turbo(&["stream", "--workers", workers], dir));
        assert_eq!(stream, stats, "--workers {workers}");
    }
}

#[test]
fn ticket_bonus_bruteforce_ip_is_reported() {
    let data = data();
    let out = turbo(&["stats", "--threads", "1"], &data.dir);
    assert!(
        failures(&out).iter().any(|(ip, _)| *ip == data.brute_ip),
        "{} absente :\n{out}",
        data.brute_ip
    );
}
