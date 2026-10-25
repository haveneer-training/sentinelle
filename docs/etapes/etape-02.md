# Étape 2 — Analyser `access.log`

> Module 1 (Références) · 30 à 40 min · `sentinelle/src/web.rs`

## L'enquête

`access.log` est le journal de l'application web interne. C'est par là que sortiront
les données. Même principe qu'à l'étape 1 : une `WebEntry<'a>` emprunte sa ligne.

## Ce que vous écrivez

`parse_web_line(line: &str) -> Result<WebEntry<'_>, ParseError>` pour le format

```
IP - USER [TIMESTAMP] "METHOD PATH HTTP/1.1" STATUS BYTES DURATION_MS
```

- `USER` vaut `-` pour une requête anonyme, `BYTES` vaut `-` sans corps de réponse :
  les deux deviennent `None` ;
- une ligne tronquée, ou avec un champ en trop, est une erreur, jamais une panique.

## Indices

- Découpez de gauche à droite sur les séparateurs successifs : `" - "`, `" ["`, `"] \""`,
  `"\" "`. Dans la requête, `rsplit_once(' ')` coupe au **dernier** espace : le protocole
  est toujours le dernier mot.
- `bool::then_some(value)` donne `Some(value)` si le booléen est vrai.
- Les volumes dépassent 4 Go : `u64`, pas `u32`.

## Vérifier

```shell
cargo test --test step2_web
```

Un test essaie **toutes** les troncatures d'une ligne valide.

## Extension

Écrivez `query_param(path: &str, key: &str) -> Option<&str>` (`/api/export?table=clients&page=3`,
clé `table` → `clients`). Quelle durée de vie pour la sortie, et pourquoi pas celle de
`key` ? Puis décodez les `%2F` en renvoyant un `Cow<'_, str>` : emprunté s'il n'y a rien
à décoder, possédé sinon.

## Diapositives

« Découper du texte sans copier » ; pour l'extension, « LES chaînes de caractères » (`Cow`)
(Les chaînes de caractères).

Rappel hors du deck de la formation, si besoin : « Closures : une fonction contextualisée »
(Esprit fonctionnel, `index.html`).
