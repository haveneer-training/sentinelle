//! Les détections. Chacune repose sur la collection qui lui convient.
//!
//! Les entrées sont supposées dans l'ordre du fichier, c'est-à-dire l'ordre de réception :
//! à quelques secondes près (et à la dérive d'horloge d'un serveur près), l'ordre
//! chronologique.

use crate::auth::AuthEntry;
#[cfg_attr(not(feature = "step4"), allow(unused_imports))] // trous étapes 4 à 6
use crate::auth::AuthEvent;
#[cfg_attr(not(feature = "step7"), allow(unused_imports))] // trou étape 7
use crate::time::HOUR;
use crate::time::{DAY, Iso, MINUTE, Timestamp};
use crate::web::WebEntry;
use std::collections::{BTreeMap, HashMap};
#[cfg_attr(not(feature = "step5"), allow(unused_imports))] // trous étapes 5 et 6
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::net::IpAddr;
#[cfg_attr(not(feature = "step4"), allow(unused_imports))] // trou étape 4
use std::{cmp::Reverse, collections::BinaryHeap};

// ------------------------------------------------------------------ étape 4

/// Nombre d'échecs d'authentification par adresse.
///
/// Une tentative sur un compte inexistant produit deux lignes (`Invalid user …` puis
/// `Failed password for invalid user …`) : elle ne doit compter qu'une fois. On compte
/// donc les seuls `Failed`, présents pour toute tentative, et on ignore `InvalidUser`.
#[cfg_attr(not(feature = "step4"), allow(unused_variables))] // trou étape 4
pub fn failures_by_ip(entries: &[AuthEntry<'_>]) -> HashMap<IpAddr, usize> {
    // TODO-STEP:4 Compter les `AuthEvent::Failed` par adresse, avec l'API `entry` (une
    // seule recherche dans la table par ligne).
    // SOLUTION-BEGIN
    let mut counts = HashMap::new();
    for entry in entries {
        if let AuthEvent::Failed { ip, .. } = entry.event {
            *counts.entry(ip).or_insert(0) += 1;
        }
    }
    counts
    // SOLUTION-END
}

/// Les `n` couples de plus grande valeur, par valeur décroissante puis clé croissante.
///
/// Accepte toute source de couples : `top_n(&table, 10)` sur une `HashMap` renvoie des
/// couples de références.
#[cfg_attr(not(feature = "step4"), allow(unused_variables, unused_mut))] // trou étape 4
pub fn top_n<K: Ord, V: Ord>(items: impl IntoIterator<Item = (K, V)>, n: usize) -> Vec<(K, V)> {
    // TODO-STEP:4 Sans trier toute la table : garder les `n` meilleurs dans un `BinaryHeap`
    // de taille bornée. `BinaryHeap` est un tas-max ; `std::cmp::Reverse` en fait un tas-min
    // dont la racine est le moins bon des retenus, celui qu'on éjecte.
    // SOLUTION-BEGIN
    if n == 0 {
        return Vec::new();
    }
    // à valeur égale, la plus petite clé l'emporte : d'où le `Reverse(key)` ;
    // le `Reverse(…)` extérieur fait un tas-min : `pop` éjecte le moins bon des retenus
    let mut heap = BinaryHeap::with_capacity(n + 1);
    for (key, value) in items {
        heap.push(Reverse((value, Reverse(key))));
        if heap.len() > n {
            heap.pop();
        }
    }
    // tri croissant des `Reverse(…)`, donc du meilleur au moins bon
    heap.into_sorted_vec()
        .into_iter()
        .map(|Reverse((value, Reverse(key)))| (key, value))
        .collect()
    // SOLUTION-END
}

// ------------------------------------------------------------------ étape 5

/// Échecs sur de nombreux comptes distincts depuis une même adresse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spraying {
    pub ip: IpAddr,
    pub accounts: usize,
    pub first: Timestamp,
    pub last: Timestamp,
}

