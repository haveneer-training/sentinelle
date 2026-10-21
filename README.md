# Sentinelle

Fil rouge des travaux pratiques de la formation « Langage Rust — intermédiaire »
(5 jours : références, collections, concurrence, bonnes pratiques, performance, débogage).

## Intention

Les stagiaires reprennent **Sentinelle**, un analyseur de journaux d'une entreprise
fictive (*Orbe Logistique*), et en reconstruisent une version juste, rapide et robuste.
Un scénario d'intrusion est caché dans les journaux synthétiques ; l'outil doit le
mettre au jour, étape par étape :

- lire les journaux sans copie (références, durées de vie) ;
- détecter les attaques avec les bonnes collections ;
- répartir l'analyse sur plusieurs threads ;
- refactoriser, documenter, mesurer et déboguer le code hérité.

Le dépôt contiendra le générateur de journaux, le fil rouge découpé en étapes pilotées
par `cargo xtask`, la v0.1 héritée à reprendre, ainsi que les supports des modules
performance et débogage.

## Licence

CC BY-NC-SA 4.0 — Pascal Havé (contact@haveneer.com)
