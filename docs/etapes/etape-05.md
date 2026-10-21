# Étape 5 — Spraying et force brute

> Module 2 (Collections) · 60 à 80 min · `sentinelle/src/detect.rs`

## L'enquête

Deux attaques classiques : le *password spraying* (un mot de passe courant essayé sur
beaucoup de comptes, lentement) et la force brute (beaucoup de mots de passe sur un
compte, vite). Elles ne se voient pas avec les mêmes outils.

## Ce que vous écrivez

1. `detect_spraying(entries, min_accounts)` : les adresses ayant échoué sur au moins
   `min_accounts` comptes **distincts**, avec le premier et le dernier instant.
2. `detect_bruteforce(entries, window, threshold)` : les rafales d'au moins `threshold`
   échecs d'une même adresse en moins de `window` secondes. Une rafale se prolonge tant que
   le seuil reste atteint, et n'est rapportée qu'une fois.

## Indices

- Comptes distincts : un `HashSet<&str>` par adresse. Les noms empruntent le journal.
- Fenêtre glissante : un `VecDeque<Timestamp>` par adresse ; `push_back` l'échec courant,
  `pop_front` tant que le plus ancien est sorti de la fenêtre. La taille de la file est le
  nombre d'échecs récents.
- Gardez dans une seconde table la rafale **en cours** de chaque adresse : on la prolonge,
  ou on la clôt (dans le résultat) pour en ouvrir une nouvelle.
- `let AuthEvent::Failed { ip, .. } = entry.event else { continue };`

## Vérifier

```shell
cargo test --test step5_spraying_bruteforce
```

## Extension

Une attaque distribuée change d'adresse à chaque essai. Agrégez les échecs par réseau /24
(`Ipv4Addr::octets`) : que voyez-vous dans le bruit de fond ?

## Diapositives

« Les autres conteneurs std : la file à double extrémité », « Benchmark : files FIFO et files
de priorité » ; « Les erreurs classiques du borrow checker ».
