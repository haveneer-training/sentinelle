# Module 1 — Références

| Temps | Activité | Support |
|---|---|---|
| cours (3 h) | Exemple TP 1 : références immuables et mutables | `exercices/m1`, exercices 1 à 4 |
| | Exemple TP 2 : durées de vie dans les fonctions | `exercices/m1`, exercices 5 et 6 |
| TP (4 h) | analyse de code réel : lecture critique de la v0.1 | ci-dessous, 45 min |
| | le fil rouge, étapes 0 à 3 | `docs/etapes/`, 2 h à 2 h 40 |
| | bugs de références dans un projet existant | `exercices/m1`, exercices 7 à 9 |

## Lecture critique de `legacy/src/main.rs` (45 min)

Lisez le code **sans le modifier**, puis répondez, en citant les lignes :

1. Combien de fois le journal d'authentification est-il entièrement copié pendant une
   exécution ? Par quelles instructions ? Que faudrait-il changer dans les signatures
   pour qu'aucune copie ne soit nécessaire ?
2. `fn field(parts: &Vec<String>, i: usize) -> String` : quel type de paramètre serait plus
   général ? Pourquoi renvoyer une `String` ici, et que coûterait renvoyer un `&str` ?
3. Pour chaque champ de `AuthEntry`, une `String` est allouée. Lesquels sont lus ensuite ?
4. Le rapport annonce, pour l'adresse du *password spraying*, plus d'échecs qu'elle n'a
   visé de comptes. Trouvez pourquoi (indice : `docs/FORMATS.md`, tentatives sur un
   compte inexistant).
5. Que se passe-t-il pour une ligne amputée d'un champ, par exemple
   `Failed password for from 192.0.2.7 port 22 ssh2` ? Et pour un octet d'UTF-8 invalide
   (`sentinelle-gen --invalid-utf8 --malformed 0.01`) ?

Gardez vos réponses : le fil rouge corrige chacun de ces points, et le module 4
refactorisera ce même code.
