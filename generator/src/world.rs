//! Le parc informatique fictif d'Orbe Logistique : serveurs et salariés.

use crate::clock::{DAY, MINUTE, days_from_civil};
use crate::rng::Rng;
use std::collections::HashSet;
use std::net::Ipv4Addr;

pub struct Host {
    pub name: &'static str,
    pub ip: Ipv4Addr,
}

pub const HOSTS: [Host; 7] = [
    Host {
        name: "bastion-01",
        ip: Ipv4Addr::new(10, 0, 0, 10),
    },
    Host {
        name: "srv-web-01",
        ip: Ipv4Addr::new(10, 0, 2, 11),
    },
    Host {
        name: "srv-web-02",
        ip: Ipv4Addr::new(10, 0, 2, 12),
    },
    Host {
        name: "srv-db-01",
        ip: Ipv4Addr::new(10, 0, 3, 21),
    },
    Host {
        name: "srv-db-02",
        ip: Ipv4Addr::new(10, 0, 3, 22),
    },
    Host {
        name: "srv-files-01",
        ip: Ipv4Addr::new(10, 0, 4, 31),
    },
    Host {
        name: "srv-build-01",
        ip: Ipv4Addr::new(10, 0, 5, 41),
    },
];

/// Seul point d'entrée SSH depuis l'extérieur.
pub const BASTION: usize = 0;
/// Frontal de l'application web interne (celui qui écrit `access.log`).
pub const WEB_FRONT: usize = 1;
/// Base des données clients : réservée aux administrateurs.
pub const CLIENT_DB: usize = 4;

const FIRST_INITIALS: &[u8] = b"abcdefghijlmnoprstvy";
const SURNAMES: &[&str] = &[
    "martin",
    "bernard",
    "thomas",
    "petit",
    "robert",
    "richard",
    "durand",
    "dubois",
    "moreau",
    "laurent",
    "simon",
    "michel",
    "lefebvre",
    "leroy",
    "roux",
    "david",
    "bertrand",
    "morel",
    "fournier",
    "girard",
    "bonnet",
    "dupont",
    "lambert",
    "fontaine",
    "rousseau",
    "vincent",
    "muller",
    "faure",
    "andre",
    "mercier",
    "blanc",
    "guerin",
    "boyer",
    "garnier",
    "chevalier",
    "legrand",
    "gauthier",
    "garcia",
    "perrin",
    "robin",
    "clement",
    "morin",
    "nicolas",
    "henry",
    "roussel",
    "mathieu",
    "masson",
    "le-gall",
    "le-goff",
    "tanguy",
    "kerjean",
    "riou",
    "quere",
];

pub struct User {
    pub name: String,
    pub uid: u32,
    pub workstation: Ipv4Addr,
    pub admin: bool,
    /// Utilise l'application web interne.
    pub web: bool,
    /// Serveurs sur lesquels l'utilisateur se connecte habituellement en SSH.
    pub servers: Vec<usize>,
}

pub struct World {
    /// Lundi 00:00:00 UTC du premier jour de journaux.
    pub start: i64,
    pub days: i64,
    pub users: Vec<User>,
    /// Dérive d'horloge de chaque serveur (secondes ajoutées à l'heure affichée).
    pub clock_offsets: [i64; HOSTS.len()],
}

impl World {
    pub fn new(rng: &mut Rng, user_count: usize, days: i64) -> Self {
        let mut clock_offsets = [0; HOSTS.len()];
        // un serveur mal synchronisé (NTP en panne) : ses lignes semblent « en retard »
        clock_offsets[CLIENT_DB] = -rng.range(MINUTE, 3 * MINUTE);
        Self {
            start: days_from_civil(2026, 11, 9) * DAY,
            days,
            users: make_users(rng, user_count),
            clock_offsets,
        }
    }

    pub fn midnight(&self, day: i64) -> i64 {
        self.start + day * DAY
    }
}

fn make_users(rng: &mut Rng, count: usize) -> Vec<User> {
    assert!(count <= 60_000, "au plus 60 000 utilisateurs");
    let regular_servers: Vec<usize> = (1..HOSTS.len()).filter(|&h| h != CLIENT_DB).collect();
    let all_servers: Vec<usize> = (1..HOSTS.len()).collect();
    let mut taken = HashSet::new();
    (0..count)
        .map(|i| {
            let base = format!(
                "{}.{}",
                *rng.pick(FIRST_INITIALS) as char,
                rng.pick(SURNAMES)
            );
            let mut name = base.clone();
            let mut suffix = 2;
            while !taken.insert(name.clone()) {
                name = format!("{base}{suffix}");
                suffix += 1;
            }
            let admin = rng.chance(0.08);
            let (pool, max) = if admin {
                (&all_servers, 6)
            } else {
                (&regular_servers, 2)
            };
            let mut servers = pool.clone();
            rng.shuffle(&mut servers);
            servers.truncate(rng.range(if admin { 3 } else { 0 }, max + 1) as usize);
            User {
                name,
                uid: 1000 + i as u32,
                workstation: Ipv4Addr::new(10, 1, (i / 250) as u8, (i % 250 + 1) as u8),
                admin,
                web: rng.chance(0.85),
                servers,
            }
        })
        .collect()
}
