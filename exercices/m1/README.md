# Module 1 — Les erreurs classiques du vérificateur d'emprunts

Neuf programmes courts, tirés de Sentinelle, qui **ne compilent pas**. Pour chacun :

1. lisez le message du compilateur en entier (`rustc --explain E0502` le détaille) ;
2. expliquez ce qui *pourrait* mal tourner si le compilateur acceptait ;
3. corrigez, en conservant les `assert!` : l'exercice affiche `exNN : OK`.

```shell
cargo run -p sentinelle-m1 --example ex01_mutable_aliasing
```

| # | Exercice | Erreur | Notion | Quand |
|---|---|---|---|---|
| 1 | `ex01_mutable_aliasing` | E0502 | emprunt partagé puis mutation | cours, Exemple TP 1 |
| 2 | `ex02_reused_buffer` | E0502 | tampon réutilisé, posséder ou emprunter | cours, Exemple TP 1 |
| 3 | `ex03_two_mutable_borrows` | E0499 | exclusivité de `&mut`, `get_disjoint_mut` | cours, Exemple TP 1 |
| 4 | `ex04_iterate_and_mutate` | E0502 | modifier pendant un parcours, `retain` | cours, Exemple TP 1 |
| 5 | `ex05_return_local_reference` | E0515 | renvoyer une référence vers une locale | cours, Exemple TP 2 |
| 6 | `ex06_lifetime_annotations` | E0106 | annoter une fonction, durées de vie distinctes | cours, Exemple TP 2 |
| 7 | `ex07_struct_outlives_data` | E0597 | structure qui emprunte | TP, bugs dans un projet |
| 8 | `ex08_map_keys_from_dropped` | E0597 | clés de table empruntées | TP, bugs dans un projet |
| 9 | `ex09_method_elision` | *lifetime may not live long enough* | élision dans une méthode | TP, bugs dans un projet |

Ce paquet ne fait pas partie des membres par défaut du workspace : `cargo test` à la
racine l'ignore, et votre éditeur y signalera des erreurs, c'est normal.
