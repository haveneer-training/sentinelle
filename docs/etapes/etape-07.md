# Étape 7 — Volumes et chronologie

> Module 2 (Collections) · 25 à 35 min · `sentinelle/src/detect.rs`

## L'enquête

Dernière pièce : la sortie des données. Elle se voit dans les volumes servis par
l'application web, heure par heure. Puis toutes les alertes se rangent en une chronologie.

## Ce que vous écrivez

1. `bytes_by_user_hour(entries) -> BTreeMap<(&str, Timestamp), u64>` : octets servis par
   (utilisateur, début d'heure).
2. `user_volume(volumes, user, from, to)` : le volume d'un utilisateur sur les heures de
   `[from, to)`, **sans parcourir toute la table**.
3. `timeline(groups)` : réunir des listes d'alertes et les trier par instant ; à instant
   égal, l'ordre des listes est conservé.

## Indices

- `ts.div_euclid(HOUR) * HOUR` tronque à l'heure.
- Les clés d'une `BTreeMap` sont triées : `range((user, from)..(user, to))` ne visite que
  l'intervalle demandé.
- `sort` est stable, `sort_unstable` ne l'est pas. `sort_by_key(Alert::at)` accepte un
  chemin de méthode.

## Vérifier

```shell
cargo test --test step7_volumes_timeline
cargo run --release -p sentinelle-gen
cargo run --release -p sentinelle -- data
```

La chronologie complète de l'intrusion s'affiche. Rédigez-en le récit en cinq lignes :
qui, depuis où, quand, comment, quoi.

## Extension

Remarquez l'horodatage des lignes de `srv-db-02` autour du rebond : son horloge dérive.
Détectez-le : pour chaque serveur, comptez les lignes dont l'instant recule par rapport
à la précédente du fichier.

## Diapositives

« Les conteneurs basiques 4/4 : structures associatives » ; « Arbre de décision pour
choisir son container ».
