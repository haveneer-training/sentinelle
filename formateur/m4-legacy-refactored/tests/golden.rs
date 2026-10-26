//! Test de caractérisation : la sortie de `sentinelle-v0` sur un jeu de données fixe.
//!
//! Il ne dit pas que la sortie est **juste** (elle ne l'est pas !), seulement qu'elle n'a
//! pas changé. C'est le filet de sécurité d'une refactorisation : chaque transformation
//! doit le laisser vert. Quand vous corrigez volontairement un comportement, régénérez la
//! référence et relisez la différence :
//!
//! ```shell
//! UPDATE_GOLDEN=1 cargo test -p sentinelle-legacy
//! git diff legacy/tests/golden/expected.txt
//! ```
//!
//! Ici (corrigé formateur), la feature `fix-d3` compare à une seconde référence :
//!
//! ```shell
//! UPDATE_GOLDEN=1 cargo test --features fix-d3 --test golden
//! ```

use sentinelle_gen::Dataset;
use std::path::PathBuf;
use std::process::Command;

/// Sans correction, la référence est celle de la v0.1 elle-même.
#[cfg(not(feature = "fix-d3"))]
const GOLDEN: &str = "../../legacy/tests/golden/expected.txt";
/// Après la correction de D3 (étape 7), une référence régénérée et relue.
#[cfg(feature = "fix-d3")]
const GOLDEN: &str = "tests/golden/expected-fix-d3.txt";

#[test]
fn output_is_unchanged() {
    let dir = std::env::temp_dir().join(format!("sentinelle-golden-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let data = Dataset::small(2026);
    std::fs::write(dir.join("auth.log"), &data.auth).unwrap();
    std::fs::write(dir.join("access.log"), &data.access).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_sentinelle-v0"))
        .arg(&dir)
        .output()
        .expect("lancement de sentinelle-v0");
    std::fs::remove_dir_all(&dir).ok();
    assert!(
        output.status.success(),
        "échec : {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = String::from_utf8(output.stdout).expect("sortie en UTF-8");

    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &actual).unwrap();
    }
    let expected =
        std::fs::read_to_string(&golden).expect("référence absente : lancer avec UPDATE_GOLDEN=1");
    assert_eq!(actual, expected, "la sortie a changé");
}
