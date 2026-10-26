# Formats des journaux

Les deux fichiers sont du texte, une entrée par ligne, triés dans l'ordre de réception
par le serveur de collecte. Les horodatages sont au format ISO 8601 UTC
(`2026-11-09T08:12:45Z`) et proviennent de l'horloge de chaque serveur.

La collecte n'est pas parfaite : quelques lignes peuvent être tronquées, vides ou
amputées d'un champ.

## `auth.log` — authentifications des serveurs

```
TIMESTAMP HOST PROCESS[PID]: MESSAGE
TIMESTAMP HOST PROCESS: MESSAGE
```

Le `[PID]` est absent pour certains processus (`sudo`). Messages rencontrés :

| Processus | Message |
|---|---|
| `sshd` | `Accepted password for USER from IP port PORT ssh2` |
| `sshd` | `Accepted publickey for USER from IP port PORT ssh2` |
| `sshd` | `Failed password for USER from IP port PORT ssh2` |
| `sshd` | `Failed password for invalid user USER from IP port PORT ssh2` |
| `sshd` | `Invalid user USER from IP port PORT` |
| `sshd` | `error: maximum authentication attempts exceeded for USER from IP port PORT ssh2 [preauth]` |
| `sshd` | `Connection closed by IP port PORT [preauth]` |
| `sshd` | `pam_unix(sshd:session): session opened for user USER(uid=UID) by (uid=0)` |
| `sshd` | `pam_unix(sshd:session): session closed for user USER` |
| `CRON` | `pam_unix(cron:session): session opened for user root(uid=0) by (uid=0)` |
| `CRON` | `pam_unix(cron:session): session closed for user root` |
| `sudo` | `  USER : TTY=pts/N ; PWD=DIR ; USER=root ; COMMAND=CMD` |

Une tentative sur un compte inexistant produit deux lignes : `Invalid user …`
puis `Failed password for invalid user …`.

Exemple :

```
2026-11-09T09:41:12Z srv-web-02 sshd[48213]: Accepted publickey for v.michel from 10.1.0.42 port 51022 ssh2
2026-11-09T09:41:50Z srv-web-02 sudo:   v.michel : TTY=pts/6 ; PWD=/home/v.michel ; USER=root ; COMMAND=/usr/bin/df -h
```

## `access.log` — application web interne (`srv-web-01`)

```
IP - USER [TIMESTAMP] "METHOD PATH HTTP/1.1" STATUS BYTES DURATION_MS
```

- `USER` vaut `-` pour une requête non authentifiée.
- `BYTES` (taille de la réponse) vaut `-` quand la réponse n'a pas de corps (`304`).
- `DURATION_MS` : temps de traitement en millisecondes.

Exemple :

```
10.1.0.42 - v.michel [2026-11-09T09:43:02Z] "GET /api/orders?id=48213 HTTP/1.1" 200 1834 44
10.1.0.42 - v.michel [2026-11-09T09:43:05Z] "GET /static/app.js HTTP/1.1" 304 - 2
```

## Le parc

| Serveur | Adresse | Rôle |
|---|---|---|
| `bastion-01` | 10.0.0.10 | seul point d'entrée SSH depuis Internet |
| `srv-web-01` | 10.0.2.11 | frontal de l'application web interne |
| `srv-web-02` | 10.0.2.12 | serveur web secondaire |
| `srv-db-01` | 10.0.3.21 | base des commandes |
| `srv-db-02` | 10.0.3.22 | base des clients (administrateurs uniquement) |
| `srv-files-01` | 10.0.4.31 | partage de fichiers |
| `srv-build-01` | 10.0.5.41 | intégration continue |

Les postes de travail sont dans `10.1.0.0/16`.

Les adresses « Internet » (tout ce qui est hors de `10.0.0.0/8`) sont fictives. Elles
viennent de plages qu'aucun organisme n'attribue à qui que ce soit : les plages réservées
à la documentation (RFC 5737 : `192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`) et
la plage partagée des opérateurs (RFC 6598 : `100.64.0.0/10`, utilisée pour le NAT chez les
fournisseurs d'accès). Aucune ne désigne donc une personne ou une entreprise réelle : un
`whois` ou une géolocalisation n'apprendrait rien, tout ce qu'il y a à savoir sur ces
adresses est dans les journaux.
