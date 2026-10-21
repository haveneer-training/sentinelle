# Module 2 — Collections

| Temps | Activité | Support |
|---|---|---|
| cours (3 h) | Exemple TP 1 : manipulation de vecteurs | `exercices/m2/ex01_vec_failures` |
| | Exemple TP 2 : `HashMap` pour des paires clé/valeur | étape 4 du fil rouge |
| TP (4 h) | cas réels : les détections | étapes 5 à 7, 2 h 05 à 2 h 50 |
| | optimisation des collections dans un projet donné | ci-dessous, 45 min à 1 h |

## Optimiser les collections de la v0.1 (45 min à 1 h)

```shell
cargo run --release -p sentinelle-gen -- --out data-100M --users 2000 --noise 10
time cargo run --release -p sentinelle-legacy -- data-100M     # une dizaine de secondes
cargo test -p sentinelle-legacy                                 # le filet de sécurité
```

1. Repérez les deux collections inadaptées : `count_failures_by_ip` (recherche linéaire
   dans un `Vec`) et `detect_spraying` (`Vec::contains` pour dédoublonner). Quelle est la
   complexité de chacune ?
2. Remplacez-les par la collection adaptée (`HashMap`, `HashSet`, API `entry`). Mesurez.
3. Le test de caractérisation casse-t-il ? Si oui, lisez la différence : à égalité de
   compte, l'ordre de sortie a changé. Choisissez en connaissance de cause : garder
   l'ordre d'apparition (une table qui donne la position dans le `Vec`), ou décider d'un
   ordre (par adresse) et régénérer la référence (`UPDATE_GOLDEN=1`).

Le module 5 reviendra sur la mesure (`cargo bench`, profileurs).
