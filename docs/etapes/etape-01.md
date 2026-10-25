# Étape 1 — Analyser `auth.log` sans copier

> Module 1 (Références) · 45 à 60 min · `sentinelle/src/auth.rs`

## L'enquête

`auth.log` raconte qui s'est connecté où, et qui a échoué. La v0.1 copie chaque champ
dans une `String` : sur un journal de 800 Mo, c'est des millions d'allocations. Ici,
une `AuthEntry<'a>` ne possède rien : ses champs textuels sont des tranches de la ligne.

## Ce que vous écrivez

1. `parse_auth_line(line: &str) -> Result<AuthEntry<'_>, ParseError>` : l'en-tête
   `TIMESTAMP HOST PROCESS[PID]: MESSAGE`. Le `[PID]` est facultatif (`sudo` n'en a pas).
2. `parse_auth_event(message: &str) -> Result<AuthEvent<'_>, ParseError>` : chaque forme
   de message décrite dans `docs/FORMATS.md` devient un variant de `AuthEvent` ; tout le
   reste devient `AuthEvent::Other(message)`.

## Indices

- `str::split_once(' ')` coupe en deux au premier séparateur ; `strip_suffix(']')` retire
  un suffixe s'il est là. `Option::ok_or(err)` transforme un `None` en erreur précise.
- Le `'_` de la signature dit : « la sortie emprunte l'entrée » (règle d'élision). Vous
  n'avez rien à annoter, mais tout ce que vous mettez dans `AuthEntry` doit provenir de `line`.
- Les **motifs de tranche** reconnaissent une suite de mots :
  ```rust
  match words[..] {
      ["Accepted", method, "for", user, "from", addr, ..] => …,
      [user, ":", tty, ..] if tty.starts_with("TTY=") => …,
      _ => …,
  }
  ```
  L'ordre des bras compte : `Failed password for invalid user X` doit être reconnu avant
  `Failed password for X`.
- Les lignes `sudo` contiennent des espaces multiples : `split_whitespace`, pas `split(' ')`.
- `"10.0.0.1".parse::<IpAddr>()` lit une adresse.

## Vérifier

```shell
cargo test --test step1_auth
```

Un test vérifie que `host` pointe bien **dans** la ligne : une copie serait refusée.

## Extension

`parse_auth_event` alloue un `Vec<&str>` par ligne. Écrivez une version sans allocation
(un tableau de taille fixe rempli depuis l'itérateur, ou des `next()` successifs) ; vous
mesurerez le gain au module 5.

## Diapositives

« Découper du texte sans copier » (Les chaînes de caractères) ; « Les cas implicitement gérés
par le compilateur : règles d'élision » (Lifetime et borrowing avancé).

Rappels hors du deck de la formation, si besoin : « Pattern matching & cie (suite) »,
« Nommer un lifetime » (Concepts clés, `index.html`).
