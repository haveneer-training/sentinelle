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

## Diapositives

« Et avec spawn ? Arc, Send et Sync », « Ce que coûte la synchronisation », « Autres
mécanismes : les types atomiques », « RwLock : plusieurs lecteurs ou un seul écrivain ».
