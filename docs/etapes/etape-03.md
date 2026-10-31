# Étape 3 — Un journal entier

> Module 1 (Références) · 20 à 30 min · `sentinelle/src/journal.rs`

## L'enquête

La v0.1 lit les fichiers avec `fs::read_to_string(..).unwrap()` : la moindre ligne
d'UTF-8 invalide fait tout planter. Ici, le fichier est lu en octets (`Vec<u8>`), une
fois, puis chaque ligne est analysée ; les lignes défectueuses sont comptées, pas fatales.

## Ce que vous écrivez

```rust
pub fn parse_lines<'a, T>(text: &'a [u8], parse: impl Fn(&'a str) -> Result<T, ParseError>) -> Journal<T>
```

Pour chaque ligne de `lines(text)` (fournie) : la convertir en `&str`, l'analyser, et
ranger le résultat dans `entries` ou, avec son numéro (à partir de 1), dans `errors`.

## Indices

- Lisez la signature avant d'écrire : le même `'a` relie le texte, l'argument de `parse`
  et donc les `T` produits. C'est ce qui permet à `T` d'être une `AuthEntry<'a>`.
- `std::str::from_utf8(bytes)` vérifie l'UTF-8 sans rien copier.
- `Result::and_then` enchaîne deux étapes qui peuvent échouer ; il accepte `&parse`.

## Vérifier

```shell
cargo test --test step3_journal
```

## Extension

Écrivez une version paresseuse, qui ne construit pas de `Vec` :
`fn entries(text: &[u8]) -> impl Iterator<Item = Result<AuthEntry<'_>, ParseError>>`.
Qu'a changé l'édition 2024 dans la capture des durées de vie par `impl Trait` ? Voir
[*RPIT lifetime capture rules*](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html)
dans l'*Edition Guide*.

## Diapositives

« Des octets au texte : l'UTF-8 aux frontières » (Les chaînes de caractères) ; « Les
annotations explicites » (Lifetime et borrowing avancé).
