//! Étape 2 : analyser une ligne de `access.log`.
#![cfg(feature = "step2")]

use sentinelle::ParseError;
use sentinelle::time::parse_timestamp;
use sentinelle::web::{WebEntry, parse_web_line};

#[test]
fn parses_an_authenticated_request() {
    let line = r#"10.1.0.42 - v.michel [2026-11-09T09:43:02Z] "GET /api/orders?id=48213 HTTP/1.1" 200 1834 44"#;
    assert_eq!(
        parse_web_line(line),
        Ok(WebEntry {
            ip: "10.1.0.42".parse().unwrap(),
            user: Some("v.michel"),
            timestamp: parse_timestamp("2026-11-09T09:43:02Z").unwrap(),
            method: "GET",
            path: "/api/orders?id=48213",
            status: 200,
            bytes: Some(1834),
            duration_ms: 44,
        })
    );
}

#[test]
fn dashes_mean_none() {
    let line = r#"203.0.113.88 - - [2026-11-09T00:15:39Z] "GET /static/app.js HTTP/1.1" 304 - 2"#;
    let entry = parse_web_line(line).unwrap();
    assert_eq!(entry.user, None);
    assert_eq!(entry.bytes, None);
    assert_eq!(entry.status, 304);
}

#[test]
fn handles_volumes_beyond_u32() {
    let line = r#"198.51.100.9 - x.y [2026-11-13T03:04:47Z] "GET /api/export?table=clients&page=1 HTTP/1.1" 200 5506054083 900"#;
    assert_eq!(parse_web_line(line).unwrap().bytes, Some(5_506_054_083));
}

#[test]
fn rejects_malformed_lines() {
    let ok = r#"10.1.0.42 - v.michel [2026-11-09T09:43:02Z] "GET / HTTP/1.1" 200 1834 44"#;
    assert!(parse_web_line(ok).is_ok());
    assert_eq!(parse_web_line(""), Err(ParseError::Empty));
    // chaque troncature de la ligne valide doit être refusée, sans paniquer (sauf celle
    // qui ne fait que raccourcir le dernier nombre, toujours lisible)
    for end in 1..ok.len() - 1 {
        assert!(parse_web_line(&ok[..end]).is_err(), "tronquée : {}", &ok[..end]);
    }
    assert_eq!(parse_web_line(&format!("{ok} 7")), Err(ParseError::BadNumber("champ en trop")));
    assert_eq!(parse_web_line(&ok.replace("10.1.0.42", "10.1.0")), Err(ParseError::BadIp));
    assert_eq!(parse_web_line(&ok.replace(" 200 ", " OK ")), Err(ParseError::BadNumber("status")));
}
