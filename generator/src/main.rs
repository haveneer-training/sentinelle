//! `sentinelle-gen` : écrit `auth.log` et `access.log` dans un répertoire.

use sentinelle_gen::{Options, generate, scenario};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "\
Usage : sentinelle-gen [OPTIONS]

  --out DIR         répertoire de sortie                      (défaut : data)
  --seed N          graine du générateur                      (défaut : 42)
  --users N         nombre de salariés                        (défaut : 150)
  --days N          nombre de jours, à partir du lundi 9 novembre 2026 (défaut : 7, min : 5)
  --noise F         intensité du bruit de fond d'Internet     (défaut : 1.0)
  --malformed P     probabilité qu'une ligne soit corrompue   (défaut : 0.0005)
  --invalid-utf8    les lignes corrompues peuvent contenir de l'UTF-8 invalide
  --injection       bonus : noms de compte piégés (docs/bonus/injection.md)
  --distributed     bonus : force brute distribuée (docs/bonus/distributed.md)
  --reveal          écrit aussi GROUND_TRUTH.md (la solution : à lire après l'enquête)

Ordres de grandeur sur 7 jours : 150 salariés ≈ 9 Mo ; --users 2000 --noise 10 ≈ 100 Mo.";

struct Args {
    out: PathBuf,
    reveal: bool,
    options: Options,
}

fn parse_args() -> Result<Args, String> {
    let mut parsed = Args {
        out: PathBuf::from("data"),
        reveal: false,
        options: Options::default(),
    };
    let opts = &mut parsed.options;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("valeur manquante pour {arg}"));
        let bad = |e: &dyn std::fmt::Display| format!("valeur invalide pour {arg} : {e}");
        match arg.as_str() {
            "--out" => parsed.out = PathBuf::from(value()?),
            "--seed" => opts.seed = value()?.parse().map_err(|e| bad(&e))?,
            "--users" => opts.users = value()?.parse().map_err(|e| bad(&e))?,
            "--days" => opts.days = value()?.parse().map_err(|e| bad(&e))?,
            "--noise" => opts.noise = value()?.parse().map_err(|e| bad(&e))?,
            "--malformed" => opts.malformed = value()?.parse().map_err(|e| bad(&e))?,
            "--invalid-utf8" => opts.invalid_utf8 = true,
            "--injection" => opts.injection = true,
            "--distributed" => opts.distributed = true,
            "--reveal" => parsed.reveal = true,
            "-h" | "--help" => return Err(USAGE.to_string()),
            _ => return Err(format!("option inconnue : {arg}\n\n{USAGE}")),
        }
    }
    if opts.days < scenario::MIN_DAYS {
        return Err(format!(
            "--days doit valoir au moins {}",
            scenario::MIN_DAYS
        ));
    }
    Ok(parsed)
}

fn run(args: &Args) -> std::io::Result<()> {
    let started = Instant::now();
    let out = &args.out;
    fs::create_dir_all(out)?;
    let create =
        |name: &str| File::create(out.join(name)).map(|f| BufWriter::with_capacity(1 << 20, f));
    let generated = generate(&args.options, create("auth.log")?, create("access.log")?)?;

    if args.reveal {
        let mut report = generated
            .scenario
            .report(&generated.world, args.options.seed);
        report.push_str(&format!(
            "\nLignes corrompues : {} / {} dans auth.log, {} / {} dans access.log. Certaines \
             restent lisibles (un champ final disparu, par exemple) : un analyseur en rejette \
             au plus autant.\n",
            generated.auth.corrupted,
            generated.auth.lines,
            generated.access.corrupted,
            generated.access.lines
        ));
        if let Some(injection) = &generated.injection {
            report.push_str(&injection.report());
        }
        fs::write(out.join("GROUND_TRUTH.md"), report)?;
    }
    eprintln!(
        "auth.log : {} lignes ({:.1} Mo), access.log : {} lignes ({:.1} Mo) — {:.1?}",
        generated.auth.lines,
        megabytes(&out.join("auth.log")),
        generated.access.lines,
        megabytes(&out.join("access.log")),
        started.elapsed()
    );
    Ok(())
}

fn megabytes(path: &Path) -> f64 {
    fs::metadata(path)
        .map(|m| m.len() as f64 / 1e6)
        .unwrap_or(0.0)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = run(&args) {
        eprintln!("erreur d'écriture dans {} : {e}", args.out.display());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
