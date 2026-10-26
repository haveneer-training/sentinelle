# Déroulé du fil rouge

Vous reconstruisez Sentinelle proprement, étape par étape : lecture des journaux sans
copie, détections reposant chacune sur la bonne collection, puis analyse répartie sur
plusieurs threads. À la fin, la chronologie de l'intrusion s'affiche d'elle-même.

À côté du fil rouge, `legacy/` contient la v0.1 héritée : on la lit, on la mesure et on
la refactorise dans les modules qui travaillent sur du code existant.

## Le déroulement d'une étape

Le squelette est là, les tests aussi : ce qui manque, ce sont **21 `todo!()`**, répartis
sur onze étapes (0 à 10, la dernière en bonus). L'énoncé de chaque étape est dans
[`docs/etapes/`](docs/etapes/) : commencez toujours par le lire.

### La boucle

```shell
cargo xtask next     # ouvrir l'étape suivante (et compléter les précédentes restées vides)
cargo test           # rouge : voici ce qu'il faut écrire, fichier et ligne à l'appui
#   ... remplacer le todo!() entre les marqueurs >>> et <<< ...
cargo test           # vert : l'étape est finie
```

`cargo test` ne montre que les étapes déjà ouvertes : jamais quarante tests rouges d'un
coup, seulement ceux qui vous concernent.

### Les commandes

| Commande | Effet |
|---|---|
| `cargo xtask status` | avancement étape par étape |
| `cargo xtask next` | passer à l'étape suivante, sans chercher son numéro (comme `goto` avec l'étape courante + 1) |
| `cargo xtask goto <n>` | passer à l'étape n ; remplit au passage les trous des étapes précédentes **restés vides** |
| `cargo xtask solve <n>` | remplir les trous de l'étape n à votre place (rattrapage) |
| `cargo xtask reset <n>` | rouvrir les trous de l'étape n pour la refaire |
| `cargo test` | vérifier |
| `cargo run --release -p sentinelle -- data` | lancer l'analyse (complète à partir de l'étape 7) |

`goto` et `next` **n'écrasent jamais** ce que vous avez écrit : ils ne remplissent que les
blocs encore occupés par un `todo!()`. Si vous décrochez, `next` et vous repartez d'un
code cohérent.

### Où écrire

```rust
pub fn parse_timestamp(text: &str) -> Option<Timestamp> {
    // À FAIRE (étape 0) Vérifier la forme `AAAA-MM-JJTHH:MM:SSZ` ...
    // >>> ÉTAPE 0 — à compléter
    todo!("étape 0 — voir le commentaire ci-dessus")
    // <<< ÉTAPE 0
}
```

Écrivez entre les deux marqueurs `>>>` et `<<<`, et laissez-les en place : c'est ainsi
que `goto`, `solve` et `reset` s'y retrouvent. Le reste du fichier vous appartient.

Vous pouvez versionner votre travail (`git init && git add -A && git commit`) pour
retrouver vos états successifs.

### Les avertissements

Un `todo!()` rend inutilisés les paramètres de sa fonction et certains imports. Ceux des
étapes pas encore ouvertes sont tus. Ceux qui restent disent quelque chose d'utile :
« import `VecDeque` inutilisé » pendant l'étape 5, c'est un indice.

## Les étapes

| # | Étape | Module | Ce que vous écrivez | Ce que vous lisez |
|---|---|---|---|---|
| [0](docs/etapes/etape-00.md) | Lire un horodatage | Références | `&str`, `as_bytes`, tranches, `Option` et `?` | doc-test, `Display` |
| [1](docs/etapes/etape-01.md) | Analyser `auth.log` sans copier | Références | `split_once`, motifs de tranche, `Result`, `AuthEntry<'_>` | `enum` à données empruntées |
| [2](docs/etapes/etape-02.md) | Analyser `access.log` | Références | découpage successif, `Option<u64>`, erreurs précises | |
| [3](docs/etapes/etape-03.md) | Un journal entier | Références | `parse_lines<'a, T>`, `from_utf8`, `impl Fn` | octets contre texte |
| [4](docs/etapes/etape-04.md) | Compter et classer | Collections | `HashMap` + API `entry`, `BinaryHeap` + `Reverse` | |
| [5](docs/etapes/etape-05.md) | Spraying et force brute | Collections | `HashSet`, fenêtre glissante `VecDeque` | |
| [6](docs/etapes/etape-06.md) | Compromission et habitudes | Collections | corrélation, `HashSet<(&str, &str)>`, `let … else` | structures à durée de vie |
| [7](docs/etapes/etape-07.md) | Volumes et chronologie | Collections | `BTreeMap`, `range`, tri stable | `analyze`, l'enquête complète |
| [8](docs/etapes/etape-08.md) | Threads qui empruntent | Concurrence | `split_at`, `thread::scope`, fusion de résumés | `Summary<U>` générique |
| [9](docs/etapes/etape-09.md) | Pipeline de threads | Concurrence | `mpsc::sync_channel`, `Mutex<Receiver>`, fermeture des canaux | `BufRead` |
| [10](docs/etapes/etape-10.md) | *Bonus* : partager plutôt que fusionner | Concurrence | `thread::spawn`, `Arc<Mutex<_>>`, atomiques | mesures d'échelle |

Les étapes s'arrêtent au module 3, et c'est voulu : elles **ajoutent** du code (lecture,
détections, threads). Les modules 4 à 6 travaillent sur du code **qui existe déjà** :
on refactorise et documente `legacy/` (4), on mesure et optimise la v0.1 et le fil rouge
terminé (5), on débogue `sentinelle-turbo` (6). Le fil rouge doit donc être complet à la
fin du module 3 ; sinon, `cargo xtask goto 10 && cargo xtask solve 10` le termine.

## Autour du fil rouge

Chaque module a sa fiche dans [`docs/modules/`](docs/modules/) : ce qui se fait en cours,
en TP, et avec quel support. Pour les modules 4 à 6, la fiche est l'énoncé du TP.

| Module | Hors fil rouge (modules 1 à 3) ou TP (modules 4 à 6) |
|---|---|
| 1. Références | `exercices/m1` : neuf erreurs classiques du vérificateur d'emprunts ; lecture critique de `legacy/` |
| 2. Collections | `exercices/m2` : manipuler un vecteur ; optimiser les collections de `legacy/` |
| 3. Concurrence | `exercices/m3` : threads, `Mutex`, canaux |
| 4. Bonnes pratiques | refactoriser `legacy/` sous test, documenter avec rustdoc |
| 5. Performance | `perf/` : `cargo bench`, profileurs, optimisations |
| 6. Débogage | `bugs/` : les tickets de `sentinelle-turbo` |
