# Bonus — Des noms de compte piégés

> Module 1 (chaînes de caractères) · 45 à 75 min · à faire quand le fil rouge sait analyser
> `auth.log` (étape 6 terminée, ou `cargo xtask goto 7`)

## L'enquête

sshd recopie dans `auth.log` le nom de compte **tel que le client l'envoie** :
`Invalid user NOM from IP port PORT`. Chez Orbe Logistique, rien ne l'échappe en
chemin. Un attaquant choisit donc une partie du texte du journal.

Générez un jeu où un attaquant en profite, puis lancez votre Sentinelle dessus :

```shell
cargo run --release -p sentinelle-gen -- --out data-injection --injection
cargo run --release -p sentinelle -- data-injection
```

Sentinelle signale une nouvelle compromission, sur `root` cette fois. Est-elle réelle ?

## Ce que vous faites

1. **Enquêter.** Retrouvez dans `data-injection/auth.log` les lignes qui produisent
   l'alerte, et celles qui les entourent. Qu'ont-elles de différent d'une vraie
   connexion ? Combien d'indices trouvez-vous ?
2. **Afficher sans se faire piéger.** Le même attaquant a essayé d'autres noms de compte.
   Listez ceux de son adresse et affichez-les d'abord avec `{}`, puis avec `{:?}`. Que
   voyez-vous dans chaque cas ?
3. **Détecter les noms suspects.** Écrivez une détection des noms de compte qui
   contiennent un caractère de contrôle, un caractère invisible (espace sans chasse,
   contrôle bidirectionnel) ou un caractère non ASCII, et affichez-les avec `{:?}`.
4. **Détecter les connexions fantômes.** Écrivez une détection des `Accepted` qui ne sont
   suivis d'aucune ouverture de session (`pam_unix(sshd:session): session opened`).
5. **Corriger à la source.** Le générateur joue le rôle de sshd. Dans
   `generator/src/injection.rs`, faites-lui échapper le nom de compte avant de l'écrire,
   régénérez le jeu et relancez Sentinelle. Que reste-t-il de l'alerte ?

## Indices

- Un `grep -n -B1 -A1` autour de l'alerte suffit pour l'étape 1. Comparez avec une vraie
  connexion : `grep -A1 "Accepted" data-injection/auth.log | head`.
- Une vraie connexion produit, avec le **même PID**, un `Accepted` puis une ouverture de
  session. Le fichier est trié dans l'ordre d'arrivée : regardez aussi les horodatages
  voisins.
- `{:?}` sur un `&str` échappe les sauts de ligne, les caractères invisibles et les
  contrôles bidirectionnels, mais pas une lettre cyrillique : pour celle-ci, `is_ascii`.
- `char::is_control` repère `\n` ou `\t`. Les caractères invisibles se listent à la
  main : `U+200B` à `U+200F`, `U+202A` à `U+202E`, `U+2066` à `U+2069`, `U+FEFF`.
- `str::escape_debug` renvoie un itérateur qui implémente `Display` : il s'utilise
  directement dans `format_args!`, sans `String` intermédiaire.

## Vérifier

`--reveal` écrit dans `GROUND_TRUTH.md` une section « Bonus » qui décrit les noms envoyés
et les fausses lignes. Ne la lisez qu'après l'étape 1.

Le corrigé est dans [`solutions/bonus-injection/`](../../solutions/bonus-injection/README.md).

## Pour aller plus loin

- Le format exact d'une ligne `Accepted` se termine par `ssh2`. Rendre l'analyse stricte
  (motif de tranche sans `..`) suffit-il à rejeter les fausses lignes ? Toutes ?
- Les homoglyphes : `rооt` et `root` sont deux comptes différents pour Sentinelle. Que
  faudrait-il pour les rapprocher (UTS #39, « skeleton ») ?
- CWE-117 (*Improper Output Neutralization for Logs*) : où d'autre vos programmes
  écrivent-ils dans un journal un texte reçu de l'extérieur ?
