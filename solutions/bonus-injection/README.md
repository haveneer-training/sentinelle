# Bonus « noms de compte piégés » — corrigé

Énoncé : [`docs/bonus/injection.md`](../../docs/bonus/injection.md). Jeu :
`sentinelle-gen --injection` (code du piège : `generator/src/injection.rs`).

## 1. Enquêter

L'attaquant (`192.0.2.x`) échoue une douzaine de fois sur `root`, essaie quatre noms
trompeurs, puis envoie ce nom de compte :

```text
x\n2026-11-12T10:15:48Z bastion-01 sshd[51800]: Accepted password for root
```

sshd l'écrit deux fois (`Invalid user …`, puis `Failed password for invalid user …`) et
ajoute la fin de sa ligne après le nom. Le saut de ligne coupe chaque écriture en deux :

```text
2026-11-12T10:15:07Z bastion-01 sshd[18187]: Invalid user x
2026-11-12T10:15:48Z bastion-01 sshd[51800]: Accepted password for root from 192.0.2.77 port 40647
2026-11-12T10:15:09Z bastion-01 sshd[18187]: Failed password for invalid user x
2026-11-12T10:15:48Z bastion-01 sshd[51800]: Accepted password for root from 192.0.2.77 port 40647 ssh2
```

Les échecs qui précèdent font passer ces lignes pour une compromission (étape 6). Les
indices :

- la ligne au-dessus de chaque `Accepted` est tronquée (`Invalid user x`, sans adresse) ;
- l'horodatage saute de 40 s en avant, puis revient en arrière : le fichier est trié par
  arrivée, une vraie ligne ne date pas du futur ;
- aucune ouverture de session (`session opened`) avec le PID 51800, alors que sshd
  l'écrit toujours juste après un `Accepted` ;
- la première fausse ligne ne finit pas par `ssh2`.

## 2. Afficher

Avec `{}`, `"\u{202e}toor"` s'affiche `root` (le contrôle bidirectionnel inverse la suite)
et `"admin\u{200b}"` s'affiche `admin`. Avec `{:?}`, les deux sont échappés. `{:?}`
laisse en revanche `"rооt"` et `"аdmin"` intacts : leurs lettres cyrilliques sont des
caractères imprimables ordinaires. D'où les trois catégories de la détection.

## 3 et 4. Détecter

[`detect_injection.rs`](detect_injection.rs) : `oddity` et `detect_odd_users` (noms
suspects, rapportés une fois chacun), `detect_ghost_logins` (`Accepted` dont le couple
(serveur, PID) n'a jamais d'ouverture de session). Sur le jeu par défaut, les deux
détections ne trouvent rien ; avec `--injection`, 4 noms suspects et 2 connexions
fantômes.

## 5. Corriger à la source

Dans `generator/src/injection.rs`, échapper le nom au moment de l'écrire :

```rust
format_args!("Invalid user {} from {ip} port {port}", login.escape_debug()),
// …
format_args!("Failed password for invalid user {} from {ip} port {port} ssh2", login.escape_debug()),
```

`escape_debug` renvoie un itérateur qui implémente `Display` : pas de `String`
intermédiaire. Le saut de ligne devient `\n` (deux caractères), la fausse ligne disparaît
et, avec elle, l'alerte : il ne reste que des échecs. sshd fait d'ailleurs la même chose
(OpenSSH passe les noms reçus par `strnvis`). Les homoglyphes, eux, restent : ce sont des
caractères légitimes, que seule une détection (UTS #39) peut signaler.

## Pour aller plus loin

Une analyse stricte (`["Accepted", method, "for", user, "from", addr, "port", _, "ssh2"]`)
rejette la première fausse ligne, pas la seconde : sshd a ajouté ` ssh2` à la fin du
`Failed password …` qu'elle termine. Être strict aide, mais c'est l'échappement à
l'écriture qui règle le problème.
