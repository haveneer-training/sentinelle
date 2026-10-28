# Bonus — Une force brute distribuée

> Module 2 (collections) · 45 à 75 min · à faire quand l'étape 6 est terminée
> (ou `cargo xtask goto 7`)

## L'enquête

Les détections des étapes 5 et 6 comptent les échecs **par adresse**. Un attaquant qui le
sait répartit sa force brute entre les machines d'un botnet : chacune n'essaie que
quelques mots de passe.

Générez un jeu où l'attaquant procède ainsi, puis lancez votre Sentinelle dessus :

```shell
cargo run --release -p sentinelle-gen -- --out data-distributed --distributed
cargo run --release -p sentinelle -- data-distributed
```

Plus de force brute, plus de `COMPROMISSION`. Il reste pourtant un `volume anormal`. Par
où l'attaquant est-il entré ?

## Ce que vous faites

1. **Enquêter.** Partez du compte du `volume anormal`. Retrouvez dans
   `data-distributed/auth.log` sa première connexion inhabituelle, puis ce qui la précède.
   Combien d'échecs sur ce compte ? Depuis combien d'adresses ? Et l'adresse de la
   connexion réussie, combien d'échecs a-t-elle ? Expliquez pourquoi
   `detect_bruteforce`, `detect_spraying` et `detect_compromise` restent muettes.
2. **Détecter la force brute par compte.** Écrivez
   `detect_account_bruteforce(entries, window, threshold, min_ips)` : les rafales d'au
   moins `threshold` échecs sur un **même compte** en moins de `window` secondes, venant
   d'au moins `min_ips` adresses distinctes. Rapportez le nombre d'échecs et d'adresses
   de chaque rafale. Essayez-la aussi sur un gros jeu
   (`--users 2000 --noise 10 --distributed`) : que remarquez-vous ? Corrigez.
3. **Détecter la compromission par compte.** Écrivez
   `detect_account_compromise(entries, window, min_failures)` : les `Accepted` sur un
   compte qui a cumulé au moins `min_failures` échecs dans les `window` secondes
   précédentes, quelles qu'en soient les adresses. Indiquez si l'adresse de la connexion
   figure parmi celles des échecs. Le rebond qui suit, avec le même compte, ne doit pas
   être rapporté une seconde fois.

## Indices

- `detect_bruteforce` (étape 5) fait déjà tout le travail de fenêtre glissante : seule la
  clé change, le compte au lieu de l'adresse.
- Compter les adresses distinctes d'une fenêtre sans la parcourir : une
  `HashMap<IpAddr, usize>` du nombre d'échecs de chaque adresse dans la fenêtre. On
  incrémente à l'entrée, on décrémente à la sortie, on retire l'adresse quand son compte
  tombe à zéro : `len()` donne alors le nombre d'adresses distinctes.
- Le bruit d'Internet essaie sans cesse `root`, `admin`, `pi`… depuis des centaines
  d'adresses. `AuthEvent::Failed` a un champ `invalid_user` : un compte qui n'existe pas
  ne mènera à aucune connexion.
- Un motif peut fixer un champ : `let AuthEvent::Failed { user, ip, invalid_user: false } = …`.

## Vérifier

`--reveal` écrit dans `GROUND_TRUTH.md` une ligne « Force brute distribuée » (nombre
d'adresses, d'échecs, maximum par adresse) et une section « Bonus ». Ne la lisez qu'après
l'étape 1.

Le corrigé est dans [`solutions/bonus-distributed/`](../../solutions/bonus-distributed/README.md).

## Pour aller plus loin

- Un attaquant patient étale ses essais sur des jours (*low and slow*). Quelle fenêtre
  faudrait-il ? Que coûte-t-elle en mémoire, et en fausses alertes ?
- Le *credential stuffing* essaie un mot de passe connu sur des milliers de comptes,
  chacun depuis une adresse différente. Ni la clé « adresse » ni la clé « compte » ne le
  voient : que compter alors ?
- Bloquer une adresse après N échecs (fail2ban) n'arrête pas un botnet. Bloquer un compte
  après N échecs l'arrête, mais permet à n'importe qui de bloquer n'importe quel compte.
  Quel compromis ?
