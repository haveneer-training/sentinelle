//! Agrégats fusionnables : chaque morceau du journal se résume indépendamment, puis les
//! résumés se fusionnent. C'est ce qui permet de répartir le travail entre threads.
//!
//! Les détections à fenêtre glissante (force brute) ou à état (compromission) ne se
//! découpent pas ainsi : elles restent séquentielles.

use crate::ParseError;
use crate::auth::{AuthEvent, parse_auth_line};
use crate::journal::lines;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::{self, BufRead};
use std::net::IpAddr;
use std::ops::Range;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
#[cfg_attr(not(feature = "step10"), allow(unused_imports))] // trou étape 10
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
#[cfg_attr(not(feature = "step9"), allow(unused_imports))] // trous étapes 9 et 10
use std::sync::{Mutex, mpsc};
#[cfg_attr(not(feature = "step8"), allow(unused_imports))] // trous étapes 8 à 10
use std::thread;
use std::time::Duration;

/// Résumé de `auth.log`.
///
/// `U` est le type des noms de compte : `&'a str` quand le texte vit assez longtemps pour
/// être emprunté, `String` quand il est détruit avant la fin de l'analyse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary<U: Eq + Hash> {
    pub lines: usize,
    pub errors: usize,
    pub failures_by_ip: HashMap<IpAddr, usize>,
    pub accounts_by_ip: HashMap<IpAddr, HashSet<U>>,
}

// pas de `#[derive(Default)]` : il exigerait `U: Default`, inutile ici
impl<U: Eq + Hash> Default for Summary<U> {
    fn default() -> Self {
        Self { lines: 0, errors: 0, failures_by_ip: HashMap::new(), accounts_by_ip: HashMap::new() }
    }
}

impl<U: Eq + Hash> Summary<U> {
    /// Ajoute une ligne brute de `auth.log` au résumé.
    pub fn record<'b>(&mut self, line: &'b [u8])
    where
        U: From<&'b str>,
    {
        self.lines += 1;
        let parsed = std::str::from_utf8(line).map_err(|_| ParseError::InvalidUtf8).and_then(parse_auth_line);
        match parsed {
            Ok(entry) => {
                if let AuthEvent::Failed { user, ip, .. } = entry.event {
                    *self.failures_by_ip.entry(ip).or_insert(0) += 1;
                    self.accounts_by_ip.entry(ip).or_default().insert(U::from(user));
                }
            }
            Err(_) => self.errors += 1,
        }
    }

    /// Fusionne `other` dans ce résumé.
    #[cfg_attr(not(feature = "step8"), allow(unused_variables))] // trou étape 8
    pub fn merge(&mut self, other: Self) {
        // TODO-STEP:8 Additionner les compteurs, et réunir les ensembles de comptes adresse
        // par adresse. `other` est pris par valeur : ses ensembles peuvent être déplacés,
        // pas copiés.
        // SOLUTION-BEGIN
        self.lines += other.lines;
        self.errors += other.errors;
        for (ip, failures) in other.failures_by_ip {
            *self.failures_by_ip.entry(ip).or_insert(0) += failures;
        }
        for (ip, accounts) in other.accounts_by_ip {
            self.accounts_by_ip.entry(ip).or_default().extend(accounts);
        }
        // SOLUTION-END
    }
}

/// Résumé calculé séquentiellement : la référence des versions parallèles.
pub fn summarize(text: &[u8]) -> Summary<&str> {
    let mut summary = Summary::default();
    for line in lines(text) {
        summary.record(line);
    }
    summary
}

/// Découpe `text` en au plus `parts` morceaux de tailles voisines, chacun finissant par
/// une fin de ligne (sauf peut-être le dernier). Aucun octet n'est perdu ni dupliqué.
#[cfg_attr(not(feature = "step8"), allow(unused_variables))] // trou étape 8
pub fn split_chunks(text: &[u8], parts: usize) -> Vec<&[u8]> {
    // TODO-STEP:8 Viser `text.len() / parts` octets par morceau, puis avancer la coupure
    // jusqu'au `\n` suivant (`position`) pour ne jamais couper une ligne en deux.
    // `split_at` rend le morceau et le reste, tous deux empruntés à `text`.
    // SOLUTION-BEGIN
    let target = text.len().div_ceil(parts.max(1)).max(1);
    let mut chunks = Vec::with_capacity(parts);
    let mut rest = text;
    while !rest.is_empty() {
        let cut = match rest.get(target..).and_then(|tail| tail.iter().position(|&b| b == b'\n')) {
            Some(newline) => target + newline + 1,
            None => rest.len(),
        };
        let (chunk, tail) = rest.split_at(cut);
        chunks.push(chunk);
        rest = tail;
    }
    chunks
    // SOLUTION-END
}

