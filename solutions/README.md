# Solutions

Les stagiaires reçoivent tout le dépôt, `solutions/` compris : les corrigés sont une
référence, à consulter après avoir cherché, jamais avant.

## Les corrigés

Le fil rouge (`sentinelle/`) est lui-même son corrigé : `cargo xtask start` en retire les
solutions pour engendrer `travail/`. Ce dossier contient tout le reste.

| Chemin | Contenu |
|---|---|
| `m1/`, `m2/`, `m3/` | corrigés des exercices courts de `exercices/`, contrôlés par `cargo xtask verify` |
| `m4-legacy-refactored/` | la v0.1 refactorisée, sortie identique ; correction de D3 derrière `--features fix-d3` (paquet autonome : `cargo test --manifest-path solutions/m4-legacy-refactored/Cargo.toml`) |
| `m5/` | mesures de référence, `parse_auth_event` sans allocation, hachage FNV |
| `BUGS.md` | causes et corrections des tickets du module 6 |
| `bonus-injection/` | bonus du module 1 (`--injection`) : enquête, détections des noms suspects et des connexions fantômes, correction à la source |
| `m6-turbo-fixed/` | `bugs/` corrigé, un test par ticket ; `scripts/check-bugs.sh` vérifie en plus que `bugs/` présente toujours chaque symptôme |

## Les défauts de la v0.1 (`legacy/src/main.rs`)

La v0.1 fonctionne sur le jeu par défaut, mais contient volontairement les défauts
suivants. Les corrigés y renvoient par leur numéro.

| # | Défaut | Module |
|---|---|---|
| D1 | Les fonctions d'analyse prennent `Vec<AuthEntry>` par valeur : 4 copies complètes du journal (`.clone()` à chaque appel) | 1 |
| D2 | Structures tout en `String`, champs parsés jamais lus (avertissements `dead_code`) ; `fn field(parts: &Vec<String>, …) -> String` | 1, 5 |
| D3 | **Bug** : `FAILED` + `INVALID` comptés ensemble, donc deux fois par tentative sur un compte inexistant (spraying : 136 « échecs » pour 121 comptes) | 1, 4 |
| D4 | Comptage par recherche linéaire dans un `Vec<(String, i32)>` ; dédoublonnage par `Vec::contains` : coût quadratique | 2, 5 |
| D5 | Double recherche `contains_key` + `insert` / `get_mut().unwrap()` au lieu de l'API `entry` | 2 |
| D6 | Force brute = « plus de 100 échecs sur la semaine », sans fenêtre de temps ; compromission non détectée | 2 |
| D7 | Une ligne amputée d'un champ est parsée de travers sans erreur (`user = "from"`) | 1, 4 |
| D8 | `read_to_string(..).unwrap()` : panique sur UTF-8 invalide | 4, 6 |
| D9 | `kind: String` au lieu d'un `enum` ; tout dans `main.rs` ; aucune doc ni test ; `timestamp: timestamp` | 4 |
| D10 | Lecture du fichier entier en mémoire, puis copie de chaque ligne (`line.to_string()`) | 3, 5 |

## Le scénario

Généré par `sentinelle-gen` à partir d'une graine (42 par défaut, `--seed N` pour une
autre) : adresses, victime et horaires en dépendent. `--reveal` écrit les valeurs exactes dans `GROUND_TRUTH.md`. Le déroulé, lui,
ne change pas :

| Jour | Étape | Ce qui la révèle |
|---|---|---|
| mar. 14h–16h | Password spraying depuis une IP `203.0.113.x` : 1 essai par compte, ~70 % des comptes + 15 comptes inexistants, un essai toutes les 25–70 s | comptes distincts par IP (`HashSet`, étape 5) |
| mer. 22h | Force brute depuis une IP `198.51.100.x` sur un salarié ordinaire : 350–450 échecs en ~15 min | fenêtre glissante (`VecDeque`, étape 5) |
| mer. 22h | **Succès** juste après les échecs | corrélation échecs → `Accepted` (étape 6 ; absente de la v0.1) |
| mer. 22h | Rebond bastion → `srv-web-01` | session hors horaires |
| jeu. ~1h30 | Rebond `srv-web-01` → `srv-db-02`, puis `sudo pg_dump` | couple (utilisateur, serveur) jamais vu ; sudo par un non-admin (étape 6) |
| ven. ~3h | Exfiltration : ~300 `GET /api/export?table=clients` depuis l'IP de la force brute, > 5,5 Go | volume par (utilisateur, heure) (`BTreeMap`, étape 7) ; dépasse `u32::MAX` (ticket 1 du module 6) |

