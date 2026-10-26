# Module 6 — Déboguer : les tickets de `sentinelle-turbo`

Un collègue a écrit `sentinelle-turbo` (`bugs/`), « la version rapide de Sentinelle ».
Elle a déclenché cinq tickets. Pour chacun : reproduire, localiser **avec le débogueur**
(pas en relisant tout le code au hasard), expliquer la cause, corriger, vérifier.

```shell
cargo build -p sentinelle-turbo                    # binaire de mise au point : symboles, contrôles de débordement
rust-gdb --args target/debug/sentinelle-turbo report data     # Linux
rust-lldb -- target/debug/sentinelle-turbo report data        # macOS
```

## TP 1 — Le débogueur sur des plantages (2 h)

### Aide-mémoire

| Action | rust-gdb | rust-lldb |
|---|---|---|
| s'arrêter à toute panique | `break rust_panic` | `b rust_panic` |
| lancer | `run` | `run` |
| pile d'appels | `bt` | `bt` |
| remonter dans la pile | `frame 5` | `frame select 5` |
| variables locales | `info locals` | `frame variable` |
| afficher une valeur | `print line` | `v line` (`frame variable line`) |
| point d'arrêt à une ligne | `break report.rs:17` | `b report.rs:17` |
| … sous condition | `break report.rs:17 if open > 30` | `b report.rs:17 -c 'open > 30'` |
| continuer | `continue` | `continue` |

Les scripts `rust-gdb` / `rust-lldb` chargent des afficheurs adaptés : `String`, `Vec`,
`HashMap`, `Option` s'affichent lisiblement. Hors débogueur, `RUST_BACKTRACE=1` donne la
pile d'une panique, et `dbg!(expr)` affiche une valeur avec son emplacement.

L'arrêt sur `rust_panic` se fait dans la bibliothèque standard, livrée compilée avec
optimisations : d'où l'avertissement « compiled with optimization », qui ne concerne pas
votre code. `bt`, puis remontez la pile jusqu'à la première trame `sentinelle_turbo::…`.
Sous macOS, lldb n'a pas de support natif de Rust (« no plugin for the language rust ») :
il lit les variables mais n'évalue pas d'expression Rust. Préférez `v` / `frame variable`
à `p`, et n'appelez pas de méthode (`line.len()`) depuis le débogueur.

### Ticket 1 — « Aucune alerte en release, plantage en debug »

> Le FAI a mesuré 5,5 Go sortis vendredi vers 3 h. `sentinelle-turbo report`, compilé en
> release, ne lève aucune alerte. Compilé en debug, il plante.

```shell
cargo run --release -p sentinelle-gen -- --out data-propre --malformed 0
cargo run --release -p sentinelle-turbo -- report data-propre
cargo run -p sentinelle-turbo -- report data-propre
```

Pourquoi deux comportements ? Lequel est le plus dangereux ? Que propose la bibliothèque
standard pour choisir explicitement (`checked_add`, `saturating_add`, `wrapping_add`) ?

### Ticket 2 — « Plantage sur le jeu du SOC »

```shell
cargo run --release -p sentinelle-turbo -- report data
```

Le message donne la ligne de code, pas la ligne du journal. Avec un point d'arrêt
conditionnel, trouvez **quelle ligne du journal** déclenche la panique et affichez-la.
Corrigez pour qu'une ligne tronquée soit ignorée, sans jamais trancher hors limites.

### Ticket 3 — « Plantage sur les journaux récupérés d'un disque abîmé »

```shell
cargo run --release -p sentinelle-gen -- --out data-abime --invalid-utf8 --malformed 0.01
cargo run --release -p sentinelle-turbo -- report data-abime
```

Quelle fonction échoue, et pourquoi le fichier entier est-il refusé pour quelques
octets ? Trouvez la position du premier octet invalide (`Utf8Error::valid_up_to`).
Corrigez pour ignorer seulement les lignes invalides.

## TP 2 — Bugs de concurrence (1 h 30 à 2 h)

### Ticket 4 — « `stats` ne rend jamais la main »

```shell
cargo run --release -p sentinelle-turbo -- stats data --threads 1   # fonctionne
cargo run --release -p sentinelle-turbo -- stats data --threads 8   # ne se termine pas
```

Lancez sous le débogueur, laissez bloquer, interrompez (`Ctrl-C`), puis
`thread apply all bt` (gdb) ou `bt all` (lldb). Que fait chaque thread ? Dessinez qui
détient quel verrou et qui attend lequel. Corrigez de deux façons : un ordre de
verrouillage unique, puis une structure qui n'a besoin que d'un verrou.

### Ticket 5 — « `stream` affiche ses blocs puis se fige »

```shell
cargo run --release -p sentinelle-turbo -- stream data
```

Tous les blocs sont agrégés, puis plus rien. `info threads` / `thread list` : combien de
threads restent, et qu'attendent-ils ? Rappel : un `Receiver` itéré par `for` attend tant
qu'**un** `Sender` existe encore.

### Ticket bonus — « L'adresse de la force brute n'apparaît pas dans `stats` »

Avec `--threads 1`, `stats` fonctionne… mais l'adresse qui a fait 445 échecs a disparu du
classement. Ce n'est pas un plantage : c'est la logique. Trouvez la règle responsable
et dites pourquoi elle est dangereuse pour un outil de détection.

## Diapositives

Déboguer du code Rust : « Avant le débogueur : dbg!, Debug et assertions », « Anatomie d'un
panic! », « Compiler pour déboguer », « rust-gdb / rust-lldb », « Inspecter des valeurs
Rust avec rust-lldb », « Déboguer un programme multi-thread : l'interblocage », « Bugs de
concurrence typiques en Rust ». Multi-threading : « mpsc::channel : le cycle de vie d'un
channel », « « Sans peur » ne veut pas dire « sans deadlock » ». Chaînes de caractères
(ticket 3) : « Des octets au texte : l'UTF-8 aux frontières ».
