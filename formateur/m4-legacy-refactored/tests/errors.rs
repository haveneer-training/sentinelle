//! Étape 5 : des erreurs au lieu de paniques (D8). Un journal illisible ou en UTF-8
//! invalide donne un message et `ExitCode::FAILURE`, jamais une panique.

use sentinelle_gen::{Dataset, Options};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Un répertoire temporaire propre à chaque test, supprimé à la fin.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sentinelle-m4-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sentinelle-v0"))
        .arg(dir)
        .output()
        .expect("lancement de sentinelle-v0")
}

fn assert_clean_failure(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(!output.status.success(), "succès inattendu");
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        !stderr.contains("panicked"),
        "panique au lieu d'une erreur : {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "rapport partiel sur la sortie standard"
    );
    stderr
}

#[test]
fn invalid_utf8_is_reported_with_its_offset() {
    let data = Dataset::generate(&Options {
        seed: 2026,
        users: 40,
        noise: 0.3,
        malformed: 0.01,
        invalid_utf8: true,
        ..Options::default()
    });
    let offset = std::str::from_utf8(&data.auth)
        .expect_err("le jeu doit contenir de l'UTF-8 invalide")
        .valid_up_to();
    let dir = TempDir::new("utf8");
    std::fs::write(dir.0.join("auth.log"), &data.auth).unwrap();
    std::fs::write(dir.0.join("access.log"), &data.access).unwrap();

    let stderr = assert_clean_failure(&run(&dir.0));
    assert!(stderr.contains("auth.log"), "{stderr}");
    assert!(
        stderr.contains(&format!("UTF-8 invalide à l'octet {offset}")),
        "{stderr}"
    );
}

#[test]
fn missing_file_is_reported() {
    let dir = TempDir::new("missing");
    std::fs::write(dir.0.join("auth.log"), "").unwrap();

    let stderr = assert_clean_failure(&run(&dir.0));
    assert!(stderr.starts_with("erreur : "), "{stderr}");
    assert!(stderr.contains("access.log"), "{stderr}");
}

#[test]
fn empty_journals_give_an_empty_report() {
    let dir = TempDir::new("empty");
    std::fs::write(dir.0.join("auth.log"), "").unwrap();
    std::fs::write(dir.0.join("access.log"), "").unwrap();

    let output = run(&dir.0);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("auth.log   : 0 entrees (0 ignorees)"),
        "{stdout}"
    );
}
