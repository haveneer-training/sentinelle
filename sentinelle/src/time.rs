//! Horodatages : secondes depuis le 1er janvier 1970, UTC.

use std::fmt;

/// Instant en secondes Unix (UTC).
pub type Timestamp = i64;

pub const MINUTE: i64 = 60;
pub const HOUR: i64 = 3_600;
pub const DAY: i64 = 86_400;

/// Nombre de jours entre le 1970-01-01 et une date du calendrier grégorien.
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = (month as i64 + 9) % 12; // mars = 0
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

/// Lit un horodatage ISO 8601 UTC de la forme `2026-11-09T08:12:45Z`.
///
/// Renvoie `None` si le texte n'a pas exactement cette forme ou si une valeur sort de
/// son domaine (mois 13, 25 heures…).
///
/// ```
/// use sentinelle::time::{parse_timestamp, DAY};
///
/// assert_eq!(parse_timestamp("1970-01-02T00:00:01Z"), Some(DAY + 1));
/// assert_eq!(parse_timestamp("1970-01-02 00:00:01"), None);
/// ```
#[cfg_attr(not(feature = "step0"), allow(unused_variables))] // trou étape 0
pub fn parse_timestamp(text: &str) -> Option<Timestamp> {
    // TODO-STEP:0 Vérifier la forme `AAAA-MM-JJTHH:MM:SSZ` (20 octets, séparateurs aux
    // bonnes places, chiffres ailleurs), extraire les six nombres par tranches (`&text[a..b]`),
    // contrôler leurs bornes, puis convertir avec `days_from_civil`.
    // SOLUTION-BEGIN
    let bytes = text.as_bytes();
    if bytes.len() != 20 {
        return None;
    }
    for (i, &b) in bytes.iter().enumerate() {
        let expected_ok = match i {
            4 | 7 => b == b'-',
            10 => b == b'T',
            13 | 16 => b == b':',
            19 => b == b'Z',
            _ => b.is_ascii_digit(),
        };
        if !expected_ok {
            return None;
        }
    }
    let number = |range: std::ops::Range<usize>| text[range].parse::<u32>().ok();
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let seconds = i64::from(hour * 3600 + minute * 60 + second);
    Some(days_from_civil(i64::from(year), month, day) * DAY + seconds)
    // SOLUTION-END
}

/// Affiche un instant au format ISO 8601 : `format!("{}", Iso(ts))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iso(pub Timestamp);

impl fmt::Display for Iso {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day) = civil_from_days(self.0.div_euclid(DAY));
        let secs = self.0.rem_euclid(DAY);
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
            secs / HOUR,
            secs / MINUTE % 60,
            secs % 60
        )
    }
}