Les pièges des données (dérive d'horloge de `srv-db-02`, espaces multiples, `[PID]`
optionnel, `-` pour les `304`, lignes corrompues) sont décrits dans `docs/FORMATS.md` et
dans les énoncés des étapes concernées.

### Le jeu officiel

Le jeu de l'enquête finale (≈ 830 Mo) se génère avec une graine choisie pour la session,
jamais 42 (celle du jeu par défaut) :

```shell
cargo run --release -p sentinelle-gen -- --out data-officiel --seed <graine-secrète> \
    --users 20000 --noise 50 --invalid-utf8 --reveal
```

Ne remettre aux stagiaires que `auth.log` et `access.log` : sans la graine, `--reveal` ne
leur sert à rien, et `GROUND_TRUTH.md` permet de vérifier leur récit. `--invalid-utf8`
est couvert par le fil rouge dès l'étape 3 ; la v0.1 panique dessus (D8), et de toute
façon ne passe pas à cette taille (coût quadratique, D4).

| Options | Taille | v0.1 |
|---|---|---|
| défaut (150 salariés) | 9 Mo | 0,2 s |
| `--users 2000 --noise 10` | 100 Mo | 12,8 s |
| `--users 20000 --noise 50` | 830 Mo | inutilisable |

### Lire la sortie de `sentinelle`

Sur le jeu officiel, la chronologie compte des milliers d'alertes. Pour retrouver le
scénario sans connaître la réponse, on ne garde que les types d'alertes rares :

```shell
cargo run --release -q -p sentinelle -- data-officiel 2>/dev/null \
  | grep -E 'password spraying|force brute|COMPROMISSION|volume anormal'
```

Il reste une poignée de lignes : la force brute, puis la `COMPROMISSION` depuis la même
adresse quelques secondes après la dernière tentative, et le `volume anormal`. Le compte de
la victime y apparaît. On le suit ensuite dans la sortie complète
(`grep <compte>`) : rebonds vers `srv-web-01` puis `srv-db-02` (« accès inédit »),
`pg_dump` (« sudo inhabituel »). Avec `--injection`, deux fausses `COMPROMISSION root`
depuis `192.0.2.77` passent aussi le filtre : c'est le piège du bonus.

Les heures de `srv-db-02` sont décalées de la dérive d'horloge indiquée dans
`GROUND_TRUTH.md` : celui-ci donne les heures réelles, `sentinelle` celles écrites dans le
journal.

**Le bruit des « accès inédits ».** Un couple (compte, serveur) jamais vu pendant les deux
jours d'apprentissage (`Config::starting_at`) déclenche une alerte. Sur le jeu par défaut
(150 salariés), c'est une trentaine d'alertes. Sur le jeu officiel (20 000 salariés), deux
jours ne suffisent pas à voir les habitudes de chacun : environ 3 800 alertes, qui
décroissent de jour en jour (≈ 2 200 le mercredi, 1 000 le jeudi, 500 le vendredi) à mesure
que les couples sont appris. Les rebonds de l'attaquant y sont noyés. Bon point de
discussion : une détection juste sur un petit jeu devient inexploitable à l'échelle si ses
alertes ne sont pas hiérarchisées (période d'apprentissage proportionnée au volume, score de
rareté, corrélation avec une compromission déjà détectée).

Les « sudo inhabituel » (environ 340 sur le jeu officiel) sont d'une autre nature : des
commandes d'administration courantes (`systemctl status`, `df -h`, `journalctl`…) lancées
par des comptes qui n'en avaient pas lancé pendant l'apprentissage. Rares, elles méritent
un coup d'œil, sans plus : c'est la commande qui alerte (un `pg_dump` de la base clients),
pas le seul fait qu'un `sudo` soit nouveau.
