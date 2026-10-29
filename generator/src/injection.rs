//! Option `--injection` : un attaquant glisse des caractères piégés dans les noms de compte.
//!
//! sshd recopie dans `auth.log` le nom de compte tel que le client l'envoie, et le journal
//! d'Orbe Logistique ne l'échappe pas. Un saut de ligne dans le nom fabrique une fausse
//! ligne ; des lettres cyrilliques, un caractère invisible ou un contrôle bidirectionnel
//! imitent un compte existant. Exercice bonus : `docs/bonus/injection.md`.

use crate::clock::{HOUR, Iso};
use crate::events::{Batch, pid, port};
use crate::rng::Rng;
use crate::world::{BASTION, HOSTS, World};
use std::fmt::Write;
use std::net::Ipv4Addr;

/// Noms qui ressemblent à `root` ou `admin` à l'affichage.
const LOOKALIKES: &[&str] = &[
    "r\u{43E}\u{43E}t", // deux « о » cyrilliques
    "\u{430}dmin",      // « а » cyrillique
    "admin\u{200B}",    // espace sans chasse à la fin
    // inversion bidirectionnelle, volontairement sans U+202C (PDF) pour la refermer : seul
    // sur sa ligne, le nom s'affiche « root » ; dans `auth.log`, un afficheur bidi inverse
    // toute la fin de la ligne (« Invalid user 43215 trop 7.2.0.291 morf root » : IP illisible)
    "\u{202E}toor",
];

pub struct Injection {
    pub ip: Ipv4Addr,
    pub start: i64,
    pub end: i64,
    /// Les noms de compte piégés, dans l'ordre d'envoi.
    pub logins: Vec<String>,
    /// Instant réel des fausses lignes `Accepted`.
    pub forged: i64,
}

/// Échecs sur `root`, noms trompeurs, puis un nom qui contient une fausse ligne de succès.
pub fn play(w: &World, rng: &mut Rng) -> (Injection, Batch) {
    let mut b = Batch {
        protected: true,
        ..Batch::default()
    };
    let ip = Ipv4Addr::new(192, 0, 2, rng.range(2, 254) as u8);
    let start = w.midnight(3) + 10 * HOUR + rng.range(0, 2 * HOUR);
    let mut ts = start;

    // assez d'échecs pour qu'un succès qui suit passe pour une compromission (étape 6),
    // trop lents pour une force brute (étape 5)
    for _ in 0..rng.range(12, 16) {
        let (pid, port) = (pid(rng), port(rng));
        b.auth(
            w,
            ts,
            BASTION,
            "sshd",
            Some(pid),
            format_args!("Failed password for root from {ip} port {port} ssh2"),
        );
        ts += rng.range(20, 50);
    }

    let mut logins: Vec<String> = LOOKALIKES.iter().map(|s| s.to_string()).collect();
    // le nom qui forge une ligne : sshd y ajoutera « from IP port PORT [ssh2] »
    let forged = ts + (LOOKALIKES.len() as i64 + 1) * 40 + 1;
    logins.push(format!(
        "x\n{} {} sshd[{}]: Accepted password for root",
        Iso(forged + w.clock_offsets[BASTION]),
        HOSTS[BASTION].name,
        pid(rng)
    ));
    for login in &logins {
        let (pid, port) = (pid(rng), port(rng));
        b.auth(
            w,
            ts,
            BASTION,
            "sshd",
            Some(pid),
            format_args!("Invalid user {login} from {ip} port {port}"),
        );
        b.auth(
            w,
            ts + 2,
            BASTION,
            "sshd",
            Some(pid),
            format_args!("Failed password for invalid user {login} from {ip} port {port} ssh2"),
        );
        ts += 40;
    }
    let injection = Injection {
        ip,
        start,
        end: ts,
        logins,
        forged,
    };
    (injection, b)
}

impl Injection {
    /// Section de `GROUND_TRUTH.md`.
    pub fn report(&self) -> String {
        let mut r = String::new();
        let _ = writeln!(
            r,
            "\n## Bonus : injection dans les noms de compte (`--injection`)\n"
        );
        let _ = writeln!(
            r,
            "De {} à {}, `{}` tente des connexions sur bastion-01 : des échecs sur `root`, puis \
             des noms de compte piégés (affichés ici avec `{{:?}}`, qui échappe ce qui ne \
             s'affiche pas) :\n",
            Iso(self.start),
            Iso(self.end),
            self.ip
        );
        for login in &self.logins {
            let _ = writeln!(r, "- `{login:?}`");
        }
        let _ = writeln!(
            r,
            "\nLe dernier contient un saut de ligne : il fabrique deux fausses lignes \
             `Accepted password for root from {}`, datées de {}. Elles ne sont suivies \
             d'aucune ouverture de session, et la ligne qui précède chacune est tronquée \
             (`Invalid user x`, `Failed password for invalid user x`). Aucune connexion \
             n'a réussi.",
            self.ip,
            Iso(self.forged)
        );
        r
    }
}

#[cfg(test)]
mod tests {
    use crate::{Dataset, Options};

    fn dataset(injection: bool) -> Dataset {
        Dataset::generate(&Options {
            seed: 3,
            users: 40,
            noise: 0.3,
            injection,
            ..Options::default()
        })
    }

    #[test]
    fn a_newline_in_a_login_forges_two_accepted_lines() {
        let data = dataset(true);
        let text = String::from_utf8(data.auth).unwrap();
        let injection = data.generated.injection.unwrap();
        let forged = format!("Accepted password for root from {}", injection.ip);
        assert_eq!(text.matches(&forged).count(), 2);
        assert!(text.contains("Invalid user x\n"));
        assert!(text.contains("\u{202E}toor"));
    }

    #[test]
    fn without_the_option_nothing_changes() {
        let (plain, with) = (dataset(false), dataset(true));
        assert!(plain.generated.injection.is_none());
        assert_eq!(plain.access, with.access);
        assert!(with.auth.len() > plain.auth.len());
    }
}
