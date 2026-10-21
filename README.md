# Sentinelle

Fil rouge des travaux pratiques de la formation « Langage Rust — intermédiaire ».

## Le contexte

*Orbe Logistique* (entreprise fictive) a reçu vendredi matin une alerte de son
fournisseur d'accès : un volume inhabituel de données serait sorti de son réseau
dans la nuit. Personne ne sait encore ce qui s'est passé.

La semaine de journaux de ses serveurs a été récupérée. Un prestataire avait commencé
un outil d'analyse, **Sentinelle**, écrit dans l'urgence, avant de quitter le projet.
L'outil est lent, fragile et incomplet. Il donne pourtant quelques pistes.

Vous reprenez Sentinelle : vous en reconstruisez une version juste, rapide et robuste,
et vous menez l'enquête.

<!-- CORRIGE-ONLY-BEGIN -->
## Ce dépôt est le corrigé

Les stagiaires travaillent dans un dossier engendré à partir de lui, où les passages à
écrire deviennent des `todo!()` :

```shell
cargo xtask start      # crée travail/ (21 trous à combler)
cd travail
```

`cargo xtask verify` contrôle les exercices courts (`exercices/m1` à `m3`) contre leurs
corrigés. Le dossier `formateur/` (plan, scénario, corrigés, estimation) n'est jamais copié
dans `travail/`.
<!-- CORRIGE-ONLY-END -->

## Démarrage

```shell
cargo build --release

# jeu de données par défaut (≈ 9 Mo) dans data/
cargo run --release -p sentinelle-gen

# la v0.1 héritée
cargo run --release -p sentinelle-legacy -- data

# le fil rouge
cargo xtask status
cargo test
```

Le déroulement des étapes est décrit dans [`ETAPES.md`](ETAPES.md), les formats des
journaux dans [`docs/FORMATS.md`](docs/FORMATS.md).

Le formateur vous fournira aussi un jeu de données « officiel » dont l'enquête est à résoudre.
Pour les mesures de performance, générez un jeu plus gros :

```shell
cargo run --release -p sentinelle-gen -- --out data-100M --users 2000 --noise 10
```

`sentinelle-gen --help` liste toutes les options.

## Organisation du dépôt

| Répertoire | Contenu |
|---|---|
| `sentinelle/` | le fil rouge : bibliothèque, binaire et tests par étape |
| `legacy/` | Sentinelle v0.1, le code hérité, à lire, mesurer et refactoriser |
| `exercices/` | exercices courts des modules 1 à 3 (exemples de TP du cours) |
| `perf/` | mesures de performance (module 5) |
| `bugs/` | `sentinelle-turbo`, un outil qui plante, à déboguer (module 6) |
| `generator/` | `sentinelle-gen`, le générateur de journaux synthétiques |
| `xtask/` | l'outil qui pilote les étapes (`cargo xtask …`) |
| `docs/` | formats des journaux, énoncés des étapes, fiches des modules |

## Licence

CC BY-NC-SA 4.0 — Pascal Havé (contact@haveneer.com)
