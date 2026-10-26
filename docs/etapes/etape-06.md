# Étape 6 — Compromission et habitudes

> Module 2 (Collections) · 40 à 55 min · `sentinelle/src/detect.rs`

## L'enquête

Une force brute qui échoue n'est qu'une nuisance. Une force brute **suivie d'un succès**
est une compromission. Ensuite, l'attaquant se déplace : il apparaît là où la victime ne
va jamais, et lance des commandes qu'elle ne lance jamais.

## Ce que vous écrivez

1. `detect_compromise(entries, min_failures)` : les connexions réussies depuis une adresse
   qui a *déjà* cumulé `min_failures` échecs (`Failed`, tous comptes et serveurs confondus,
   depuis le début du journal). Chaque connexion au-dessus du seuil est rapportée.
2. `detect_new_access(entries, learn_until)` : la première connexion d'un utilisateur sur
   un serveur, quand elle survient après la période d'apprentissage.
3. `detect_unusual_sudo(entries, learn_until)` : les `sudo` de comptes qui n'en ont lancé
   aucun pendant l'apprentissage.

## Indices

- Les résultats (`Compromise<'a>`, …) empruntent le journal : regardez comment le `'a` de
  la signature relie entrées et sorties.
- `HashSet::insert` renvoie `true` si l'élément était absent : c'est tout le test de
  nouveauté. Un tuple `(&str, &str)` fait une très bonne clé.
- Deux passages sont plus clairs qu'un seul pour le `sudo` ; `filter_map` + `collect`
  construit directement un `HashSet`.

## Vérifier

```shell
cargo test --test step6_correlation
```

## Extension

Sur le jeu par défaut, « accès inédit » lève une trentaine de fausses alertes : des
salariés qui vont pour la première fois sur un serveur, en journée, depuis leur poste.
Filtrez ce qui est vraiment suspect (hors heures ouvrées, adresse non privée :
`Ipv4Addr::is_private`). Quelles alertes restent ?

## Diapositives

« Itérateurs & chaînage fonctionnel : un premier exemple », « Adaptateurs 1/2 : transformer »
(`filter_map`), « Consommateurs 2/2 : collect » (Les itérateurs).
