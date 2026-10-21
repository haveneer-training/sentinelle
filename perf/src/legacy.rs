//! Le parseur et le comptage de la v0.1 (`legacy/src/main.rs`), recopiés à l'identique
//! pour être mesurés à côté de ceux du fil rouge.

#![allow(dead_code)] // les champs non lus sont un défaut de la v0.1, conservé
#![allow(clippy::all)]

#[derive(Debug, Clone)]
pub struct AuthEntry {
    timestamp: String,
    host: String,
    process: String,
    message: String,
    kind: String,
    user: String,
    ip: String,
}

fn field(parts: &Vec<String>, i: usize) -> String {
    if i < parts.len() {
        parts[i].clone()
    } else {
        String::new()
    }
}

pub fn parse_auth_line(line: String) -> Option<AuthEntry> {
    let parts: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();
    if parts.len() < 4 {
        return None;
    }
    let timestamp = parts[0].clone();
    let host = parts[1].clone();
    let mut process = parts[2].clone();
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

pub fn count_failures_by_ip(entries: Vec<AuthEntry>) -> Vec<(String, i32)> {
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

/// Toutes les entrées d'un texte, comme le fait `main` dans la v0.1.
pub fn parse_all(text: &str) -> Vec<AuthEntry> {
    text.lines()
        .filter_map(|line| parse_auth_line(line.to_string()))
        .collect()
}
