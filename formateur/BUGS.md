# Module 6 — les bugs de `sentinelle-turbo` (formateur)

Reproductions vérifiées avec les jeux par défaut (graine 42).

| Ticket | Commande | Symptôme | Cause | Correction |
|---|---|---|---|---|
| 1 | `report data-propre` (`--malformed 0`) | debug : `attempt to add with overflow` (`report.rs:22`) ; release : 0 alerte | volumes en `u32` : 5,5 Go reboucle à ~1,2 Go, sous le seuil de 2 Go | `u64` partout (`BTreeMap<_, u64>`, plus de `as u32`) ; mentionner `checked_add`, `overflow-checks = true` en release |
| 2 | `report data` | `end byte index 34 is out of bounds for string of length 24` (`report.rs:17`) | `&line[open + 1..open + 14]` sur une ligne tronquée juste après `[` | `line.get(open + 1..open + 14)` + `continue` ; point d'arrêt conditionnel : `break report.rs:17 if line.length < open + 14` (gdb) pour afficher la ligne |
| 3 | `report data-abime` | `stream did not contain valid UTF-8` (`load.rs:7`) | `read_to_string(..).unwrap()` refuse tout le fichier | `fs::read` + `str::from_utf8` par ligne (comme l'étape 3 du fil rouge) ; `Utf8Error::valid_up_to` donne la position |
| 4 | `stats data --threads 8` | blocage (10 essais sur 10 sur la machine du formateur) ; `--threads 1` fonctionne | `failed` verrouille `failures` puis `accounts`, `accepted` verrouille `accounts` puis `failures` : interblocage | ordre unique (toujours `failures` puis `accounts`), ou un seul `Mutex<(HashMap, HashMap)>`, ou mieux : une table par thread fusionnée à la fin (étape 8) |
| 5 | `stream data` | les blocs s'agrègent puis plus rien | le thread principal garde `count_tx` : `for counts in count_rx` attend un émetteur qui n'émettra jamais | `drop(count_tx)` après le lancement des travailleurs (cf. étape 9) |
| bonus | `stats data --threads 1` | l'IP de la force brute est absente du top | `accepted` « blanchit » une adresse dont tous les échecs visaient le compte finalement ouvert : c'est exactement une force brute réussie | supprimer la règle ; discuter : une heuristique qui efface des données dans un outil de détection |

Ordre conseillé : 1, 2, 3 (TP 1, panique et inspection), puis 4, 5 (TP 2, concurrence),
bonus pour les plus rapides.

Session type, ticket 2 (gdb) :

```text
$ rust-gdb --args target/debug/sentinelle-turbo report data
(gdb) break rust_panic
(gdb) run
(gdb) bt                 # repérer la trame de sentinelle_turbo::report::run
(gdb) frame 9            # numéro selon la pile
(gdb) print line
(gdb) print open
```

Limite constatée en préparant : sur macOS, le débogueur doit être autorisé à s'attacher
aux processus (outils de développement Xcode, `DevToolsSecurity -enable`). À vérifier sur
les postes de la salle, avec `gdb` installé sous Linux (`rust-gdb` n'est qu'un script).
