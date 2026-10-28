# Étape 10 (bonus) — Partager plutôt que fusionner

> Module 3 (Concurrence) · 50 à 70 min, plus 20 à 30 min de mesures · `sentinelle/src/summary.rs`

## L'enquête

Jusqu'ici, chaque thread a son propre résumé. L'idée qui vient souvent en premier est
l'inverse : un seul résumé, partagé, protégé par un verrou. Écrivez-la, puis mesurez.

## Ce que vous écrivez

1. `summarize_shared(text: Arc<Vec<u8>>, threads) -> Summary<String>` avec
   `thread::spawn` (pas de `scope`) et un `Arc<Mutex<Summary<String>>>`.
2. `summarize_with_progress(text, threads, period, report)` : comme `summarize_scoped`,
   plus un thread qui appelle `report(lignes_traitées)` à intervalles réguliers.

## Indices

- `thread::spawn` exige une fermeture `'static` : elle ne peut rien emprunter. `Arc::clone`
  donne à chaque thread sa propre poignée sur les mêmes données. `chunk_ranges` (fournie)
  donne des plages d'octets, qui ne sont pas des emprunts.
- Après les `join()`, plus personne d'autre ne détient l'`Arc` : `Arc::try_unwrap`, puis
  `Mutex::into_inner`.
- `AtomicUsize::fetch_add` incrémente sans verrou ; `AtomicBool` sert de drapeau d'arrêt.
  Pour que le **dernier** rapport soit le total, attendez la fin du rapporteur avant de
  l'émettre.

## Vérifier, puis mesurer

```shell
cargo test --test step10_shared_state
cargo run --release -p sentinelle-gen -- --out data-800M --users 20000 --noise 50
cargo run --release -p sentinelle --example scaling -- data-800M
```

Expliquez la colonne « partagé ». Jusqu'où montent les deux autres, et qu'est-ce qui les
limite (loi d'Amdahl : la lecture, la fusion) ?

## Extensions

- Une liste noire d'adresses partagée en `RwLock` : les travailleurs la consultent à
  chaque ligne, l'agrégateur l'enrichit. Pourquoi `RwLock` plutôt que `Mutex` ici ?
- Dans `main`, analysez `auth.log` et `access.log` dans deux threads.

## Pour aller plus loin : paralléliser toute la Sentinelle

Les étapes 8 à 10 ne parallélisent que le résumé, un calcul à côté de l'analyse. Sur un
jeu de 800 Mo (`--users 20000 --noise 50`, 10 cœurs), le temps de `sentinelle` se répartit
à peu près ainsi : analyse syntaxique 50 % (surtout `access.log`), `bytes_by_user_hour`
45 %, détections 5 %, lecture des fichiers quelques pour cent. Le faire complètement
demanderait de traiter, ensemble :

- **L'analyse syntaxique** : chaque ligne est indépendante. `split_chunks`, un thread par
  morceau, puis concaténation des `Vec` **dans l'ordre des morceaux**, car les détections
  supposent les entrées dans l'ordre du journal. Les numéros de ligne des erreurs sont à
  décaler de ceux des morceaux précédents.
- **Les agrégats** (`bytes_by_user_hour`, `failures_by_ip`) : un agrégat partiel par
  morceau puis une fusion, comme `Summary::merge`. Ici, une `HashMap` cumulée puis
  convertie une seule fois en `BTreeMap` gagnerait déjà beaucoup, sans aucun thread :
  mesurer avant de paralléliser.
- **Les détections** : indépendantes les unes des autres, donc une par thread. À
  l'intérieur d'une détection, les fenêtres glissantes suivent le temps ; on ne peut les
  répartir que par clé (adresse, compte), chaque thread voyant toutes les entrées de ses
  clés, dans l'ordre. Peu de gain pour beaucoup de complexité.
- **La cohérence** : la sortie doit rester identique à la version séquentielle, quel que
  soit le nombre de threads. Un test qui compare les deux sur des jeux générés (comme
  `parallel_summary_equals_sequential` à l'étape 8), et un ordre de sortie qui ne dépend
  ni de l'ordre d'arrivée des threads ni de celui d'une `HashMap`.
- **La mémoire** : les entrées empruntent le texte, d'où `thread::scope` et le texte
  entier en mémoire. Un pipeline (étape 9) imposerait des entrées possédées (`String`),
  donc des allocations par ligne, à rebours du module 5.
- **Ce qui reste séquentiel** (loi d'Amdahl) : la lecture, les concaténations, les
  fusions, le tri de la chronologie. Ordre de grandeur attendu : de 6 s à 1,5 s environ.

C'est le travail que fait la crate `rayon` (`par_chunks`, `par_iter().fold(..).reduce(..)`) ;
ce dépôt s'en passe pour rester sans dépendance externe.

## Diapositives

« Et avec spawn ? Arc, Send et Sync », « Ce que coûte la synchronisation », « RwLock :
plusieurs lecteurs ou un seul écrivain ».

Les atomiques ne sont pas au programme du cours : pour ce bonus, la documentation de
[`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/) suffit (`AtomicUsize::fetch_add`,
`AtomicBool`, `Ordering`).
