//! Activité « normale » : salariés, tâches cron et bruit de fond d'Internet.

use crate::clock::{DAY, HOUR, Iso, MINUTE};
use crate::rng::Rng;
use crate::world::{BASTION, HOSTS, User, World};
use std::fmt::Display;
use std::net::Ipv4Addr;

pub struct Line {
    /// Instant réel de l'événement (ordre d'écriture dans le fichier).
    pub ts: i64,
    pub text: String,
    /// Ligne du scénario : jamais corrompue par l'injection d'erreurs.
    pub protected: bool,
}

#[derive(Default)]
pub struct Batch {
    pub auth: Vec<Line>,
    pub access: Vec<Line>,
    pub protected: bool,
}

impl Batch {
    /// Ligne de `auth.log` : `TS HOST PROCESS[PID]: MESSAGE`, horodatée par l'horloge (éventuellement fausse) du serveur.
    pub fn auth(&mut self, world: &World, ts: i64, host: usize, process: &str, pid: Option<u32>, message: impl Display) {
        let shown = Iso(ts + world.clock_offsets[host]);
        let name = HOSTS[host].name;
        let text = match pid {
            Some(pid) => format!("{shown} {name} {process}[{pid}]: {message}"),
            None => format!("{shown} {name} {process}: {message}"),
        };
        self.auth.push(Line { ts, text, protected: self.protected });
    }

    /// Ligne de `access.log` : `IP - USER [TS] "METHOD PATH HTTP/1.1" STATUS BYTES DURATION_MS`.
    #[allow(clippy::too_many_arguments)]
    pub fn access(&mut self, ts: i64, ip: Ipv4Addr, user: Option<&str>, method: &str, path: impl Display, status: u16, bytes: Option<u64>, duration_ms: u64) {
        let user = user.unwrap_or("-");
        let bytes = bytes.map_or_else(|| "-".to_string(), |b| b.to_string());
        let text = format!("{ip} - {user} [{}] \"{method} {path} HTTP/1.1\" {status} {bytes} {duration_ms}", Iso(ts));
        self.access.push(Line { ts, text, protected: self.protected });
    }

    pub fn append(&mut self, other: Batch) {
        self.auth.extend(other.auth);
        self.access.extend(other.access);
    }
}

pub fn pid(rng: &mut Rng) -> u32 {
    rng.range(1_000, 65_000) as u32
}

pub fn port(rng: &mut Rng) -> u16 {
    rng.range(32_768, 61_000) as u16
}

/// Session SSH réussie : connexion, ouverture puis fermeture de session PAM.
#[allow(clippy::too_many_arguments)]
pub fn ssh_session(b: &mut Batch, w: &World, rng: &mut Rng, ts: i64, host: usize, user: &str, uid: u32, from: Ipv4Addr, method: &str, duration: i64) -> u32 {
    let pid = pid(rng);
    let port = port(rng);
    b.auth(w, ts, host, "sshd", Some(pid), format_args!("Accepted {method} for {user} from {from} port {port} ssh2"));
    b.auth(w, ts + 1, host, "sshd", Some(pid), format_args!("pam_unix(sshd:session): session opened for user {user}(uid={uid}) by (uid=0)"));
    b.auth(w, ts + duration, host, "sshd", Some(pid), format_args!("pam_unix(sshd:session): session closed for user {user}"));
    pid
}

pub fn sudo(b: &mut Batch, w: &World, ts: i64, host: usize, user: &str, tty: u32, command: &str) {
    // sudo aligne son message avec des espaces : le format réel contient des champs vides
    b.auth(w, ts, host, "sudo", None, format_args!("  {user} : TTY=pts/{tty} ; PWD=/home/{user} ; USER=root ; COMMAND={command}"));
}

