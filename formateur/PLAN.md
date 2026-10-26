# Sentinelle — plan formateur

Document formateur : retirer `formateur/` avant de distribuer le dépôt.

Formation « Langage Rust — intermédiaire » (Ascent Formation, 5 jours, 35 h).
Sentinelle sert de fil rouge à tous les blocs de TP. Les « Exemple TP » du programme
sont traités en exercices courts, eux aussi tirés de Sentinelle.

## Le scénario

Généré par `sentinelle-gen` à partir de la graine. Toutes les valeurs (adresses,
victime, horaires) en dépendent. `--reveal` écrit la vérité terrain dans
`GROUND_TRUTH.md`. Distribuer aux stagiaires un jeu généré avec une graine non publiée.

Le jeu « officiel » (≈ 830 Mo, celui de l'étape 8) se génère ainsi, avec une graine
choisie pour la session (jamais 42, celle du jeu par défaut) :

```shell
cargo run --release -p sentinelle-gen -- --out data-officiel --seed <graine-secrète> \
    --users 20000 --noise 50 --invalid-utf8 --reveal
```

Ne distribuer que `auth.log` et `access.log` : `GROUND_TRUTH.md` (la solution) reste chez
le formateur. `--invalid-utf8` est couvert par le fil rouge dès l'étape 3 ; la v0.1, elle,
panique dessus (D8) et de toute façon ne passe pas à cette taille.

| Jour | Étape | Ce qui la révèle |
|---|---|---|
| mar. 14h–16h | Password spraying depuis une IP `203.0.113.x` : 1 essai par compte, ~70 % des comptes + 15 comptes inexistants, un essai toutes les 25–70 s | comptes distincts par IP (`HashSet`) |
| mer. 22h | Force brute depuis une IP `198.51.100.x` sur un salarié ordinaire : 350–450 échecs en ~15 min | fenêtre glissante (`VecDeque`) |
| mer. 22h | **Succès** juste après les échecs | corrélation échecs → `Accepted` (absente de la v0.1) |
| mer. 22h | Rebond bastion → `srv-web-01` | session hors horaires |
| jeu. ~1h30 | Rebond `srv-web-01` → `srv-db-02`, puis `sudo pg_dump` | couple (utilisateur, serveur) jamais vu ; sudo par un non-admin |
| ven. ~3h | Exfiltration : ~300 `GET /api/export?table=clients` depuis l'IP de la force brute, > 5,5 Go | volume par (utilisateur, heure) (`BTreeMap`), top-N (`BinaryHeap`) ; dépasse `u32::MAX` |

Pièges réalistes semés dans les données :
- L'horloge de `srv-db-02` dérive de 1 à 3 minutes : reconstruire une chronologie
  multi-serveurs impose de trier, et l'ordre du fichier n'est pas l'ordre des horodatages.
- La ligne `sudo` contient des espaces multiples (champs vides avec `split(' ')`).
- `[PID]` optionnel ; `BYTES` vaut `-` pour les `304`.
- `--malformed P` : lignes tronquées, vides ou amputées d'un champ (défaut 0,05 %).
- `--invalid-utf8` : octets invalides, qui font paniquer `read_to_string` (v0.1).
- Le bruit de fond (scanners SSH et web, cron) masque l'attaque et grossit les volumes.

Volumes, 7 jours (machine du formateur, release) :

| Options | Taille | v0.1 |
|---|---|---|
| défaut (150 salariés) | 9 Mo | 0,2 s |
| `--users 2000 --noise 10` | 100 Mo | 12,8 s |
| `--users 20000 --noise 50` | 830 Mo | (quadratique : inutilisable) |

## La v0.1 héritée (`legacy/src/main.rs`)

Fonctionne sur le jeu par défaut, mais contient volontairement :

| # | Défaut | Module |
|---|---|---|
| D1 | Les fonctions d'analyse prennent `Vec<AuthEntry>` par valeur : 4 copies complètes du journal (`.clone()` à chaque appel) | 1 |
| D2 | Structures tout en `String`, champs parsés jamais lus (avertissements `dead_code`) ; `fn field(parts: &Vec<String>, …) -> String` | 1, 5 |
| D3 | **Bug** : `FAILED` + `INVALID` comptés ensemble, donc deux fois par tentative sur un compte inexistant (spraying : 136 « échecs » pour 121 comptes) | 1 (TP « analyse de code réel ») |
| D4 | Comptage par recherche linéaire dans un `Vec<(String, i32)>` ; dédoublonnage par `Vec::contains` : coût quadratique | 2, 5 |
| D5 | Double recherche `contains_key` + `insert` / `get_mut().unwrap()` au lieu de l'API `entry` | 2 |
| D6 | Force brute = « plus de 100 échecs sur la semaine », sans fenêtre de temps ; compromission non détectée | 2 |
| D7 | Une ligne amputée d'un champ est parsée de travers sans erreur (`user = "from"`) | 1, 4 |
| D8 | `read_to_string(..).unwrap()` : panique sur UTF-8 invalide | 6 (voire 1 : `&[u8]` contre `&str`) |
| D9 | `kind: String` au lieu d'un `enum` ; tout dans `main.rs` ; aucune doc ni test ; `timestamp: timestamp` | 4 |
| D10 | Lecture du fichier entier en mémoire, puis copie de chaque ligne (`line.to_string()`) | 3, 5 |

## Organisation

- **Le fil rouge `sentinelle/`**, piloté par `cargo xtask` comme wind-tunnel : ce dépôt
  est le corrigé, `cargo xtask start` engendre `travail/` où les blocs
  `SOLUTION-BEGIN/END` deviennent des `todo!()`. Onze étapes (0 à 10, la 10 en bonus),
  une feature `stepN` par étape, `step11` sert de sentinelle. Les trous portent sur ce
  qui **s'ajoute** (modules 1 à 3). Les tests s'appuient sur `sentinelle_gen::Dataset` :
  ils vérifient la détection du scénario réel, vérité terrain comprise.
- **Les exercices courts** `exercices/m1` à `m3` (exemples de TP du cours) : `m1` ne
  compile pas (erreurs du vérificateur d'emprunts), `m2` et `m3` sont des `todo!()`.
  Corrigés dans `formateur/solutions/`, contrôlés par `cargo xtask verify`.
- **Le code existant, hors étapes** : `legacy/` (modules 1, 2, 4, 5) avec son test de
  caractérisation ; `perf/` (module 5) ; `bugs/` (module 6).

| Module | Cours (exemples de TP) | TP | Fiche |
|---|---|---|---|
| 1. Références | `exercices/m1` 1 à 6 | lecture de `legacy/`, étapes 0 à 3, `exercices/m1` 7 à 9 | `docs/modules/module-1.md` |
| 2. Collections | `exercices/m2`, étape 4 | étapes 5 à 7, optimisation des collections de `legacy/` | `docs/modules/module-2.md` |
| 3. Concurrence | `exercices/m3` 1 et 2 | étapes 8 et 9, bonus étape 10 | `docs/modules/module-3.md` |
| 4. Bonnes pratiques | — | refactorisation de `legacy/` sous test, rustdoc | `docs/modules/module-4.md` |
| 5. Performance | — | `cargo bench`, profileurs, mémoire ; trois optimisations | `docs/modules/module-5.md` |
| 6. Débogage | — | tickets 1 à 3 (paniques), 4 et 5 (concurrence), bonus | `docs/modules/module-6.md` |

Durées, notions et références aux diapositives : `ESTIMATION.local.md` (non versionné).

## Corrigés formateur

| Fichier | Contenu |
|---|---|
| `solutions/m1`, `m2`, `m3` | corrigés des exercices courts (`cargo xtask verify`) |
| `m4-legacy-refactored/` | la v0.1 refactorisée, sortie identique (paquet autonome : `cargo test --manifest-path formateur/m4-legacy-refactored/Cargo.toml`) |
| `solutions/m5/` | mesures de référence, `parse_auth_event` sans allocation, hachage FNV |
| `BUGS.md` | causes et corrections des tickets du module 6 |

## Prérequis techniques à faire valider

- Réseau probablement isolé : seuls `perf/criterion/` (facultatif) a des dépendances
  externes ; préparer son archive `cargo vendor` (procédure dans `docs/modules/module-5.md`).
  Tout le reste se construit sans réseau.
- `gdb` (Linux ; `rust-gdb` n'est qu'un script) ou lldb autorisé à s'attacher aux processus
  (macOS : `DevToolsSecurity -enable`) ; `perf` ou `samply` pour le profilage.
- Rust **1.90.0**, épinglé par `rust-toolchain.toml` (copié dans `travail/`, même version en CI) : à installer
  sur chaque poste avant la formation, réseau probablement isolé :
  `rustup toolchain install 1.90.0 --profile minimal --component rustfmt,clippy`.

## Reste à faire

- [x] fil rouge, étapes 0 à 10, xtask (`start`, `goto`, `solve`, `reset`, `status`, `verify`)
- [x] énoncés des étapes (`docs/etapes/`) et fiches des modules (`docs/modules/`)
- [x] exercices courts `m1` à `m3` et leurs corrigés
- [x] module 4 : test de caractérisation de `legacy/`, consignes, corrigé
- [x] module 5 : `perf/` (sans dépendance), `perf/criterion/`, mesures de référence
- [x] module 6 : `bugs/`, tickets, fiche formateur
- [ ] répéter une session de débogage complète sur un poste Linux de la salle (non faite :
      le bac à sable de la machine de préparation interdit l'attachement du débogueur)
- [ ] préparer l'archive hors ligne de `perf/criterion/` une fois l'environnement connu
- [x] deck de la formation : `index3.html` (édition DGSE en parties ; voir `ESTIMATION.local.md`,
      « Matériel disponible »)
