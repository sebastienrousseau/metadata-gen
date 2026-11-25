// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ISO 8601 dates without a time: calendar (`2024-03-21`, `20240321`),
//! ordinal (`2024-081`, `2024081`) and week (`2024-W12-4`, `2024W124`)
//! forms, with an optional sign and six-digit year (`+002024-03-21`).
//!
//! It accepts exactly what `time` 0.3's `Date::parse(s, &Iso8601::DATE)`
//! accepts for input with no `T` (the only input
//! [`super::date`] hands it) and returns the same date. It exists
//! because that generic parser compiled 3.2 KB into this crate (#61);
//! the tests below hold the two to the same answers.
//!
//! The rules, from `time`'s `Iso8601::parse_date`: a year (four digits,
//! or a sign and six), then `-` for the extended format or nothing for
//! the basic one, then the first of month-day, ordinal and week whose
//! digits are there. A form whose digits are there but whose values are
//! out of range rejects the input; the next form is not tried. Anything
//! left over rejects it too: after a date, a time needs a `T`.

/// A calendar date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Ymd {
    pub(super) year: i32,
    pub(super) month: u8,
    pub(super) day: u8,
}

/// `time`'s year range without its `large-dates` feature.
const MIN_YEAR: i32 = -9999;
const MAX_YEAR: i32 = 9999;

/// One date form's answer: `None` when its digits are not there (try the
/// next form), `Some(None)` when they are but the date is invalid.
type Form<'a> = Option<(Option<Ymd>, &'a [u8])>;

/// The date `s` names, or `None` when `time` would reject it.
pub(super) fn parse(s: &str) -> Option<Ymd> {
    let (year, rest) = year(s.as_bytes())?;
    let (extended, rest) = match rest {
        [b'-', rest @ ..] => (true, rest),
        rest => (false, rest),
    };
    let (date, rest) = calendar(year, extended, rest)
        .or_else(|| ordinal(year, rest))
        .or_else(|| week(year, extended, rest))?;
    date.filter(|_| rest.is_empty())
}

/// Four digits, or `+`/`-` and six.
fn year(s: &[u8]) -> Option<(i32, &[u8])> {
    match s {
        [sign @ (b'+' | b'-'), rest @ ..] => {
            let (value, rest) = digits(rest, 6)?;
            let value = value as i32;
            Some((if *sign == b'-' { -value } else { value }, rest))
        }
        _ => digits(s, 4).map(|(value, rest)| (value as i32, rest)),
    }
}

/// Exactly `n` ASCII digits as a number, and what follows them.
fn digits(s: &[u8], n: usize) -> Option<(u32, &[u8])> {
    let (head, rest) = s.split_at_checked(n)?;
    let mut value = 0;
    for &b in head {
        if !b.is_ascii_digit() {
            return None;
        }
        value = value * 10 + u32::from(b - b'0');
    }
    Some((value, rest))
}

/// `MM-DD` (extended) or `MMDD` (basic).
fn calendar(year: i32, extended: bool, s: &[u8]) -> Form<'_> {
    let (month, rest) =
        digits(s, 2).filter(|(m, _)| (1..=12).contains(m))?;
    let rest = match (extended, rest) {
        (true, [b'-', rest @ ..]) => rest,
        (true, _) => return None,
        (false, rest) => rest,
    };
    let (day, rest) = digits(rest, 2).filter(|&(d, _)| d != 0)?;
    let valid = in_range(year)
        && day <= 31
        && day <= u32::from(days_in_month(year, month as u8));
    let date = Ymd {
        year,
        month: month as u8,
        day: day as u8,
    };
    Some((valid.then_some(date), rest))
}

/// `DDD`, the day of the year.
fn ordinal(year: i32, s: &[u8]) -> Form<'_> {
    let (day, rest) = digits(s, 3).filter(|&(d, _)| d != 0)?;
    let valid = in_range(year) && day <= u32::from(days_in_year(year));
    Some((valid.then(|| from_ordinal(year, day as u16)), rest))
}

/// `Www-D` (extended) or `WwwD` (basic).
fn week(year: i32, extended: bool, s: &[u8]) -> Form<'_> {
    let [b'W', rest @ ..] = s else { return None };
    let (week, rest) = digits(rest, 2).filter(|&(w, _)| w != 0)?;
    let rest = match (extended, rest) {
        (true, [b'-', rest @ ..]) => rest,
        (true, _) => return None,
        (false, rest) => rest,
    };
    let (weekday, rest) =
        digits(rest, 1).filter(|w| (1..=7).contains(&w.0))?;
    let valid =
        in_range(year) && week <= u32::from(weeks_in_year(year));
    let date =
        valid.then(|| from_week(year, week as u8, weekday as u8));
    Some((date.flatten(), rest))
}

fn in_range(year: i32) -> bool {
    (MIN_YEAR..=MAX_YEAR).contains(&year)
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_year(year: i32) -> u16 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The calendar date of day `ordinal` (1-based) of `year`.
fn from_ordinal(year: i32, mut ordinal: u16) -> Ymd {
    let mut month = 1;
    while ordinal > u16::from(days_in_month(year, month)) {
        ordinal -= u16::from(days_in_month(year, month));
        month += 1;
    }
    Ymd {
        year,
        month,
        day: ordinal as u8,
    }
}

/// Days from 0001-01-01 to January 1st of `year`, as `time` counts
/// them (floor division, so it holds for years before 1).
fn days_before(year: i32) -> i32 {
    let y = year - 1;
    365 * y + y.div_euclid(4) - y.div_euclid(100) + y.div_euclid(400)
}

/// 0 for Monday through 6 for Sunday: January 1st of year 1 was a
/// Monday.
fn jan_1_weekday(year: i32) -> i32 {
    days_before(year).rem_euclid(7)
}

/// 53 when the year starts on a Thursday, or on a Wednesday in a leap
/// year; 52 otherwise.
fn weeks_in_year(year: i32) -> u8 {
    match jan_1_weekday(year) {
        3 => 53,
        2 if is_leap_year(year) => 53,
        _ => 52,
    }
}

/// `time`'s `Date::from_iso_week_date`, branch for branch.
fn from_week(year: i32, week: u8, weekday: u8) -> Option<Ymd> {
    // The ordinal of the Monday before week 1, plus the days into it:
    // week 1 is the one holding January 4th.
    let jan_4_offset = (jan_1_weekday(year) + 3).rem_euclid(7);
    let ordinal =
        i32::from(week) * 7 + i32::from(weekday) - 4 - jan_4_offset;
    if ordinal <= 0 {
        // As in `time`, the previous year is not range-checked. It never
        // needs to be: January 1st of -9999 is a Monday, so week 1 of
        // the first ISO year starts on it.
        let ordinal = ordinal + i32::from(days_in_year(year - 1));
        return Some(from_ordinal(year - 1, ordinal as u16));
    }
    let days = i32::from(days_in_year(year));
    if ordinal > days {
        // `time` refuses to roll week 52/53 of 9999 into 10000.
        return (year != MAX_YEAR)
            .then(|| from_ordinal(year + 1, (ordinal - days) as u16));
    }
    Some(from_ordinal(year, ordinal as u16))
}

#[cfg(test)]
mod tests;
