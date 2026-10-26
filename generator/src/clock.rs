//! Conversion secondes Unix <-> date civile UTC (algorithmes de H. Hinnant,
//! <https://howardhinnant.github.io/date_algorithms.html>).

use std::fmt;

pub const DAY: i64 = 86_400;
pub const HOUR: i64 = 3_600;
pub const MINUTE: i64 = 60;

// Constantes des algorithmes de H. Hinnant (« chrono-Compatible Low-Level Date
// Algorithms »). L'année y est décalée pour commencer le 1er mars : le 29 février devient
// le dernier jour de l'année, et la longueur des mois suit un motif régulier.

/// Le calendrier grégorien se répète à l'identique tous les 400 ans : une « ère ».
const YEARS_PER_ERA: i64 = 400;
/// Jours d'une année commune.
const DAYS_PER_YEAR: i64 = 365;
/// Une année sur 4 est bissextile…
const LEAP_EVERY: i64 = 4;
/// … sauf les années séculaires, hormis celles multiples de 400 (une par ère).
const CENTURY: i64 = 100;
/// Jours d'une ère : 400 × 365 + 97 jours bissextiles = 146 097.
const DAYS_PER_ERA: i64 =
    YEARS_PER_ERA * DAYS_PER_YEAR + YEARS_PER_ERA / LEAP_EVERY - YEARS_PER_ERA / CENTURY + 1;
/// Jours de 4 années communes : 1 460, un cycle bissextile sans son 29 février.
const DAYS_PER_4_YEARS: i64 = LEAP_EVERY * DAYS_PER_YEAR;
/// Jours d'un siècle sans année multiple de 400 : 100 × 365 + 24 jours bissextiles = 36 524.
const DAYS_PER_CENTURY: i64 = CENTURY * DAYS_PER_YEAR + CENTURY / LEAP_EVERY - 1;
/// Jours entre le début de l'ère 0 (1er mars de l'an 0) et le 1970-01-01.
const DAYS_TO_UNIX_EPOCH: i64 = 719_468;
const MONTHS_PER_YEAR: i64 = 12;
/// Premier mois de l'année décalée.
const MARCH: i64 = 3;
/// De mars à juillet comme d'août à décembre, 5 mois font 153 jours (31 30 31 30 31) :
/// le mois `m` (mars = 0) commence au jour `(153 × m + 2) / 5` de l'année décalée,
/// l'arrondi `+ 2` plaçant les mois de 30 jours (0, 31, 61, 92, 122, 153…).
const DAYS_PER_5_MONTHS: i64 = 153;
const MONTHS_PER_CYCLE: i64 = 5;
const MONTH_ROUNDING: i64 = 2;

/// Nombre de jours depuis le 1970-01-01.
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    // janvier et février appartiennent à l'année décalée précédente
    let year = if i64::from(month) < MARCH {
        year - 1
    } else {
        year
    };
    let era = year.div_euclid(YEARS_PER_ERA);
    let yoe = year.rem_euclid(YEARS_PER_ERA); // année de l'ère
    let mp = (month as i64 - MARCH).rem_euclid(MONTHS_PER_YEAR); // mars = 0
    let doy = (DAYS_PER_5_MONTHS * mp + MONTH_ROUNDING) / MONTHS_PER_CYCLE + day as i64 - 1;
    let doe = yoe * DAYS_PER_YEAR + yoe / LEAP_EVERY - yoe / CENTURY + doy; // jour de l'ère
    era * DAYS_PER_ERA + doe - DAYS_TO_UNIX_EPOCH
}

pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + DAYS_TO_UNIX_EPOCH;
    let era = z.div_euclid(DAYS_PER_ERA);
    let doe = z.rem_euclid(DAYS_PER_ERA); // jour de l'ère
    // Retire les jours bissextiles écoulés (un tous les 4 ans, sauf un par siècle, plus
    // celui de l'an 400) pour se ramener à des années de 365 jours. Chaque diviseur est
    // un cycle moins son dernier jour (1 461 − 1, 36 525 − 1, 146 097 − 1) : le 29 février
    // qui clôt le cycle reste ainsi dans l'année qu'il termine.
    let yoe = (doe - doe / DAYS_PER_4_YEARS + doe / DAYS_PER_CENTURY - doe / (DAYS_PER_ERA - 1))
        / DAYS_PER_YEAR;
    let doy = doe - (DAYS_PER_YEAR * yoe + yoe / LEAP_EVERY - yoe / CENTURY);
    let mp = (MONTHS_PER_CYCLE * doy + MONTH_ROUNDING) / DAYS_PER_5_MONTHS; // mars = 0
    let day = (doy - (DAYS_PER_5_MONTHS * mp + MONTH_ROUNDING) / MONTHS_PER_CYCLE + 1) as u32;
    let month = (mp + MARCH - 1) % MONTHS_PER_YEAR + 1;
    // janvier et février appartiennent à l'année décalée précédente
    let year = yoe + era * YEARS_PER_ERA + i64::from(month < MARCH);
    (year, month as u32, day)
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
        assert_eq!(
            Iso(days_from_civil(2026, 11, 9) * DAY + 8 * HOUR + 5).to_string(),
            "2026-11-09T08:00:05Z"
        );
        // le 9 novembre 2026 est un lundi (le 1970-01-01 était un jeudi)
        assert_eq!((days_from_civil(2026, 11, 9) + 3).rem_euclid(7), 0);
    }
}
