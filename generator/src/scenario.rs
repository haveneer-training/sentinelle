//! Le scénario d'intrusion caché dans les journaux.
//!
//! Tous les paramètres (adresses, victime, horaires) dérivent de la graine :
//! lire ce code ne donne pas la solution d'un jeu de données généré avec une graine inconnue.

use crate::clock::{HOUR, Iso, MINUTE};
use crate::events::{Batch, internet_ip, port, ssh_session, sudo};
use crate::rng::Rng;
use crate::world::{BASTION, CLIENT_DB, HOSTS, WEB_FRONT, World};
use std::fmt::Write;
use std::net::Ipv4Addr;

pub const MIN_DAYS: i64 = 5;

const FAKE_ACCOUNTS: &[&str] = &[
    "support",
    "helpdesk",
    "stagiaire",
    "compta",
    "rh",
    "direction",
    "backup",
    "svc-backup",
    "scanner",
    "imprimante",
    "formation",
    "accueil",
    "sauvegarde",
    "admin2",
    "test1",
];

pub struct Scenario {
    pub spray_ip: Ipv4Addr,
    pub brute_ip: Ipv4Addr,
    pub victim: usize,
    pub spray: (i64, i64, usize),
    pub brute: (i64, i64, u32),
    /// Option `--distributed` : les bots qui mènent la force brute, avec leur nombre
    /// d'échecs (vide sans l'option).
    pub bots: Vec<(Ipv4Addr, u32)>,
    pub compromise: i64,
    pub pivot_web: i64,
    pub pivot_db: i64,
    pub dump: i64,
    pub exfil: (i64, i64, u32, u64),
}