/// Adresses ayant échoué sur au moins `min_accounts` comptes **distincts**, par nombre de
/// comptes décroissant (puis adresse croissante).
#[cfg_attr(not(feature = "step5"), allow(unused_variables))] // trou étape 5
pub fn detect_spraying(entries: &[AuthEntry<'_>], min_accounts: usize) -> Vec<Spraying> {
    // TODO-STEP:5 Associer à chaque adresse l'ensemble (`HashSet<&str>`) des comptes visés,
    // ainsi que le premier et le dernier instant ; garder les adresses au-dessus du seuil.
    // SOLUTION-BEGIN
    let mut by_ip: HashMap<IpAddr, (HashSet<&str>, Timestamp, Timestamp)> = HashMap::new();
    for entry in entries {
        if let AuthEvent::Failed { user, ip, .. } = entry.event {
            let ts = entry.timestamp;
            let (accounts, first, last) =
                by_ip.entry(ip).or_insert_with(|| (HashSet::new(), ts, ts));
            accounts.insert(user);
            *first = (*first).min(ts);
            *last = (*last).max(ts);
        }
    }
    let mut found: Vec<Spraying> = by_ip
        .into_iter()
        .filter(|(_, (accounts, _, _))| accounts.len() >= min_accounts)
        .map(|(ip, (accounts, first, last))| Spraying {
            ip,
            accounts: accounts.len(),
            first,
            last,
        })
        .collect();
    found.sort_by(|a, b| b.accounts.cmp(&a.accounts).then(a.ip.cmp(&b.ip)));
    found
    // SOLUTION-END
}

/// Rafale d'échecs depuis une même adresse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BruteForce {
    pub ip: IpAddr,
    /// Le plus ancien échec de la fenêtre qui a ouvert la rafale.
    pub first: Timestamp,
    /// Le dernier échec où la fenêtre atteignait le seuil : pas forcément le dernier échec
    /// de l'adresse, ceux qui suivent sous le seuil ne prolongent pas la rafale.
    pub last: Timestamp,
    /// Le nombre exact d'échecs de l'adresse de `first` à `last` inclus.
    pub failures: usize,
}

/// Rafales d'au moins `threshold` échecs d'une même adresse en moins de `window` secondes.
///
/// Une rafale se prolonge tant que le seuil reste atteint ; elle n'est rapportée qu'une
/// fois. Résultat par ordre chronologique de début.
///
/// `entries` doit être dans l'ordre chronologique. `failures` compte tous les échecs de
/// `first` à `last`, y compris ceux arrivés pendant que la fenêtre était passée sous le
/// seuil, avant que la rafale soit prolongée.
#[cfg_attr(not(feature = "step5"), allow(unused_variables))] // trou étape 5
pub fn detect_bruteforce(
    entries: &[AuthEntry<'_>],
    window: i64,
    threshold: usize,
) -> Vec<BruteForce> {
    // TODO-STEP:5 Fenêtre glissante par adresse : un `VecDeque<Timestamp>` des échecs
    // récents (on ajoute à droite, on retire à gauche ce qui est sorti de la fenêtre).
    // Quand la fenêtre atteint le seuil, ouvrir ou prolonger la rafale en cours de
    // l'adresse ; une rafale interrompue depuis plus de `window` est terminée.
    // SOLUTION-BEGIN
    // par adresse : les instants des échecs encore dans la fenêtre, du plus ancien au
    // plus récent (les entrées arrivent dans l'ordre chronologique)
    let mut windows: HashMap<IpAddr, VecDeque<Timestamp>> = HashMap::new();
    // par adresse : la rafale en cours, pas encore rapportée
    let mut open: HashMap<IpAddr, BruteForce> = HashMap::new();
    let mut found = Vec::new();
    for entry in entries {
        let AuthEvent::Failed { ip, .. } = entry.event else {
            continue;
        };
        let now = entry.timestamp;
        let recent = windows.entry(ip).or_default();
        // l'échec entre à droite ; à gauche sortent ceux d'il y a `window` secondes ou plus
        recent.push_back(now);
        while recent.front().is_some_and(|&first| now - first >= window) {
            recent.pop_front();
        }
        if recent.len() < threshold {
            // sous le seuil : ni ouverture ni prolongation ; l'échec reste dans `recent`
            // et sera compté si la rafale en cours est prolongée
            continue;
        }
        // seuil atteint : prolonger la rafale en cours si elle est assez récente,
        // sinon en ouvrir une nouvelle
        match open.get_mut(&ip) {
            Some(burst) if now - burst.last < window => {
                // tous les échecs depuis la fin connue de la rafale, y compris ceux passés
                // sous le seuil : ils sont encore dans `recent` (trié), puisque
                // `now - burst.last < window` (cas limite : test
                // `a_burst_counts_the_failures_below_the_threshold`, étape 5)
                burst.failures += recent.len() - recent.partition_point(|&t| t <= burst.last);
                burst.last = now;
            }
            _ => {
                // la rafale démarre au plus ancien échec de la fenêtre, pas à `now`
                let burst = BruteForce {
                    ip,
                    first: recent[0],
                    last: now,
                    failures: recent.len(),
                };
                // `insert` rend l'éventuelle rafale précédente, terminée : `Option` est
                // itérable, `extend` l'ajoute aux résultats s'il y en a une
                found.extend(open.insert(ip, burst));
            }
        }
    }
    // les rafales encore ouvertes en fin de journal sont terminées elles aussi
    found.extend(open.into_values());
    found.sort_by_key(|burst| (burst.first, burst.ip));
    found
    // SOLUTION-END
}

// ------------------------------------------------------------------ étape 6

/// Connexion réussie depuis une adresse qui venait d'échouer de nombreuses fois.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compromise<'a> {
    pub ip: IpAddr,
    /// Le compte connecté.
    pub user: &'a str,
    /// Le serveur de la connexion réussie.
    pub host: &'a str,
    /// L'instant de la connexion réussie.
    pub at: Timestamp,
    /// Les échecs de `ip` avant cette connexion, depuis le début du journal.
    pub failures_before: usize,
}

