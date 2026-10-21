# Étape 0 — Lire un horodatage

> Module 1 (Références) · 20 à 30 min · `sentinelle/src/time.rs`

## L'enquête

Toutes les lignes des deux journaux commencent par un instant au format ISO 8601 :
`2026-11-09T08:12:45Z`. Avant de comparer des connexions, il faut savoir lire ces
instants et les convertir en secondes (`Timestamp`, un `i64`).

## Ce que vous écrivez

`parse_timestamp(text: &str) -> Option<Timestamp>` :

- refuser tout ce qui n'a pas **exactement** la forme `AAAA-MM-JJTHH:MM:SSZ` (20 octets) ;
- extraire les six nombres par tranches (`&text[0..4]`, …) ;
- refuser les valeurs hors domaine (mois 13, 24 h, 60 min…) ;
- convertir avec `days_from_civil`, fournie.

## Indices

- `text` est emprunté : vous ne créez aucune `String`. Une tranche `&text[a..b]` est une
  vue sur le même texte.
- Trancher un `&str` au milieu d'un caractère multi-octets **panique**. Vérifiez la forme
  sur les octets (`text.as_bytes()`, `u8::is_ascii_digit`) avant de trancher.
- Dans une fonction qui renvoie `Option`, l'opérateur `?` propage un `None`.

## Vérifier

```shell
cargo test --test step0_time
cargo test --doc
```

Le commentaire de documentation de la fonction contient un exemple : `cargo test --doc`
l'exécute. La documentation est du code testé.

## Extension

Écrivez un test qui parcourt toute une journée seconde par seconde et vérifie l'aller-retour
`parse_timestamp(&Iso(ts).to_string()) == Some(ts)`.

## Diapositives

« array, slice, Box » et « String et str » (conteneurs basiques) ; « Tout est expression ».