/// `botnet` (option `--distributed`) : flux à part qui répartit la force brute entre de
/// nombreuses adresses. Le flux principal est consommé à l'identique : le reste du scénario
/// ne change pas.
pub fn play(w: &World, rng: &mut Rng, mut botnet: Option<&mut Rng>) -> (Scenario, Batch) {
    assert!(
        w.days >= MIN_DAYS,
        "le scénario demande au moins {MIN_DAYS} jours"
    );
    let mut b = Batch {
        protected: true,
        ..Batch::default()
    };

    let doc_ip = |rng: &mut Rng, net: [u8; 3]| {
        Ipv4Addr::new(net[0], net[1], net[2], rng.range(2, 254) as u8)
    };
    let spray_ip = doc_ip(rng, [203, 0, 113]);
    let brute_ip = doc_ip(rng, [198, 51, 100]);
    // la victime : un salarié ordinaire, qui n'a normalement pas accès à la base clients
    let candidates: Vec<usize> = (0..w.users.len())
        .filter(|&i| !w.users[i].admin && w.users[i].web)
        .collect();
    assert!(
        !candidates.is_empty(),
        "pas assez d'utilisateurs pour jouer le scénario"
    );
    let victim = *rng.pick(&candidates);
    let v = &w.users[victim];

    // Jour 1 (mardi après-midi) : password spraying, un ou deux essais par compte, lentement
    let spray_start = w.midnight(1) + 14 * HOUR + rng.range(0, HOUR);
    let mut targets: Vec<&str> = w
        .users
        .iter()
        .filter(|_| rng.chance(0.7))
        .map(|u| u.name.as_str())
        .collect();
    targets.extend(FAKE_ACCOUNTS);
    rng.shuffle(&mut targets);
    let mut ts = spray_start;
    // instant du dernier essai écrit : `ts` a déjà avancé d'un pas en fin de boucle
    let mut spray_last = ts;
    for &login in &targets {
        let port = port(rng);
        let pid = Some(crate::events::pid(rng));
        if FAKE_ACCOUNTS.contains(&login) {
            spray_last = ts + 2;
            b.auth(
                w,
                ts,
                BASTION,
                "sshd",
                pid,
                format_args!("Invalid user {login} from {spray_ip} port {port}"),
            );
            b.auth(
                w,
                ts + 2,
                BASTION,
                "sshd",
                pid,
                format_args!(
                    "Failed password for invalid user {login} from {spray_ip} port {port} ssh2"
                ),
            );
        } else {
            spray_last = ts;
            b.auth(
                w,
                ts,
                BASTION,
                "sshd",
                pid,
                format_args!("Failed password for {login} from {spray_ip} port {port} ssh2"),
            );
        }
        ts += rng.range(25, 70);
    }
    let spray = (spray_start, spray_last, targets.len());

    // Jour 2 (mercredi soir) : force brute sur la victime, puis succès
    let brute_start = w.midnight(2) + 22 * HOUR + rng.range(0, 30 * MINUTE);
    let failures = rng.range(350, 450) as u32;
    // (adresse, échecs) de chaque bot, à des adresses toutes distinctes
    let mut bots: Vec<(Ipv4Addr, u32)> = Vec::new();
    if let Some(bot_rng) = botnet.as_deref_mut() {
        let mut taken = vec![spray_ip, brute_ip];
        for _ in 0..bot_rng.range(80, 120) {
            let ip = internet_ip(bot_rng, &taken);
            taken.push(ip);
            bots.push((ip, 0));
        }
    }
    let mut ts = brute_start;
    let mut pid = crate::events::pid(rng);
    let mut port = port(rng);
    let mut brute_last = ts;
    for i in 1..=failures {
        brute_last = ts;
        if let Some(bot_rng) = botnet.as_deref_mut() {
            // chaque essai vient d'un bot pris au hasard, sur une connexion à lui
            // tour à tour, dans un ordre rebattu à chaque tour : au plus
            // ⌈450 / 80⌉ = 6 essais par bot, espacés de plusieurs minutes
            let round = (i as usize - 1) % bots.len();
            if round == 0 {
                bot_rng.shuffle(&mut bots);
            }
            let (bot, tries) = &mut bots[round];
            *tries += 1;
            let (bot_pid, bot_port) = (crate::events::pid(bot_rng), crate::events::port(bot_rng));
            b.auth(
                w,
                ts,
                BASTION,
                "sshd",
                Some(bot_pid),
                format_args!(
                    "Failed password for {} from {bot} port {bot_port} ssh2",
                    v.name
                ),
            );
        } else {
            b.auth(
                w,
                ts,
                BASTION,
                "sshd",
                Some(pid),
                format_args!(
                    "Failed password for {} from {brute_ip} port {port} ssh2",
                    v.name
                ),
            );
        }
        if i % 6 == 0 {
            if botnet.is_none() {
                b.auth(w, ts, BASTION, "sshd", Some(pid), format_args!("error: maximum authentication attempts exceeded for {} from {brute_ip} port {port} ssh2 [preauth]", v.name));
            }
            pid += rng.range(1, 40) as u32;
            port += rng.range(1, 9) as u16;
        }
        ts += rng.range(1, 4);
    }
    let brute = (brute_start, brute_last, failures);
    let compromise = ts + rng.range(2, 6);
    let session = rng.range(HOUR, 2 * HOUR);
    ssh_session(
        &mut b, w, rng, compromise, BASTION, &v.name, v.uid, brute_ip, "password", session,
    );

    // Rebond vers le frontal web depuis le bastion
    let pivot_web = compromise + rng.range(10 * MINUTE, 20 * MINUTE);
    let session = rng.range(3 * HOUR, 4 * HOUR);
    ssh_session(
        &mut b,
        w,
        rng,
        pivot_web,
        WEB_FRONT,
        &v.name,
        v.uid,
        HOSTS[BASTION].ip,
        "password",
        session,
    );

    // Jour 3 (jeudi, 1h30) : rebond vers la base clients, élévation de privilèges, extraction
    let pivot_db = w.midnight(3) + HOUR + 20 * MINUTE + rng.range(0, 30 * MINUTE);
    let session = 40 * MINUTE + rng.range(0, 10 * MINUTE);
    ssh_session(
        &mut b,
        w,
        rng,
        pivot_db,
        CLIENT_DB,
        &v.name,
        v.uid,
        HOSTS[WEB_FRONT].ip,
        "password",
        session,
    );
    let dump = pivot_db + rng.range(MINUTE, 5 * MINUTE);
    sudo(
        &mut b,
        w,
        dump,
        CLIENT_DB,
        &v.name,
        1,
        "/usr/bin/pg_dump -t clients crm -f /tmp/crm-clients.sql",
    );
    sudo(
        &mut b,
        w,
        dump + rng.range(5 * MINUTE, 10 * MINUTE),
        CLIENT_DB,
        &v.name,
        1,
        "/usr/bin/cp /tmp/crm-clients.sql /srv/export/",
    );

    // Jour 4 (vendredi, vers 3h) : exfiltration via l'API d'export, depuis l'adresse de l'attaquant
    let exfil_start = w.midnight(4) + 2 * HOUR + 50 * MINUTE + rng.range(0, 20 * MINUTE);
    let mut ts = exfil_start;
    let (mut total, mut requests) = (0u64, 0u32);
    // volontairement au-delà de u32::MAX octets
    while total < 5_500_000_000 {
        requests += 1;
        let bytes = rng.range(12_000_000, 24_000_000) as u64;
        b.access(
            ts,
            brute_ip,
            Some(&v.name),
            "GET",
            format_args!("/api/export?table=clients&page={requests}"),
            200,
            Some(bytes),
            bytes / 20_000,
        );
        total += bytes;
        ts += rng.range(4, 12);
    }
    let exfil = (exfil_start, ts, requests, total);

    let s = Scenario {
        spray_ip,
        brute_ip,
        victim,
        spray,
        brute,
        bots,
        compromise,
        pivot_web,
        pivot_db,
        dump,
        exfil,
    };
    (s, b)
}

