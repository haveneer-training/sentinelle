# Module 4 — Bonnes pratiques : reprendre le code hérité

Le fil rouge a été écrit proprement dès le départ. Le vrai travail quotidien est
ailleurs : reprendre un code qui fonctionne à peu près, sans le casser. C'est le sort de
`legacy/` (Sentinelle v0.1).

## TP 1 — Refactoriser sous test (2 h à 2 h 30)

### La règle

Une refactorisation **ne change pas le comportement**. Le test de caractérisation le
vérifie : il compare la sortie de `sentinelle-v0` sur un jeu fixe à une référence.

```shell
cargo test -p sentinelle-legacy      # vert avant de commencer, vert après chaque étape
cargo clippy -p sentinelle-legacy    # ce que l'outil reproche au code
```

Versionnez (`git commit`) après chaque étape verte : vous pourrez toujours revenir en arrière.

### Les étapes

1. **État des lieux.** Lancez `cargo clippy` et `cargo fmt --check`. Classez les
   remarques : style, idiomes, vrais défauts. Corrigez celles de style
   (`timestamp: timestamp`…).
2. **Emprunter au lieu de copier.** Les fonctions d'analyse prennent `Vec<AuthEntry>` par
   valeur, d'où les `.clone()` à chaque appel : passez-les en `&[AuthEntry]`. Qu'est-ce
   qui disparaît dans `main` ? `detect_bruteforce` recompte ce que `count_failures_by_ip`
   vient de compter : réutilisez le résultat.
3. **Des types qui disent ce qu'ils sont.** `kind: String` devient un `enum AuthKind`.
   Les champs jamais lus disparaissent (le compilateur vous les désigne).
4. **Bibliothèque et binaire.** Créez `src/lib.rs` ; `main.rs` ne garde que la ligne de
   commande. Découpez en modules : `entry` (les types), `parse` (lecture d'une ligne),
   `analysis` (comptages, détections), `report` (mise en forme). Tout est privé par défaut :
   n'exposez que ce que `main` utilise, `pub(crate)` pour le reste.
5. **Des erreurs au lieu de paniques.** `read_to_string(..).unwrap()` devient un
   `enum LoadError` (fichier illisible, UTF-8 invalide avec sa position) qui implémente
   `Display` et `std::error::Error` ; `main` l'affiche et renvoie `ExitCode::FAILURE`.
   Testez avec `sentinelle-gen --invalid-utf8 --malformed 0.01`.
6. **Nommage et constantes.** `100` et `20` deviennent des constantes nommées ; vérifiez les
   conventions (`snake_case`, `CamelCase`, `SCREAMING_SNAKE_CASE`, méthodes `as_`/`to_`/`into_`).
7. **Corriger, enfin.** Le double comptage des comptes inexistants (`Invalid user` +
   `Failed … invalid user`) est un bug : corrigez-le, puis régénérez la référence
   (`UPDATE_GOLDEN=1`) et **relisez la différence** : seuls les comptes attendus ont-ils changé ?

### Pour les plus rapides

- Si ce n'est pas fait au module 2 : `count_failures_by_ip` cherche chaque adresse dans un
  `Vec`. Remplacez-la par une table, **sans** changer l'ordre de sortie à égalité (indice :
  une table qui donne la position dans le `Vec`).
- Configurez les lints du projet dans `Cargo.toml` (`[lints.clippy]`) et passez
  `cargo clippy -- -D warnings`.

## TP 2 — Documenter un module (45 min à 1 h)

1. Une documentation de crate (`//!` en tête de `lib.rs`) : à quoi sert-elle, comment
   l'utiliser, avec un tableau des modules et des liens (`[`parse`]`).
2. Un `///` sur chaque élément public, avec une section `# Errors` quand une fonction
   renvoie `Result`.
3. Des exemples dans la documentation de `parse` : ils sont compilés et exécutés par
   `cargo test --doc`. Montrez un cas qui marche et un cas refusé.
4. `#![warn(missing_docs)]` : plus rien d'oublié.
5. `cargo doc --open` : lisez votre documentation comme un utilisateur la lirait.

## Diapositives

Structuration d'un projet : « Des modules, des crates et des packages », « Visibilité :
tout est privé par défaut », « Concevoir l'API publique d'une crate ». Bonnes pratiques :
« cargo fmt et cargo clippy », « Conventions de nommage », « Conventions de méthodes »,
« Configurer les lints du projet », « Rustdoc : écrire une documentation utile »,
« Rustdoc : générer la documentation HTML », « Rustdoc : faire respecter la documentation ».