const ADMIN_COMMANDS: &[&str] = &[
    "/usr/bin/systemctl restart nginx",
    "/usr/bin/systemctl status postgresql",
    "/usr/bin/apt-get update",
    "/usr/bin/journalctl -u sshd --since today",
    "/usr/bin/tail -n 200 /var/log/syslog",
    "/usr/bin/df -h",
    "/usr/sbin/ss -tlnp",
];

pub fn normal_day(b: &mut Batch, w: &World, rng: &mut Rng, day: i64) {
    let midnight = w.midnight(day);
    let weekday = day.rem_euclid(7) < 5;
    for user in &w.users {
        let present = if weekday { rng.chance(0.93) } else { user.admin && rng.chance(0.15) };
        if !present {
            continue;
        }
        let arrive = midnight + rng.range(7 * HOUR + 30 * MINUTE, 9 * HOUR + 30 * MINUTE);
        let leave = midnight + rng.range(16 * HOUR + 30 * MINUTE, 19 * HOUR + 30 * MINUTE);
        if !user.servers.is_empty() {
            ssh_activity(b, w, rng, user, arrive, leave, midnight + DAY - 1);
        }
        if user.web {
            web_activity(b, rng, user, arrive, leave);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn ssh_activity(b: &mut Batch, w: &World, rng: &mut Rng, user: &User, arrive: i64, leave: i64, end_of_day: i64) {
    for _ in 0..rng.range(1, if user.admin { 6 } else { 3 }) {
        let host = *rng.pick(&user.servers);
        let mut ts = rng.range(arrive, leave);
        let from = user.workstation;
        if rng.chance(0.04) {
            // faute de frappe sur le mot de passe
            b.auth(w, ts, host, "sshd", Some(pid(rng)), format_args!("Failed password for {} from {from} port {} ssh2", user.name, port(rng)));
            ts += rng.range(4, 20);
        }
        let method = if user.admin && rng.chance(0.7) { "publickey" } else { "password" };
        let duration = rng.range(MINUTE, 3 * HOUR).min(end_of_day - ts);
        ssh_session(b, w, rng, ts, host, &user.name, user.uid, from, method, duration);
        if user.admin {
            let tty = rng.range(0, 8) as u32;
            for _ in 0..rng.range(0, 5) {
                let at = ts + rng.range(2, duration.max(3));
                sudo(b, w, at, host, &user.name, tty, rng.pick(ADMIN_COMMANDS));
            }
        }
    }
}

fn web_activity(b: &mut Batch, rng: &mut Rng, user: &User, arrive: i64, leave: i64) {
    let ip = user.workstation;
    let login_ts = arrive + rng.range(30, 600);
    b.access(login_ts, ip, None, "POST", "/login", 302, Some(rng.range(80, 120) as u64), rng.range(40, 200) as u64);
    for _ in 0..rng.range(20, 150) {
        let ts = rng.range(login_ts + 1, leave);
        let user = Some(user.name.as_str());
        let (method, path, status, bytes): (&str, String, u16, Option<i64>) = match rng.range(0, 100) {
            0..10 => ("GET", "/".into(), 200, Some(rng.range(2_000, 6_000))),
            10..30 => ("GET", "/dashboard".into(), 200, Some(rng.range(15_000, 60_000))),
            30..55 => ("GET", format!("/api/orders?id={}", rng.range(10_000, 99_999)), 200, Some(rng.range(500, 4_000))),
            55..70 => {
                let status = if rng.chance(0.03) { 404 } else { 200 };
                ("GET", format!("/api/clients/{}", rng.range(1, 30_000)), status, Some(rng.range(300, 2_500)))
            }
            // ressources en cache : pas de corps de réponse, la taille est notée « - »
            70..90 => ("GET", rng.pick(&["/static/app.js", "/static/style.css", "/static/logo.svg"]).to_string(), 304, None),
            90..96 => ("POST", "/api/orders".into(), 201, Some(rng.range(100, 400))),
            96..98 => ("GET", format!("/api/export?table=orders&page={}", rng.range(1, 5)), 200, Some(rng.range(50_000, 800_000))),
            _ => ("GET", "/api/report".into(), 500, Some(rng.range(200, 600))),
        };
        let duration = bytes.map_or(2, |b| 5 + b / 5_000 + rng.range(0, 80)) as u64;
        b.access(ts, ip, user, method, path, status, bytes.map(|b| b as u64), duration);
    }
}

pub fn cron_day(b: &mut Batch, w: &World, rng: &mut Rng, day: i64) {
    let midnight = w.midnight(day);
    for host in 1..HOSTS.len() {
        for hour in 0..24 {
            let ts = midnight + hour * HOUR + rng.range(0, 3);
            let pid = Some(pid(rng));
            b.auth(w, ts, host, "CRON", pid, "pam_unix(cron:session): session opened for user root(uid=0) by (uid=0)");
            b.auth(w, ts + rng.range(1, 30), host, "CRON", pid, "pam_unix(cron:session): session closed for user root");
        }
    }
}

/// Adresse « Internet » : plages de documentation (RFC 5737) ou partagées (RFC 6598), jamais attribuables.
pub fn internet_ip(rng: &mut Rng, excluded: &[Ipv4Addr]) -> Ipv4Addr {
    loop {
        let ip = if rng.chance(0.7) {
            let n = rng.range(0, 1 << 22) as u32;
            Ipv4Addr::from(u32::from(Ipv4Addr::new(100, 64, 0, 0)) | n)
        } else {
            let net = *rng.pick(&[[192, 0, 2], [198, 51, 100], [203, 0, 113]]);
            Ipv4Addr::new(net[0], net[1], net[2], rng.range(1, 255) as u8)
        };
        if !excluded.contains(&ip) {
            return ip;
        }
    }
}

const SCANNER_LOGINS: &[&str] = &["root", "admin", "test", "oracle", "ubuntu", "pi", "user", "guest", "postgres", "ftp", "git", "deploy"];
const SCANNER_PATHS: &[&str] = &["/wp-login.php", "/.env", "/phpmyadmin/", "/admin", "/.git/config", "/cgi-bin/luci", "/actuator/health", "/server-status"];

pub fn noise_day(b: &mut Batch, w: &World, rng: &mut Rng, day: i64, noise: f64, excluded: &[Ipv4Addr]) {
    let midnight = w.midnight(day);
    let ssh_scanners = (noise * rng.range(300, 600) as f64) as usize;
    for _ in 0..ssh_scanners {
        let ip = internet_ip(rng, excluded);
        let mut ts = midnight + rng.range(0, DAY - 5 * MINUTE);
        let pid = Some(pid(rng));
        let port = port(rng);
        for _ in 0..rng.range(1, 9) {
            let login = *rng.pick(SCANNER_LOGINS);
            if login == "root" {
                b.auth(w, ts, BASTION, "sshd", pid, format_args!("Failed password for root from {ip} port {port} ssh2"));
            } else {
                b.auth(w, ts, BASTION, "sshd", pid, format_args!("Invalid user {login} from {ip} port {port}"));
                b.auth(w, ts + 2, BASTION, "sshd", pid, format_args!("Failed password for invalid user {login} from {ip} port {port} ssh2"));
            }
            ts += rng.range(3, 8);
        }
        b.auth(w, ts, BASTION, "sshd", pid, format_args!("Connection closed by {ip} port {port} [preauth]"));
    }
    let web_scanners = (noise * rng.range(50, 150) as f64) as usize;
    for _ in 0..web_scanners {
        let ip = internet_ip(rng, excluded);
        let mut ts = midnight + rng.range(0, DAY - 5 * MINUTE);
        for _ in 0..rng.range(3, 20) {
            b.access(ts, ip, None, "GET", rng.pick(SCANNER_PATHS), 404, Some(rng.range(150, 300) as u64), rng.range(1, 5) as u64);
            ts += rng.range(0, 4);
        }
    }
}