/// Résumé calculé par `threads` threads, chacun sur un morceau du texte.
///
/// Le texte n'est pas copié : les threads l'empruntent, ce que permet `thread::scope`
/// (tous les threads sont terminés avant que `text` puisse disparaître).
#[cfg_attr(not(feature = "step8"), allow(unused_variables))] // trou étape 8
pub fn summarize_scoped(text: &[u8], threads: usize) -> Summary<&str> {
    // TODO-STEP:8 Dans un `thread::scope`, lancer un thread par morceau de `split_chunks`
    // (chacun appelle `summarize`), puis fusionner les résultats des `join()`.
    // SOLUTION-BEGIN
    let chunks = split_chunks(text, threads);
    thread::scope(|scope| {
        let handles: Vec<_> = chunks.into_iter().map(|chunk| scope.spawn(move || summarize(chunk))).collect();
        let mut total = Summary::default();
        for handle in handles {
            total.merge(handle.join().expect("un thread d'analyse a paniqué"));
        }
        total
    })
    // SOLUTION-END
}

/// Lit `reader` par blocs d'au plus `lines_per_block` lignes complètes et les envoie
/// dans `blocks`, jusqu'à la fin du flux.
#[cfg_attr(not(feature = "step9"), allow(dead_code))] // à appeler dans le trou de l'étape 9
fn read_blocks(mut reader: impl BufRead, lines_per_block: usize, blocks: SyncSender<Vec<u8>>) -> io::Result<()> {
    loop {
        let mut block = Vec::new();
        for _ in 0..lines_per_block {
            if reader.read_until(b'\n', &mut block)? == 0 {
                break;
            }
        }
        // fin du flux, ou plus personne pour recevoir
        if block.is_empty() || blocks.send(block).is_err() {
            return Ok(());
        }
    }
}

/// Pipeline : un thread lecteur découpe le flux en blocs possédés (`Vec<u8>`) qu'il
/// envoie dans un canal borné ; `workers` threads les résument ; le thread appelant
/// fusionne les résumés partiels.
///
/// Le flux n'est jamais entièrement en mémoire. Les blocs sont détruits après analyse :
/// les noms de compte doivent donc être copiés (`Summary<String>`).
#[cfg_attr(not(feature = "step9"), allow(unused_variables))] // trou étape 9
pub fn summarize_pipeline<R: BufRead + Send>(reader: R, workers: usize, lines_per_block: usize) -> io::Result<Summary<String>> {
    // TODO-STEP:9 Deux canaux : `mpsc::sync_channel` (borné) pour les blocs, `mpsc::channel`
    // pour les résumés partiels. Un `Receiver` n'a qu'un seul propriétaire : les travailleurs
    // se partagent celui des blocs derrière un `Mutex` (avec `thread::spawn`, il faudrait en
    // plus un `Arc`). Le lecteur appelle `read_blocks`. Attention : l'agrégation par
    // `for partial in receiver` ne se termine que lorsque **tous** les émetteurs sont détruits.
    // SOLUTION-BEGIN
    let workers = workers.max(1);
    let (block_tx, block_rx) = mpsc::sync_channel::<Vec<u8>>(2 * workers);
    let block_rx = Mutex::new(block_rx);
    let (summary_tx, summary_rx) = mpsc::channel::<Summary<String>>();
    thread::scope(|scope| {
        for _ in 0..workers {
            let (block_rx, summary_tx) = (&block_rx, summary_tx.clone());
            scope.spawn(move || {
                let mut summary = Summary::default();
                loop {
                    // le verrou n'est tenu que le temps de recevoir un bloc, pas de l'analyser
                    let received = block_rx.lock().expect("verrou empoisonné").recv();
                    let Ok(block) = received else {
                        break;
                    };
                    for line in lines(&block) {
                        summary.record(line);
                    }
                }
                // le canal ne peut être fermé ici : le thread appelant attend nos résumés
                summary_tx.send(summary).expect("agrégateur disparu");
            });
        }
        // sans ce `drop`, l'agrégation attendrait indéfiniment le dernier émetteur
        drop(summary_tx);
        let reader = scope.spawn(move || read_blocks(reader, lines_per_block.max(1), block_tx));
        let mut total = Summary::default();
        for partial in summary_rx {
            total.merge(partial);
        }
        reader.join().expect("le thread de lecture a paniqué")?;
        Ok(total)
    })
    // SOLUTION-END
}