/// Connexions réussies (`Accepted`) depuis une adresse ayant *déjà* cumulé au moins
/// `min_failures` échecs.
///
/// - Un échec est un `Failed` (comme pour [`failures_by_ip`], `InvalidUser` est ignoré),
///   quels que soient le compte et le serveur visés.
/// - « *déjà* » : seuls comptent les échecs placés **avant** la connexion dans `entries`,
///   supposé dans l'ordre chronologique. Pas de fenêtre de temps, et le compte n'est
///   jamais remis à zéro, même après un succès.
/// - Chaque connexion qui passe le seuil est rapportée, pas seulement la première de
///   l'adresse. Résultat dans l'ordre de `entries`.
#[cfg_attr(not(feature = "step6"), allow(unused_variables))] // trou étape 6
pub fn detect_compromise<'a>(
    entries: &[AuthEntry<'a>],
    min_failures: usize,
) -> Vec<Compromise<'a>> {
    // TODO-STEP:6 Un seul passage : compter les échecs par adresse au fil de l'eau et,
    // à chaque `Accepted`, consulter le compte de son adresse.
    // SOLUTION-BEGIN
    let mut failures: HashMap<IpAddr, usize> = HashMap::new();
    let mut found = Vec::new();
    for entry in entries {
        match entry.event {
            AuthEvent::Failed { ip, .. } => *failures.entry(ip).or_insert(0) += 1,
            AuthEvent::Accepted { user, ip, .. } => {
                let failures_before = failures.get(&ip).copied().unwrap_or(0);
                if failures_before >= min_failures {
                    found.push(Compromise {
                        ip,
                        user,
                        host: entry.host,
                        at: entry.timestamp,
                        failures_before,
                    });
                }
            }
            _ => {}
        }
    }
    found
    // SOLUTION-END
}

/// Première connexion d'un utilisateur sur un serveur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAccess<'a> {
    pub user: &'a str,
    pub host: &'a str,
    pub ip: IpAddr,
    pub at: Timestamp,
}

/// Connexions réussies sur un couple (utilisateur, serveur) jamais vu pendant la période
/// d'apprentissage (les entrées antérieures à `learn_until`). Chaque nouveau couple n'est
/// rapporté qu'une fois.
#[cfg_attr(not(feature = "step6"), allow(unused_variables))] // trou étape 6
pub fn detect_new_access<'a>(
    entries: &[AuthEntry<'a>],
    learn_until: Timestamp,
) -> Vec<NewAccess<'a>> {
    // TODO-STEP:6 Un `HashSet<(&str, &str)>` des couples connus : `insert` dit en même
    // temps si le couple était nouveau.
    // SOLUTION-BEGIN
    let mut known: HashSet<(&str, &str)> = HashSet::new();
    let mut found = Vec::new();
    for entry in entries {
        let AuthEvent::Accepted { user, ip, .. } = entry.event else {
            continue;
        };
        if known.insert((user, entry.host)) && entry.timestamp >= learn_until {
            found.push(NewAccess {
                user,
                host: entry.host,
                ip,
                at: entry.timestamp,
            });
        }
    }
    found
    // SOLUTION-END
}

