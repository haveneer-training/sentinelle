//! Conversion secondes Unix <-> date civile UTC (algorithmes de H. Hinnant).

use std::fmt;

pub const DAY: i64 = 86_400;
pub const HOUR: i64 = 3_600;
pub const MINUTE: i64 = 60;

/// Nombre de jours depuis le 1970-01-01.
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = (month as i64 + 9) % 12; // mars = 0
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

/// Affiche un instant au format ISO 8601 UTC : `2026-11-09T08:12:45Z`.
pub struct Iso(pub i64);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for days in -800_000..800_000 {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days);
        }
    }

    #[test]
    fn known_dates() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(Iso(days_from_civil(2026, 11, 9) * DAY + 8 * HOUR + 5).to_string(), "2026-11-09T08:00:05Z");
        // le 9 novembre 2026 est un lundi (le 1970-01-01 était un jeudi)
        assert_eq!((days_from_civil(2026, 11, 9) + 3).rem_euclid(7), 0);
    }
}