// ------------------------------------------------------------------ étape 10 (bonus)

/// Les morceaux de [`split_chunks`], sous forme de plages d'octets.
///
/// Une plage ne contient pas de référence : elle peut partir dans un thread lancé par
/// `thread::spawn`, contrairement à une tranche empruntée.
pub fn chunk_ranges(text: &[u8], parts: usize) -> Vec<Range<usize>> {
    let mut start = 0;
    split_chunks(text, parts)
        .iter()
        .map(|chunk| {
            let range = start..start + chunk.len();
            start = range.end;
            range
        })
        .collect()
}

/// Variante à **état partagé** : chaque thread enregistre ses lignes directement dans un
/// unique résumé protégé par un `Mutex`, au lieu de fusionner des résumés partiels.
///
/// Les threads sont lancés par `thread::spawn` : ils pourraient survivre à cette
/// fonction, ils ne peuvent donc rien emprunter. Le texte et le résumé sont partagés par
/// `Arc`, et les noms de compte possédés (`String`).
#[cfg_attr(not(feature = "step10"), allow(unused_variables))] // trou étape 10
pub fn summarize_shared(text: Arc<Vec<u8>>, threads: usize) -> Summary<String> {
    // TODO-STEP:10 Un `Arc<Mutex<Summary<String>>>` partagé ; pour chaque plage de
    // `chunk_ranges`, cloner les deux `Arc` et lancer un `thread::spawn(move || …)` qui
    // verrouille le résumé à chaque ligne. Après les `join()`, récupérer le résumé avec
    // `Arc::try_unwrap` puis `Mutex::into_inner`.
    // SOLUTION-BEGIN
    let shared = Arc::new(Mutex::new(Summary::default()));
    let handles: Vec<_> = chunk_ranges(&text, threads)
        .into_iter()
        .map(|range| {
            let (text, shared) = (Arc::clone(&text), Arc::clone(&shared));
            thread::spawn(move || {
                for line in lines(&text[range]) {
                    // un verrou par ligne : c'est précisément ce coût qu'on veut mesurer
                    shared.lock().expect("verrou empoisonné").record(line);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("un thread d'analyse a paniqué");
    }
    let mutex = Arc::try_unwrap(shared).unwrap_or_else(|_| unreachable!("tous les threads sont terminés"));
    mutex.into_inner().expect("verrou empoisonné")
    // SOLUTION-END
}

/// Résumé parallèle, comme [`summarize_scoped`], qui signale sa progression.
///
/// Un thread dédié appelle `report` toutes les `period` avec le nombre de lignes déjà
/// traitées ; un dernier appel, une fois tout terminé, donne le total. Les valeurs
/// transmises ne décroissent jamais.
#[cfg_attr(not(feature = "step10"), allow(unused_variables))] // trou étape 10
pub fn summarize_with_progress(text: &[u8], threads: usize, period: Duration, report: impl Fn(usize) + Sync) -> Summary<&str> {
    // TODO-STEP:10 Un `AtomicUsize` incrémenté par les threads d'analyse (`fetch_add`), un
    // `AtomicBool` d'arrêt pour le thread rapporteur (`load` / `store`, `thread::sleep`).
    // Tout tient dans un `thread::scope`. Pour que le dernier appel soit bien le total,
    // attendre la fin du rapporteur avant de l'émettre.
    // SOLUTION-BEGIN
    let processed = AtomicUsize::new(0);
    let done = AtomicBool::new(false);
    thread::scope(|scope| {
        let reporter = scope.spawn(|| {
            while !done.load(Ordering::Acquire) {
                report(processed.load(Ordering::Relaxed));
                thread::sleep(period);
            }
        });
        let processed = &processed;
        let handles: Vec<_> = split_chunks(text, threads)
            .into_iter()
            .map(|chunk| {
                scope.spawn(move || {
                    let mut summary = Summary::default();
                    for line in lines(chunk) {
                        summary.record(line);
                        processed.fetch_add(1, Ordering::Relaxed);
                    }
                    summary
                })
            })
            .collect();
        let mut total = Summary::default();
        for handle in handles {
            total.merge(handle.join().expect("un thread d'analyse a paniqué"));
        }
        done.store(true, Ordering::Release);
        reporter.join().expect("le thread rapporteur a paniqué");
        report(processed.load(Ordering::Relaxed));
        total
    })
    // SOLUTION-END
}
