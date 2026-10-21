# Étape 4 — Compter et classer

> Module 2 (Collections) · 30 à 45 min · `sentinelle/src/detect.rs`

## L'enquête

Première question : quelles adresses échouent le plus ? La v0.1 y répond… en comptant
deux fois chaque tentative sur un compte inexistant, et en cherchant chaque adresse dans
un `Vec` (coût quadratique).

## Ce que vous écrivez

1. `failures_by_ip(entries) -> HashMap<IpAddr, usize>` : un échec = un `AuthEvent::Failed`
   (les lignes `Invalid user` ne comptent pas, elles doublonnent).
2. `top_n(items, n) -> Vec<(K, V)>` : les `n` plus grandes valeurs, à valeur égale la plus
   petite clé d'abord, **sans trier toute la table**.

## Indices

- `map.entry(key).or_insert(0)` renvoie un `&mut` vers la valeur : une seule recherche.
- `BinaryHeap` est un tas-max ; `std::cmp::Reverse(x)` inverse l'ordre. Un tas-min de
  taille `n` garde les `n` meilleurs : on pousse, et on éjecte la racine quand il déborde.
- Les tuples se comparent dans l'ordre de leurs champs.

## Vérifier

```shell
cargo test --test step4_counting
```

## Diapositives

« L'API Entry », « Les autres conteneurs : le tas binaire », « Complexité des opérations
courantes » (Les conteneurs de données et leurs usages).
