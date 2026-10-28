# Étape 8 — Des threads qui empruntent

> Module 3 (Concurrence) · 45 à 60 min · `sentinelle/src/summary.rs`

## L'enquête

Le jeu « officiel » fait 800 Mo. L'analyse des échecs se découpe : chaque morceau du
fichier peut être résumé indépendamment (`Summary`), puis les résumés se fusionnent.

## Ce que vous écrivez

1. `Summary::merge(&mut self, other: Self)` : additionner les compteurs, réunir les
   ensembles de comptes.
2. `split_chunks(text, parts) -> Vec<&[u8]>` : au plus `parts` morceaux de tailles voisines,
   chacun finissant par une fin de ligne. Aucun octet perdu ni dupliqué.
3. `summarize_scoped(text, threads)` : un thread par morceau, puis fusion.

## Indices

- `other` est pris par valeur : ses `HashSet` peuvent être déplacés (`extend`), pas copiés.
- `slice.split_at(i)` rend deux tranches empruntées ; `iter().position(|&b| b == b'\n')`
  trouve la fin de ligne suivante.
- `thread::scope(|scope| { scope.spawn(move || …) … })` : tous les threads se terminent
  avant la fin du bloc, ils peuvent donc **emprunter** `text`. Le résumé obtenu,
  `Summary<&str>`, emprunte toujours le texte.
- `join()` renvoie un `Result` : `Err` si le thread a paniqué.

## Vérifier

```shell
cargo test --test step8_scoped_threads
cargo run --release -p sentinelle -- data --threads 4
```

`--threads` ne fait que **refaire** le résumé, en parallèle, une fois l'analyse terminée :
le temps total ne baisse pas, il augmente un peu. Comparez la ligne `résumé sur N threads`
(sur la sortie d'erreur) pour plusieurs valeurs de N. Paralléliser le reste de l'analyse :
voir « Pour aller plus loin » à l'[étape 10](etape-10.md).

## Diapositives

« spawn exige move, scope autorise l'emprunt », « Lancer un thread : spawn et join »,
« Concurrence et Parallélisme : deux notions distinctes » (Le multi-threading sans peur).
