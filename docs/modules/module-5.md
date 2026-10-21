# Module 5 — Performance : mesurer, comprendre, optimiser

Règle d'or : **on ne devine pas, on mesure**. Avant chaque optimisation, une mesure ;
après, la même mesure ; on ne garde que ce qui gagne.

## Préparer

```shell
cargo run --release -p sentinelle-gen -- --out data-100M --users 2000 --noise 10
cargo xtask goto 10 && cargo xtask solve 10     # si le fil rouge n'est pas terminé
```

## TP 1 — Mesurer et profiler (1 h 45 à 2 h)

### 1. `cargo bench` (30 min)

```shell
cargo bench -p sentinelle-perf                  # toutes les mesures
cargo bench -p sentinelle-perf --bench detect   # une seule
cargo bench -p sentinelle-perf -- parse/ligne   # filtrer par nom
```

Le harnais (`perf/src/lib.rs`, une centaine de lignes, sans dépendance) fait ce que fait
criterion en plus simple : échauffement, calibrage, échantillons, médiane. Lisez-le :
pourquoi `std::hint::black_box` ? pourquoi la médiane plutôt que la moyenne ?

Questions sur les résultats :

- `parse` : d'où vient l'écart de débit entre la v0.1 et le fil rouge ?
- `detect` : quand le bruit double, comment évoluent les deux durées ? Déduisez la
  complexité de chaque version. Que coûte la seule copie du journal ?
- `summary` : jusqu'où montent les versions parallèles, et que fait la version partagée ?

### 2. Écrire une mesure (30 min)

Ajoutez à `perf/benches/detect.rs` les mesures de `detect_bruteforce`,
`bytes_by_user_hour` et de la chronologie complète sur le jeu ×4. Laquelle domine
`analyze` ?

### 3. Profiler (45 min à 1 h)

Un profileur échantillonne la pile d'appels pendant l'exécution.

```shell
# Linux
perf record -g ./target/release/sentinelle-v0 data-100M && perf report
cargo flamegraph -p sentinelle-legacy -- data-100M      # cargo install flamegraph
# Linux et macOS
samply record ./target/release/sentinelle-v0 data-100M  # cargo install samply
```

Le profil `release` du workspace garde les symboles (`debug = true`) : les noms de
fonctions apparaissent. Où la v0.1 passe-t-elle son temps ? Et le fil rouge ?

### 4. La mémoire (20 min)

```shell
/usr/bin/time -v ./target/release/sentinelle-v0 data-100M   # Linux : « Maximum resident set size »
/usr/bin/time -l ./target/release/sentinelle-v0 data-100M   # macOS
```

Comparez le pic mémoire des deux versions au volume des journaux. Expliquez le rapport :
combien d'allocations par ligne pour la v0.1 ? pour le fil rouge ?

## TP 2 — Optimiser (1 h 30 à 2 h)

Choisissez **trois** pistes, dont O1. Pour chacune : mesure avant, modification,
`cargo test`, mesure après.

| # | Piste | Où |
|---|---|---|
| O1 | `parse_auth_event` alloue un `Vec<&str>` par ligne : un tableau de taille fixe suffit | `sentinelle/src/auth.rs` |
| O2 | la v0.1 : supprimer les copies du journal et la recherche linéaire (si ce n'est pas fait aux modules 2 et 4) | `legacy/` |
| O3 | profil de compilation : `lto = "fat"`, `codegen-units = 1` | `Cargo.toml` (racine) |
| O4 | le hachage : SipHash par défaut, ou un hachage plus rapide (FNV-1a, une vingtaine de lignes) via `HashMap::with_hasher` | `sentinelle/src/detect.rs` |
| O5 | la lecture : tout charger (`fs::read`) ou lire en flux (`BufRead`), quel pic mémoire ? | `sentinelle/src/main.rs` |

Pour O3, sans toucher au fichier :
`CARGO_PROFILE_BENCH_LTO=fat CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 cargo bench -p sentinelle-perf`.

Pour O4, une question de sécurité avant d'être une question de vitesse : les clés de la
table viennent des journaux, donc en partie de l'attaquant. Que risque-t-on avec un
hachage prévisible ?

## Criterion (facultatif)

`perf/criterion/` reprend la mesure du parseur avec criterion (intervalles de confiance,
comparaison avec la mesure précédente, rapport HTML) :

```shell
cargo bench --manifest-path perf/criterion/Cargo.toml
```

### Hors ligne

criterion dépend d'une vingtaine de crates. Sur un réseau isolé, préparer l'archive sur
une machine connectée :

```shell
cd perf/criterion && cargo vendor > vendor-config.toml
# copier perf/criterion (avec vendor/) sur le réseau isolé, puis :
mkdir -p .cargo && mv vendor-config.toml .cargo/config.toml
cargo bench --offline
```

## Diapositives

Outils pour les tests et l'analyse de performance : « Mesurer avant d'optimiser »,
« Options du compilateur et profils », « Criterion : comparer des variantes », « Le piège :
mesurer le compilateur, pas le code », « flamegraph », « samply », « Profiler la mémoire ».
Bonnes pratiques : « Les benchmarks : mesurer avant d'optimiser ». Conteneurs : « Choisir
sa fonction de hachage ». Représentation des types : « Coût des méthodes d'allocation ».
