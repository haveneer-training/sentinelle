# Module 5 — corrigé et mesures de référence

Mesures sur la machine du formateur (Apple Silicon, `cargo bench`, profil `bench`).
L'ordre de grandeur compte, pas les chiffres exacts.

## TP 1

| Mesure | v0.1 | Fil rouge |
|---|---|---|
| analyse d'une ligne | 613 ns (173 Mo/s) | 235 ns (451 Mo/s) |
| `auth.log` par défaut (3,8 Mo) | 25,6 ms (147 Mo/s) | 10,9 ms (346 Mo/s) |
| comptage, bruit ×1 / ×2 / ×4 | 44 / 189 / 894 ms (quadratique) | 0,47 / 0,98 / 2,09 ms (linéaire) |
| jeu de 100 Mo, binaire complet | 12,8 s, pic 1,13 Go | 0,47 s, pic 231 Mo |

Résumés parallèles (`--users 600 --noise 4`, 15 Mo) : séquentiel 300 Mo/s ; `scope` et
pipeline ~1 Go/s à 8 threads ; état partagé 100 à 180 Mo/s (pire que le séquentiel dès
2 threads : un verrou par ligne).

Mémoire de la v0.1 : le texte (100 Mo) + une `String` par champ et par ligne + quatre
copies complètes du journal d'authentification (les `.clone()` de `main`).

## TP 2

| # | Résultat | Corrigé |
|---|---|---|
| O1 | messages : 24,1 → 12,4 ms (×1,9), comportement identique (vérifié sur tous les messages du jeu) | `parse_auth_event.rs` |
| O2 | 12,8 s → < 1 s (voir `formateur/m4-legacy-refactored/src/analysis.rs`) | module 4 |
| O3 | aucun gain mesurable (340 contre 346 Mo/s) : le code chaud est déjà dans une seule crate, rien à inliner entre crates. Bonne leçon : mesurer avant d'adopter une recette | — |
| O4 | `failures_by_ip` (bruit ×4) : 2,11 → 0,95 ms (×2,2) | `fnv.rs` |
| O5 | lecture en flux : pic mémoire borné par la taille des blocs, au prix d'une copie des noms (`Summary<String>`, étape 9) | étape 9 |
