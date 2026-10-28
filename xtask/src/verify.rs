//! Vérification des exercices courts (`cargo xtask verify`).

use crate::files::rust_files_in;
use std::fs;
use std::path::Path;

/// Modules dotés d'exercices courts : `exercices/<m>/examples` et `solutions/<m>`.
const SHORT_EXERCISES: [&str; 3] = ["m1", "m2", "m3"];

/// Ligne d'en-tête qui annonce l'erreur attendue d'un exercice qui ne doit pas compiler.
const EXPECTED_ERROR: &str = "Erreur attendue : ";

/// Vérifie les exercices courts : un exercice qui annonce une erreur doit échouer avec
/// elle, les autres (des `todo!()`) doivent compiler ; chaque corrigé doit compiler et
/// afficher « OK ». Les exercices sont des fichiers autonomes : `rustc` suffit.
pub fn verify(root: &Path) -> Result<(), String> {
    let scratch = std::env::temp_dir().join(format!("sentinelle-verify-{}", std::process::id()));
    fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let mut failures = Vec::new();
    for module in SHORT_EXERCISES {
        verify_module(root, module, &scratch, &mut failures)?;
    }
    let _ = fs::remove_dir_all(&scratch);
    if failures.is_empty() {
        println!("Exercices courts : tout est conforme.");
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn verify_module(
    root: &Path,
    module: &str,
    scratch: &Path,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    for path in rust_files_in(&root.join("exercices").join(module).join("examples")) {
        let name = format!("{module}/{}", file_stem(&path));
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let expected = text.lines().find_map(|l| {
            l.split_once(EXPECTED_ERROR)
                .map(|(_, e)| e.trim().to_string())
        });
        let output = rustc(&path, &scratch.join(file_stem(&path)))?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        match (expected, output.status.success()) {
            (Some(expected), true) => failures.push(format!(
                "{name} : compile, alors qu'il devrait échouer ({expected})"
            )),
            (Some(expected), false) if !stderr.contains(&expected) => failures.push(format!(
                "{name} : échoue, mais sans « {expected} »\n{stderr}"
            )),
            (Some(expected), false) => {
                println!("  exercice {name} : échoue comme prévu ({expected})")
            }
            (None, true) => println!("  exercice {name} : compile"),
            (None, false) => failures.push(format!("{name} : ne compile pas\n{stderr}")),
        }
    }

    let solutions = rust_files_in(&root.join("solutions").join(module));
    if solutions.is_empty() {
        return Err(format!(
            "aucun corrigé dans solutions/{module} : `verify` s'utilise dans le dépôt du corrigé"
        ));
    }
    for path in solutions {
        let name = format!("{module}/{}", file_stem(&path));
        let binary = scratch.join(format!("solution-{}", file_stem(&path)));
        let output = rustc(&path, &binary)?;
        if !output.status.success() {
            failures.push(format!(
                "corrigé {name} : ne compile pas\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
            continue;
        }
        let run = std::process::Command::new(&binary)
            .output()
            .map_err(|e| e.to_string())?;
        if run.status.success() && String::from_utf8_lossy(&run.stdout).contains("OK") {
            println!("  corrigé {name} : OK");
        } else {
            failures.push(format!(
                "corrigé {name} : échec à l'exécution\n{}",
                String::from_utf8_lossy(&run.stderr)
            ));
        }
    }
    Ok(())
}

/// Compile un fichier autonome avec l'édition du workspace.
fn rustc(source: &Path, output: &Path) -> Result<std::process::Output, String> {
    std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .args(["--edition", "2024", "--error-format=short", "-o"])
        .arg(output)
        .arg(source)
        .output()
        .map_err(|e| format!("rustc introuvable : {e}"))
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}