/// Commande `sudo` d'un compte qui n'en lance pas d'habitude.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnusualSudo<'a> {
    pub user: &'a str,
    pub host: &'a str,
    pub command: &'a str,
    pub at: Timestamp,
}

/// Commandes `sudo` lancées après `learn_until` par des comptes qui n'en ont lancé aucune
/// pendant la période d'apprentissage.
#[cfg_attr(not(feature = "step6"), allow(unused_variables))] // trou étape 6
pub fn detect_unusual_sudo<'a>(
    entries: &[AuthEntry<'a>],
    learn_until: Timestamp,
) -> Vec<UnusualSudo<'a>> {
    // TODO-STEP:6 Deux passages : l'ensemble des comptes habitués à `sudo`, puis les
    // commandes des autres.
    // SOLUTION-BEGIN
    let sudoers: HashSet<&str> = entries
        .iter()
        .filter(|entry| entry.timestamp < learn_until)
        .filter_map(|entry| match entry.event {
            AuthEvent::Sudo { user, .. } => Some(user),
            _ => None,
        })
        .collect();
    entries
        .iter()
        .filter(|entry| entry.timestamp >= learn_until)
        .filter_map(|entry| match entry.event {
            AuthEvent::Sudo { user, command } if !sudoers.contains(user) => Some(UnusualSudo {
                user,
                host: entry.host,
                command,
                at: entry.timestamp,
            }),
            _ => None,
        })
        .collect()
    // SOLUTION-END
}

// ------------------------------------------------------------------ étape 7

/// Octets servis par (utilisateur, début d'heure), classés par utilisateur puis par heure.
///
/// Les requêtes anonymes et les réponses sans corps sont ignorées.
#[cfg_attr(not(feature = "step7"), allow(unused_variables))] // trou étape 7
pub fn bytes_by_user_hour<'a>(entries: &[WebEntry<'a>]) -> BTreeMap<(&'a str, Timestamp), u64> {
    // TODO-STEP:7 Tronquer l'horodatage à l'heure (`div_euclid`) et cumuler dans une
    // `BTreeMap` : ses clés sont triées, ce que les requêtes par intervalle exploiteront.
    // SOLUTION-BEGIN
    let mut volumes = BTreeMap::new();
    for entry in entries {
        if let (Some(user), Some(bytes)) = (entry.user, entry.bytes) {
            // `div_euclid` arrondit vers -∞, `/` vers 0 : seul le premier convient avant
            // 1970 (test `hours_before_1970_round_down`, étape 7)
            let hour = entry.timestamp.div_euclid(HOUR) * HOUR;
            *volumes.entry((user, hour)).or_insert(0) += bytes;
        }
    }
    volumes
    // SOLUTION-END
}

/// Volume servi à `user` pendant les heures commençant dans `[from, to)`.
#[cfg_attr(not(feature = "step7"), allow(unused_variables))] // trou étape 7
pub fn user_volume(
    volumes: &BTreeMap<(&str, Timestamp), u64>,
    user: &str,
    from: Timestamp,
    to: Timestamp,
) -> u64 {
    // TODO-STEP:7 Pas de parcours complet : `BTreeMap::range` sur les clés `(user, from)`
    // à `(user, to)`.
    // SOLUTION-BEGIN
    volumes
        .range((user, from)..(user, to))
        .map(|(_, bytes)| bytes)
        .sum()
    // SOLUTION-END
}

/// Heure pendant laquelle un utilisateur a reçu un volume anormal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeSpike<'a> {
    pub user: &'a str,
    pub hour: Timestamp,
    pub bytes: u64,
}

/// Les couples (utilisateur, heure) dont le volume atteint `threshold` octets.
pub fn detect_volume_spikes<'a>(
    volumes: &BTreeMap<(&'a str, Timestamp), u64>,
    threshold: u64,
) -> Vec<VolumeSpike<'a>> {
    volumes
        .iter()
        .filter(|&(_, &bytes)| bytes >= threshold)
        .map(|(&(user, hour), &bytes)| VolumeSpike { user, hour, bytes })
        .collect()
}

