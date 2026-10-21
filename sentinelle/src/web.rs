//! Lignes de `access.log` :
//! `IP - USER [TIMESTAMP] "METHOD PATH HTTP/1.1" STATUS BYTES DURATION_MS`.

use crate::ParseError;
use crate::time::Timestamp;
#[cfg_attr(not(feature = "step2"), allow(unused_imports))] // trou étape 2
use crate::time::parse_timestamp;
use std::net::IpAddr;

/// Une ligne de `access.log`, analysée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebEntry<'a> {
    pub ip: IpAddr,
    /// `None` pour une requête non authentifiée (`-`).
    pub user: Option<&'a str>,
    pub timestamp: Timestamp,
    pub method: &'a str,
    pub path: &'a str,
    pub status: u16,
    /// `None` quand la réponse n'a pas de corps (`-`).
    pub bytes: Option<u64>,
    pub duration_ms: u32,
}

/// Analyse une ligne de `access.log`, sans copier aucune chaîne.
#[cfg_attr(not(feature = "step2"), allow(unused_variables))] // trou étape 2
pub fn parse_web_line(line: &str) -> Result<WebEntry<'_>, ParseError> {
    // TODO-STEP:2 Découper la ligne sur ses séparateurs successifs (` - `, ` [`, `] "`,
    // `" `), puis la requête (`METHOD PATH PROTOCOLE`) et les trois nombres de la fin. Un `-`
    // dans `USER` ou `BYTES` donne `None`. Un champ en trop est une erreur, comme un manquant.
    // SOLUTION-BEGIN
    if line.is_empty() {
        return Err(ParseError::Empty);
    }
    let (ip, rest) = line
        .split_once(" - ")
        .ok_or(ParseError::MissingField("user"))?;
    let ip: IpAddr = ip.parse().map_err(|_| ParseError::BadIp)?;
    let (user, rest) = rest
        .split_once(" [")
        .ok_or(ParseError::MissingField("timestamp"))?;
    let user = (user != "-").then_some(user);
    let (timestamp, rest) = rest
        .split_once("] \"")
        .ok_or(ParseError::MissingField("request"))?;
    let timestamp = parse_timestamp(timestamp).ok_or(ParseError::BadTimestamp)?;
    let (request, rest) = rest
        .split_once("\" ")
        .ok_or(ParseError::MissingField("status"))?;
    let (method, rest_of_request) = request
        .split_once(' ')
        .ok_or(ParseError::MissingField("path"))?;
    let (path, _protocol) = rest_of_request
        .rsplit_once(' ')
        .ok_or(ParseError::MissingField("protocol"))?;

    let mut numbers = rest.split(' ');
    let mut next = |name: &'static str| {
        numbers
            .next()
            .filter(|s| !s.is_empty())
            .ok_or(ParseError::MissingField(name))
    };
    let status = next("status")?
        .parse()
        .map_err(|_| ParseError::BadNumber("status"))?;
    let bytes = match next("bytes")? {
        "-" => None,
        bytes => Some(bytes.parse().map_err(|_| ParseError::BadNumber("bytes"))?),
    };
    let duration_ms = next("duration")?
        .parse()
        .map_err(|_| ParseError::BadNumber("duration"))?;
    if numbers.next().is_some() {
        return Err(ParseError::BadNumber("champ en trop"));
    }
    Ok(WebEntry {
        ip,
        user,
        timestamp,
        method,
        path,
        status,
        bytes,
        duration_ms,
    })
    // SOLUTION-END
}