impl Scenario {
    pub fn report(&self, w: &World, seed: u64) -> String {
        let v = &w.users[self.victim].name;
        let db = HOSTS[CLIENT_DB].name;
        let mut r = String::new();
        let _ = writeln!(r, "# Vérité terrain — graine {seed}\n");
        let _ = writeln!(
            r,
            "La solution de l'enquête, donnée comme référence : ne l'ouvrez qu'après avoir mené\nla vôtre. La lire avant, c'est tricher, et surtout vous priver de l'exercice.\n"
        );
        let _ = writeln!(r, "| Étape | Début | Fin | Détails |\n|---|---|---|---|");
        let _ = writeln!(
            r,
            "| Password spraying | {} | {} | `{}` → bastion-01, {} comptes visés (dont {} inexistants) |",
            Iso(self.spray.0),
            Iso(self.spray.1),
            self.spray_ip,
            self.spray.2,
            FAKE_ACCOUNTS.len()
        );
        if self.bots.is_empty() {
            let _ = writeln!(
                r,
                "| Force brute | {} | {} | `{}` → `{v}`@bastion-01, {} échecs |",
                Iso(self.brute.0),
                Iso(self.brute.1),
                self.brute_ip,
                self.brute.2
            );
        } else {
            let _ = writeln!(
                r,
                "| Force brute distribuée | {} | {} | {} adresses → `{v}`@bastion-01, {} échecs (au plus {} par adresse) |",
                Iso(self.brute.0),
                Iso(self.brute.1),
                self.bots.len(),
                self.brute.2,
                self.max_bot_failures()
            );
        }
        let _ = writeln!(
            r,
            "| Compromission | {} | | `Accepted password for {v} from {}` |",
            Iso(self.compromise),
            self.brute_ip
        );
        let _ = writeln!(
            r,
            "| Rebond web | {} | | bastion-01 → srv-web-01 (`{v}`) |",
            Iso(self.pivot_web)
        );
        let _ = writeln!(
            r,
            "| Rebond base clients | {} | | srv-web-01 → {db} (`{v}`, jamais vu sur ce serveur) |",
            Iso(self.pivot_db)
        );
        let _ = writeln!(
            r,
            "| Élévation + dump | {} | | `sudo pg_dump` par un non-administrateur sur {db} |",
            Iso(self.dump)
        );
        let _ = writeln!(
            r,
            "| Exfiltration | {} | {} | `{}` (`{v}`), {} requêtes `/api/export?table=clients`, {} octets (> u32::MAX) |",
            Iso(self.exfil.0),
            Iso(self.exfil.1),
            self.brute_ip,
            self.exfil.2,
            self.exfil.3
        );
        let _ = writeln!(
            r,
            "\nHeures réelles (UTC). L'horloge de {db} dérive de {} s : ses lignes affichent une heure décalée.",
            w.clock_offsets[CLIENT_DB]
        );
        if !self.bots.is_empty() {
            let _ = writeln!(
                r,
                "\n## Bonus : force brute distribuée (`--distributed`)\n\n\
                 Les {} échecs sur `{v}` viennent de {} adresses, au plus {} chacune : aucune \
                 ne franchit les seuils par adresse (force brute, spraying, compromission). La \
                 connexion réussie vient de `{}`, qui n'a jamais échoué ; c'est aussi \
                 l'adresse de l'exfiltration.",
                self.brute.2,
                self.bots.len(),
                self.max_bot_failures(),
                self.brute_ip
            );
        }
        r
    }

    /// Le plus grand nombre d'échecs d'un même bot (0 sans `--distributed`).
    pub fn max_bot_failures(&self) -> u32 {
        self.bots.iter().map(|&(_, tries)| tries).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Dataset, Options};

    fn dataset(distributed: bool) -> Dataset {
        Dataset::generate(&Options {
            seed: 3,
            users: 40,
            noise: 0.3,
            distributed,
            ..Options::default()
        })
    }

    #[test]
    fn a_distributed_brute_force_stays_under_the_per_address_thresholds() {
        let data = dataset(true);
        let text = String::from_utf8(data.auth).unwrap();
        let s = &data.generated.scenario;
        let victim = &data.generated.world.users[s.victim].name;
        let tries: u32 = s.bots.iter().map(|&(_, tries)| tries).sum();
        assert_eq!(tries, s.brute.2);
        assert!(s.bots.len() >= 80);
        let distinct: std::collections::HashSet<_> = s.bots.iter().map(|&(ip, _)| ip).collect();
        assert_eq!(distinct.len(), s.bots.len());
        // loin du seuil de force brute par adresse (20 en 5 min)
        assert!(s.max_bot_failures() <= 6);
        let failures = format!("Failed password for {victim} from ");
        assert!(text.matches(&failures).count() >= s.brute.2 as usize);
        // l'adresse de l'attaquant n'échoue jamais, mais se connecte
        let attacker = format!("from {} port", s.brute_ip);
        let attacker: Vec<&str> = text.lines().filter(|l| l.contains(&attacker)).collect();
        assert!(attacker.iter().all(|l| !l.contains("Failed")));
        assert!(attacker.iter().any(|l| l.contains("Accepted")));
    }

    #[test]
    fn the_rest_of_the_scenario_does_not_change() {
        let (plain, with) = (dataset(false), dataset(true));
        let (p, w) = (&plain.generated.scenario, &with.generated.scenario);
        assert!(p.bots.is_empty());
        assert_eq!(p.max_bot_failures(), 0);
        assert_eq!(
            (p.brute, p.compromise, p.dump, p.exfil),
            (w.brute, w.compromise, w.dump, w.exfil)
        );
    }
}
