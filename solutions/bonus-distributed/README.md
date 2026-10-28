# Bonus « force brute distribuée » — corrigé

Énoncé : [`docs/bonus/distributed.md`](../../docs/bonus/distributed.md). Jeu :
`sentinelle-gen --distributed` (code : la force brute de `generator/src/scenario.rs`).

## 1. Enquêter

Avec la graine 42, le `volume anormal` désigne `l.roux`. Sa première connexion à
bastion-01 est un « accès inédit », depuis `198.51.100.227`. Juste avant, en 15 minutes,
445 échecs sur `l.roux` venant de 100 adresses : au plus 5 chacune, à plusieurs minutes
d'intervalle. Le botnet a trouvé le mot de passe ; l'attaquant s'est connecté depuis sa
propre machine, qui n'a **jamais** échoué, et c'est d'elle que part l'exfiltration.

Pourquoi les détections restent muettes :

| Détection | Seuil | Ce qu'elle voit |
|---|---|---|
| `detect_bruteforce` | 20 échecs d'une même adresse en 5 min | au plus 5 par adresse sur toute l'attaque |
| `detect_spraying` | 20 comptes distincts par adresse | 1 compte par adresse |
| `detect_compromise` | 10 échecs de l'adresse du `Accepted` | 0 : `198.51.100.227` n'a jamais échoué |

Les détections par compte (accès inédit, sudo inhabituel, volume) voient toujours la
suite : on découvre l'exfiltration, mais pas l'entrée.

## 2 et 3. Détecter

[`detect_distributed.rs`](detect_distributed.rs), en deux fonctions qui partagent la
même fenêtre : un `VecDeque<(Timestamp, IpAddr)>` des échecs récents d'un compte, et une
`HashMap<IpAddr, usize>` de leurs adresses, tenue à jour à l'entrée et à la sortie.

- `detect_account_bruteforce` : la logique de `detect_bruteforce`, avec le compte pour
  clé, et le seuil d'adresses distinctes en plus.
- `detect_account_compromise` : à chaque `Accepted`, on retire de la fenêtre du compte ce
  qui a expiré, puis on compare au seuil. Une connexion rapportée vide la fenêtre : le
  rebond vers `srv-web-01`, 13 minutes après avec le même compte, sinon serait rapporté
  à nouveau.

Paramètres utilisés ci-dessous : rafale de 20 échecs en 5 min depuis au moins 5 adresses ;
compromission après 10 échecs dans l'heure.

| Jeu | Rafales par compte | Compromissions par compte |
|---|---|---|
| `--distributed` (défaut) | 1 : `l.roux`, 445 échecs, 100 adresses | `l.roux`, 445 échecs de 100 adresses, **adresse jamais en échec** |
| `--users 2000 --noise 10 --distributed` | 6 : la victime, et 5 sur `root` | la victime seule |
| jeu par défaut, sans l'option | aucune | la victime (1 adresse, en échec) : la détection par compte englobe celle par adresse |

**Le piège de l'étape 2** : sans filtrer `invalid_user`, le jeu officiel
(`--users 20000 --noise 50`) donne 1 159 rafales, toutes sur des comptes de scanners
(`pi`, `deploy`, `oracle`, `admin`…), essayés par des centaines d'adresses en continu.
Une fois `invalid_user` écarté, il ne reste que `root` (103 rafales) : un compte réel,
mais que personne n'utilise par mot de passe. On peut l'écarter aussi, ou ne retenir que
les comptes qui se sont déjà connectés. La compromission par compte, elle, n'a pas ce
problème : `root` ne réussit jamais (sauf la fausse ligne du bonus `--injection`, que
`detect_ghost_logins` démasque).

## Pour aller plus loin

- *Low and slow* : plus la fenêtre est longue, plus `root` déclenche. Une piste :
  un ratio échecs / succès par compte sur la semaine, comparé à l'habitude du compte.
- *Credential stuffing* : beaucoup de comptes, beaucoup d'adresses, un essai chacun.
  Rien ne dépasse un seuil par clé ; il faut surveiller le taux global d'échecs sur les
  comptes **existants**, ou la proportion de comptes visés.
- Le verrouillage de compte arrête le botnet mais offre un déni de service ciblé : on
  préfère en général un délai croissant par compte, ou un second facteur.
