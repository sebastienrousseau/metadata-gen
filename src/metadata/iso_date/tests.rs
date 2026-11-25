// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [`super::parse`] against `time`'s own `Date::parse(s, &Iso8601::DATE)`:
//! every calendar, ordinal and week date (valid and not) of a spread of
//! years in both formats, the year edges, and random strings over the
//! grammar's alphabet.

use super::*;
use alloc::format;
use alloc::string::String;
use time::format_description::well_known::Iso8601;

/// `time`'s answer as the same shape as ours.
fn reference(s: &str) -> Option<Ymd> {
    time::Date::parse(s, &Iso8601::DATE).ok().map(|d| Ymd {
        year: d.year(),
        month: d.month() as u8,
        day: d.day(),
    })
}

fn check(s: &str) {
    assert_eq!(parse(s), reference(s), "input: {s:?}");
}

/// Years around every rule: leap centuries, year 0, negatives, the
/// range edges, both 52- and 53-week years.
const YEARS: [i32; 18] = [
    -9999, -9998, -401, -400, -100, -1, 0, 1, 4, 100, 1900, 2000, 2004,
    2015, 2020, 2024, 9998, 9999,
];

/// `year` written the ways the grammar allows.
fn year_spellings(year: i32) -> [String; 2] {
    let signed = if year < 0 {
        format!("-{:06}", -year)
    } else {
        format!("+{year:06}")
    };
    let plain = if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else {
        signed.clone()
    };
    [plain, signed]
}

#[test]
#[cfg_attr(
    miri,
    ignore = "exhaustive; too slow under Miri, run natively"
)]
fn weeks_in_year_matches_time() {
    for year in MIN_YEAR..=MAX_YEAR {
        assert_eq!(
            weeks_in_year(year),
            time::util::weeks_in_year(year),
            "year {year}"
        );
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "exhaustive; too slow under Miri, run natively"
)]
fn calendar_dates_match_time() {
    for year in YEARS {
        for y in year_spellings(year) {
            for month in 0..=13 {
                for day in 0..=32 {
                    check(&format!("{y}-{month:02}-{day:02}"));
                    check(&format!("{y}{month:02}{day:02}"));
                }
            }
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "exhaustive; too slow under Miri, run natively"
)]
fn ordinal_dates_match_time() {
    for year in YEARS {
        for y in year_spellings(year) {
            for day in 0..=367 {
                check(&format!("{y}-{day:03}"));
                check(&format!("{y}{day:03}"));
            }
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "exhaustive; too slow under Miri, run natively"
)]
fn week_dates_match_time() {
    for year in YEARS {
        for y in year_spellings(year) {
            for week in 0..=54 {
                for weekday in 0..=8 {
                    check(&format!("{y}-W{week:02}-{weekday}"));
                    check(&format!("{y}W{week:02}{weekday}"));
                }
            }
        }
    }
}

#[test]
fn week_dates_roll_across_years_like_time() {
    // 2020-W53-7 is 2021-01-03; 2021-W01-1 is 2021-01-04; 2015-W01-1 is
    // 2014-12-29; 9999-W52-7 would be in 10000, which `time` refuses.
    assert_eq!(
        parse("2020-W53-7"),
        Some(Ymd {
            year: 2021,
            month: 1,
            day: 3
        })
    );
    assert_eq!(
        parse("2015-W01-1"),
        Some(Ymd {
            year: 2014,
            month: 12,
            day: 29
        })
    );
    assert_eq!(
        parse("2021W011"),
        Some(Ymd {
            year: 2021,
            month: 1,
            day: 4
        })
    );
    check("9999-W52-7");
    // January 1st of -9999 is a Monday, so the first ISO week of the
    // range starts on it (`time` 0.3.55, release build: (-9999, 1, 1)).
    assert_eq!(
        parse("-009999-W01-1"),
        Some(Ymd {
            year: -9999,
            month: 1,
            day: 1
        })
    );
}

#[test]
fn malformed_and_trailing_input_matches_time() {
    for s in [
        "",
        "2024",
        "2024-",
        "2024-01",
        "2024-1-01",
        "2024-01-1",
        "2024-0101",
        "202401-01",
        "2024-01-01x",
        "2024-01-01Z",
        "2024-01-01+01:00",
        "2024-W1-1",
        "2024-W011",
        "2024W01-1",
        "2024-W01",
        "+2024-01-01",
        "-2024-01-01",
        "+002024",
        "２０２４-01-01",
        "2024-001",
        "2024-366",
        "2023-366",
        "2024-000",
        "10000-01-01",
        "2024-02-30",
        "2024-13-01",
    ] {
        check(s);
    }
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig {
        cases: if cfg!(miri) { 16 } else { 20_000 },
        failure_persistence: if cfg!(miri) {
            None
        } else {
            proptest::prelude::ProptestConfig::default().failure_persistence
        },
        ..proptest::prelude::ProptestConfig::default()
    })]

    /// Strings over the grammar's own alphabet, the way `date.rs` calls
    /// the parser: never with a `T` or a space in the input.
    #[test]
    fn random_strings_match_time(s in "[0-9+W:.Zz-]{0,14}") {
        proptest::prop_assert_eq!(parse(&s), reference(&s), "input: {:?}", s);
    }
}
