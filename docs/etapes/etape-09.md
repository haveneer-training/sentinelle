# Étape 9 — Un pipeline de threads

> Module 3 (Concurrence) · 60 à 80 min · `sentinelle/src/summary.rs`

## L'enquête

Charger 800 Mo d'un coup n'est pas toujours possible. Un pipeline lit le flux par blocs,
les distribue à des travailleurs, et un agrégateur fusionne leurs résumés : la mémoire
reste bornée.

## Ce que vous écrivez

`summarize_pipeline(reader, workers, lines_per_block) -> io::Result<Summary<String>>` :

- un thread lecteur appelle `read_blocks` (fournie), qui envoie des `Vec<u8>` ;
- `workers` threads reçoivent les blocs et les résument ;
- le thread appelant fusionne les résumés partiels.

## Indices

- `mpsc::sync_channel(n)` est **borné** : le lecteur attend quand `n` blocs sont en
  attente (contre-pression). `mpsc::channel()` ne l'est pas.
- Un `Receiver` n'a qu'un propriétaire et n'est pas partageable tel quel : les
  travailleurs se le partagent derrière un `Mutex`. Ne gardez le verrou **que** le temps
  du `recv()`, pas pendant l'analyse.
- `for partial in receiver` ne s'arrête que lorsque **tous** les `Sender` sont détruits.
  Chaque travailleur a son clone ; et l'original ?
- Pourquoi `Summary<String>` et plus `Summary<&str>` ? Que deviennent les blocs ?

## Vérifier

```shell
cargo test --test step9_pipeline
```

Le test `empty_input_terminates` échoue par blocage si un canal reste ouvert.

## Diapositives

« Le concept de channel », « mpsc::channel : le cycle de vie d'un channel »,
« mpsc::sync_channel : channel borné avec back-pressure », « Mutex<T> : la donnée est dans
le verrou », « Verrou et empoisonnement ».
