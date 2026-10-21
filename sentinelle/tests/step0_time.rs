//! Étape 0 : lire un horodatage.
#![cfg(feature = "step0")]

use sentinelle::time::{DAY, HOUR, Iso, days_from_civil, parse_timestamp};

#[test]
fn parses_epoch_and_a_known_date() {
    assert_eq!(parse_timestamp("1970-01-01T00:00:00Z"), Some(0));
    let expected = days_from_civil(2026, 11, 9) * DAY + 8 * HOUR + 12 * 60 + 45;
    assert_eq!(parse_timestamp("2026-11-09T08:12:45Z"), Some(expected));
}

#[test]
fn round_trips_with_display() {
    for ts in [0, 1_793_000_000, 1_794_355_199] {
        assert_eq!(parse_timestamp(&Iso(ts).to_string()), Some(ts));
    }
}

#[test]
fn rejects_wrong_shapes() {
    for text in [
        "",
        "2026-11-09",
        "2026-11-09 08:12:45Z",
        "2026-11-09T08:12:45",
        "2026-11-09T08:12:45+01:00",
        "2026/11/09T08:12:45Z",
        "2026-11-09T08:12:4Z",
        "2026-11-09T08:12:455Z",
        "2026-1a-09T08:12:45Z",
        "+026-11-09T08:12:45Z",
    ] {
        assert_eq!(
            parse_timestamp(text),
            None,
            "« {text} » devrait être refusé"
        );
    }
}

#[test]
fn rejects_out_of_range_values() {
    for text in [
        "2026-13-09T08:12:45Z",
        "2026-00-09T08:12:45Z",
        "2026-11-00T08:12:45Z",
        "2026-11-09T24:00:00Z",
        "2026-11-09T08:60:00Z",
    ] {
        assert_eq!(
            parse_timestamp(text),
            None,
            "« {text} » devrait être refusé"
        );
    }
}

#[test]
fn does_not_panic_on_multibyte_text() {
    // 20 octets, mais pas 20 caractères : un découpage par tranches doit rester sûr
    assert_eq!(parse_timestamp("2026-11-09T08:12:éZ"), None);
}
