// Sentinelle v0.1
// Ecrit en urgence le lendemain de l'alerte. Ca marche sur mon jeu de test.
// TODO: nettoyer avant la mise en prod

use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone)]
struct AuthEntry {
    timestamp: String,
    host: String,
    process: String,
    message: String,
    kind: String, // "FAILED", "ACCEPTED", "INVALID" ou "OTHER"
    user: String,
    ip: String,
}

#[derive(Debug, Clone)]
struct WebEntry {
    ip: String,
    user: String,
    timestamp: String,
    method: String,
    path: String,
    status: String,
    bytes: String,
    duration: String,
}

fn field(parts: &Vec<String>, i: usize) -> String {
    if i < parts.len() {
        parts[i].clone()
    } else {
        String::new()
    }
}

fn parse_auth_line(line: String) -> Option<AuthEntry> {
    let parts: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();
    if parts.len() < 4 {
        return None;
    }
    let timestamp = parts[0].clone();
    let host = parts[1].clone();
    let mut process = parts[2].clone();
    // enleve le [pid] et le ':'
    if let Some(i) = process.find('[') {
        process = process[..i].to_string();
    } else {
        process = process.replace(":", "");
    }
    let message = parts[3..].join(" ");

    let mut kind = "OTHER".to_string();
    let mut user = String::new();
    let mut ip = String::new();
    if message.starts_with("Failed password for invalid user ") {
        kind = "FAILED".to_string();
        user = field(&parts, 8);
        ip = field(&parts, 10);
    } else if message.starts_with("Failed password for ") {
        kind = "FAILED".to_string();
        user = field(&parts, 6);
        ip = field(&parts, 8);
    } else if message.starts_with("Accepted ") {
        kind = "ACCEPTED".to_string();
        user = field(&parts, 6);
        ip = field(&parts, 8);
    } else if message.starts_with("Invalid user ") {
        kind = "INVALID".to_string();
        user = field(&parts, 5);
        ip = field(&parts, 7);
    }

    Some(AuthEntry {
        timestamp: timestamp,
        host: host,
        process: process,
        message: message,
        kind: kind,
        user: user,
        ip: ip,
    })
}

fn parse_web_line(line: String) -> Option<WebEntry> {
    let parts: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();
    if parts.len() < 10 {
        return None;
    }
    Some(WebEntry {
        ip: parts[0].clone(),
        user: parts[2].clone(),
        timestamp: parts[3].replace("[", "").replace("]", ""),
        method: parts[4].replace("\"", ""),
        path: parts[5].clone(),
        status: parts[7].clone(),
        bytes: parts[8].clone(),
        duration: parts[9].clone(),
    })
}

fn count_failures_by_ip(entries: Vec<AuthEntry>) -> Vec<(String, i32)> {
    let mut counts: Vec<(String, i32)> = Vec::new();
    for e in entries {
        if e.kind == "FAILED" || e.kind == "INVALID" {
            let mut found = false;
            for c in counts.iter_mut() {
                if c.0 == e.ip {
                    c.1 += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                counts.push((e.ip.clone(), 1));
            }
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts
}

fn detect_bruteforce(entries: Vec<AuthEntry>) -> Vec<String> {
    let counts = count_failures_by_ip(entries.clone());
    let mut result = Vec::new();
    for c in counts {
        if c.1 > 100 {
            result.push(c.0);
        }
    }
    result
}

fn detect_spraying(entries: Vec<AuthEntry>) -> Vec<(String, usize)> {
    let mut users_by_ip: HashMap<String, Vec<String>> = HashMap::new();
    for e in entries.iter() {
        if e.kind == "FAILED" {
            if !users_by_ip.contains_key(&e.ip) {
                users_by_ip.insert(e.ip.clone(), Vec::new());
            }
            let users = users_by_ip.get_mut(&e.ip).unwrap();
            if !users.contains(&e.user) {
                users.push(e.user.clone());
            }
        }
    }
    let mut result = Vec::new();
    for (ip, users) in users_by_ip {
        if users.len() > 20 {
            result.push((ip, users.len()));
        }
    }
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

fn bytes_by_user(entries: Vec<WebEntry>) -> Vec<(String, u64)> {
    let mut totals: HashMap<String, u64> = HashMap::new();
    for e in entries.clone() {
        let b = e.bytes.parse::<u64>().unwrap_or(0);
        if totals.contains_key(&e.user) {
            let t = totals[&e.user];
            totals.insert(e.user.clone(), t + b);
        } else {
            totals.insert(e.user.clone(), b);
        }
    }
    let mut v: Vec<(String, u64)> = totals.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() > 1 { args[1].clone() } else { "data".to_string() };
    let start = Instant::now();

    let auth_text = fs::read_to_string(format!("{}/auth.log", dir)).unwrap();
    let mut auth_entries = Vec::new();
    let mut auth_skipped = 0;
    for line in auth_text.lines() {
        match parse_auth_line(line.to_string()) {
            Some(e) => auth_entries.push(e),
            None => auth_skipped += 1,
        }
    }

    let web_text = fs::read_to_string(format!("{}/access.log", dir)).unwrap();
    let mut web_entries = Vec::new();
    let mut web_skipped = 0;
    for line in web_text.lines() {
        match parse_web_line(line.to_string()) {
            Some(e) => web_entries.push(e),
            None => web_skipped += 1,
        }
    }

    println!("=== Sentinelle v0.1 ===");
    println!("auth.log   : {} entrees ({} ignorees)", auth_entries.len(), auth_skipped);
    println!("access.log : {} entrees ({} ignorees)", web_entries.len(), web_skipped);

    println!();
    println!("--- Top 10 des IP en echec ---");
    let top = count_failures_by_ip(auth_entries.clone());
    for i in 0..10 {
        if i < top.len() {
            println!("{:>18} {:>6}", top[i].0, top[i].1);
        }
    }

    println!();
    println!("--- Force brute (plus de 100 echecs) ---");
    for ip in detect_bruteforce(auth_entries.clone()) {
        println!("{:>18}", ip);
    }

    println!();
    println!("--- Password spraying (plus de 20 comptes) ---");
    for (ip, n) in detect_spraying(auth_entries.clone()) {
        println!("{:>18} {:>6} comptes", ip, n);
    }

    println!();
    println!("--- Top 5 des volumes web par utilisateur ---");
    let volumes = bytes_by_user(web_entries.clone());
    for i in 0..5 {
        if i < volumes.len() {
            println!("{:>18} {:>14} octets", volumes[i].0, volumes[i].1);
        }
    }

    eprintln!();
    eprintln!("Analyse terminee en {:?}", start.elapsed());
}
