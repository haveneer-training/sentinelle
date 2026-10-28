// Bonus « force brute distribuée » : étapes 2 et 3 de docs/bonus/distributed.md.
// À ajouter à `sentinelle/src/detect.rs` (ou dans un module à part).

use sentinelle::auth::{AuthEntry, AuthEvent};
use sentinelle::time::Timestamp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::IpAddr;

/// Les échecs récents d'un compte, et combien en vient de chaque adresse.
#[derive(Default)]
struct Recent {
    failures: VecDeque<(Timestamp, IpAddr)>,
    by_ip: HashMap<IpAddr, usize>,
}

impl Recent {
    fn push(&mut self, at: Timestamp, ip: IpAddr) {
        self.failures.push_back((at, ip));
        *self.by_ip.entry(ip).or_insert(0) += 1;
    }

    /// Oublie les échecs d'il y a `window` secondes ou plus.
    fn expire(&mut self, now: Timestamp, window: i64) {
        while let Some(&(at, ip)) = self.failures.front() {
            if now - at < window {
                break;
            }
            self.failures.pop_front();
            // une adresse qui n'a plus d'échec dans la fenêtre sort de la table : `len()`
            // reste le nombre d'adresses distinctes
            if let Some(n) = self.by_ip.get_mut(&ip) {
                *n -= 1;
                if *n == 0 {
                    self.by_ip.remove(&ip);
                }
            }
        }
    }
}

/// Rafale d'échecs sur un même compte, toutes adresses confondues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountBruteForce<'a> {
    pub user: &'a str,
    pub first: Timestamp,
    pub last: Timestamp,
    /// Les échecs sur le compte de `first` à `last` inclus.
    pub failures: usize,
    /// Les adresses distinctes de ces échecs.
    pub ips: usize,
}

/// Rafales d'au moins `threshold` échecs sur un même compte en moins de `window` secondes,
/// venant d'au moins `min_ips` adresses distinctes. Même logique que `detect_bruteforce`
/// (étape 5), avec le compte pour clé au lieu de l'adresse.
///
/// Les échecs sur un compte inexistant (`invalid user`) sont ignorés : ils ne mènent à
/// aucune connexion, et le bruit d'Internet en produit sans cesse (`admin`, `pi`, `test`…).
pub fn detect_account_bruteforce<'a>(
    entries: &[AuthEntry<'a>],
    window: i64,
    threshold: usize,
    min_ips: usize,
) -> Vec<AccountBruteForce<'a>> {
    let mut recent: HashMap<&str, Recent> = HashMap::new();
    // la rafale en cours de chaque compte, avec l'ensemble de ses adresses
    let mut open: HashMap<&str, (AccountBruteForce<'a>, HashSet<IpAddr>)> = HashMap::new();
    let mut found = Vec::new();
    let close = |(mut burst, ips): (AccountBruteForce<'a>, HashSet<IpAddr>)| {
        burst.ips = ips.len();
        burst
    };
    for entry in entries {
        let AuthEvent::Failed {
            user,
            ip,
            invalid_user: false,
        } = entry.event
        else {
            continue;
        };
        let now = entry.timestamp;
        let window_of = recent.entry(user).or_default();
        window_of.push(now, ip);
        window_of.expire(now, window);
        if window_of.failures.len() < threshold || window_of.by_ip.len() < min_ips {
            continue;
        }
        match open.get_mut(user) {
            Some((burst, ips)) if now - burst.last < window => {
                // les échecs arrivés depuis la fin connue de la rafale, y compris ceux passés
                // sous le seuil entre-temps
                let since = window_of
                    .failures
                    .partition_point(|&(t, _)| t <= burst.last);
                for &(_, ip) in window_of.failures.range(since..) {
                    burst.failures += 1;
                    ips.insert(ip);
                }
                burst.last = now;
            }
            _ => {
                let burst = AccountBruteForce {
                    user,
                    first: window_of.failures[0].0,
                    last: now,
                    failures: window_of.failures.len(),
                    ips: 0,
                };
                let ips = window_of.by_ip.keys().copied().collect();
                found.extend(open.insert(user, (burst, ips)).map(close));
            }
        }
    }
    found.extend(open.into_values().map(close));
    found.sort_by_key(|burst| (burst.first, burst.user));
    found
}

/// Connexion réussie sur un compte qui venait d'échouer de nombreuses fois.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountCompromise<'a> {
    pub user: &'a str,
    pub host: &'a str,
    /// L'adresse de la connexion réussie.
    pub ip: IpAddr,
    pub at: Timestamp,
    /// Les échecs sur le compte dans les `window` secondes qui précèdent.
    pub failures: usize,
    /// Les adresses distinctes de ces échecs.
    pub ips: usize,
    /// `ip` figure-t-elle parmi ces adresses ? Si non, l'attaquant s'est connecté depuis
    /// une machine qui n'a jamais échoué.
    pub from_failing_ip: bool,
}

/// Connexions réussies (`Accepted`) sur un compte qui a cumulé au moins `min_failures`
/// échecs dans les `window` secondes précédentes, quelles qu'en soient les adresses.
///
/// Une connexion rapportée consomme les échecs qui la précèdent : le rebond qui suit, avec
/// le même compte, n'est pas rapporté une seconde fois.
pub fn detect_account_compromise<'a>(
    entries: &[AuthEntry<'a>],
    window: i64,
    min_failures: usize,
) -> Vec<AccountCompromise<'a>> {
    let mut recent: HashMap<&str, Recent> = HashMap::new();
    let mut found = Vec::new();
    for entry in entries {
        match entry.event {
            AuthEvent::Failed { user, ip, .. } => {
                recent.entry(user).or_default().push(entry.timestamp, ip);
            }
            AuthEvent::Accepted { user, ip, .. } => {
                let Some(window_of) = recent.get_mut(user) else {
                    continue;
                };
                window_of.expire(entry.timestamp, window);
                if window_of.failures.len() >= min_failures {
                    found.push(AccountCompromise {
                        user,
                        host: entry.host,
                        ip,
                        at: entry.timestamp,
                        failures: window_of.failures.len(),
                        ips: window_of.by_ip.len(),
                        from_failing_ip: window_of.by_ip.contains_key(&ip),
                    });
                    *window_of = Recent::default();
                }
            }
            _ => {}
        }
    }
    found
}