/// Une alerte, quelle que soit la détection qui l'a levée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alert<'a> {
    Spraying(Spraying),
    BruteForce(BruteForce),
    Compromise(Compromise<'a>),
    NewAccess(NewAccess<'a>),
    UnusualSudo(UnusualSudo<'a>),
    VolumeSpike(VolumeSpike<'a>),
}

impl Alert<'_> {
    /// Instant de l'alerte (début de l'épisode).
    pub fn at(&self) -> Timestamp {
        match self {
            Alert::Spraying(a) => a.first,
            Alert::BruteForce(a) => a.first,
            Alert::Compromise(a) => a.at,
            Alert::NewAccess(a) => a.at,
            Alert::UnusualSudo(a) => a.at,
            Alert::VolumeSpike(a) => a.hour,
        }
    }
}

impl fmt::Display for Alert<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}  ", Iso(self.at()))?;
        match self {
            Alert::Spraying(a) => write!(
                f,
                "password spraying   {} : {} comptes jusqu'à {}",
                a.ip,
                a.accounts,
                Iso(a.last)
            ),
            Alert::BruteForce(a) => write!(
                f,
                "force brute         {} : {} échecs jusqu'à {}",
                a.ip,
                a.failures,
                Iso(a.last)
            ),
            Alert::Compromise(a) => {
                write!(
                    f,
                    "COMPROMISSION       {} sur {} depuis {} après {} échecs",
                    a.user, a.host, a.ip, a.failures_before
                )
            }
            Alert::NewAccess(a) => write!(
                f,
                "accès inédit        {} sur {} depuis {}",
                a.user, a.host, a.ip
            ),
            Alert::UnusualSudo(a) => write!(
                f,
                "sudo inhabituel     {} sur {} : {}",
                a.user, a.host, a.command
            ),
            Alert::VolumeSpike(a) => write!(
                f,
                "volume anormal      {} : {} octets dans l'heure",
                a.user, a.bytes
            ),
        }
    }
}

/// Réunit plusieurs listes d'alertes en une chronologie.
///
/// À instant égal, l'ordre des listes est conservé.
#[cfg_attr(not(feature = "step7"), allow(unused_variables))] // trou étape 7
pub fn timeline<'a>(groups: Vec<Vec<Alert<'a>>>) -> Vec<Alert<'a>> {
    // TODO-STEP:7 Aplatir les listes (`into_iter().flatten()`), puis trier par instant avec
    // un tri **stable**.
    // SOLUTION-BEGIN
    let mut alerts: Vec<Alert<'a>> = groups.into_iter().flatten().collect();
    alerts.sort_by_key(Alert::at);
    alerts
    // SOLUTION-END
}

/// Seuils des détections.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Fin de la période d'apprentissage des habitudes.
    pub learn_until: Timestamp,
    pub brute_window: i64,
    pub brute_threshold: usize,
    pub spraying_accounts: usize,
    pub compromise_failures: usize,
    pub volume_threshold: u64,
}

impl Config {
    /// Seuils par défaut ; les habitudes s'apprennent sur les deux premiers jours.
    ///
    /// `first` est l'horodatage de la première entrée. Il est arrondi au minuit **le plus
    /// proche** : la première ligne peut venir d'un serveur dont l'horloge retarde, et
    /// dater de quelques minutes avant le début réel du journal.
    pub fn starting_at(first: Timestamp) -> Self {
        Self {
            learn_until: (first + DAY / 2).div_euclid(DAY) * DAY + 2 * DAY,
            brute_window: 5 * MINUTE,
            brute_threshold: 20,
            spraying_accounts: 20,
            compromise_failures: 10,
            volume_threshold: 1_000_000_000,
        }
    }
}

/// Lance toutes les détections et renvoie la chronologie des alertes.
pub fn analyze<'a>(
    auth: &[AuthEntry<'a>],
    web: &[WebEntry<'a>],
    config: &Config,
) -> Vec<Alert<'a>> {
    let volumes = bytes_by_user_hour(web);
    timeline(vec![
        detect_spraying(auth, config.spraying_accounts)
            .into_iter()
            .map(Alert::Spraying)
            .collect(),
        detect_bruteforce(auth, config.brute_window, config.brute_threshold)
            .into_iter()
            .map(Alert::BruteForce)
            .collect(),
        detect_compromise(auth, config.compromise_failures)
            .into_iter()
            .map(Alert::Compromise)
            .collect(),
        detect_new_access(auth, config.learn_until)
            .into_iter()
            .map(Alert::NewAccess)
            .collect(),
        detect_unusual_sudo(auth, config.learn_until)
            .into_iter()
            .map(Alert::UnusualSudo)
            .collect(),
        detect_volume_spikes(&volumes, config.volume_threshold)
            .into_iter()
            .map(Alert::VolumeSpike)
            .collect(),
    ])
}
