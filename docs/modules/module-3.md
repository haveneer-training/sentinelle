# Module 3 — Concurrence

| Temps | Activité | Support |
|---|---|---|
| cours (3 h) | Exemple TP 1 : créer des threads, les synchroniser | `exercices/m3/ex01_threads` |
| | Exemple TP 2 : canaux `mpsc`, envoi et réception | `exercices/m3/ex02_mpsc` |
| TP (4 h) | présentation de programmes concurrents (formateur) | diapositives, 30 min |
| | tâches concurrentes dans un programme existant | étapes 8 et 9, 1 h 45 à 2 h 20 |
| | pour aller plus loin : état partagé, atomiques, mesures | étape 10 (bonus), 1 h 10 à 1 h 40 |

Le fil rouge est le « programme existant » : ses analyses séquentielles des étapes 4 à 7
deviennent parallèles, sans changer de résultat (les tests le vérifient).
