// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Date normalisation for the `date` field.
//!
//! Accepts RFC 3339 timestamps, ISO 8601 dates and slash-separated dates
//! (`DD/MM/YYYY` by default, `MM/DD/YYYY` with [`DateOrder::MonthFirst`])
//! and renders every one of them as `YYYY-MM-DD`. Built on the `time`
//! crate so it works without `std`.

use super::DateOrder;
use crate::error::MetadataError;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use time::format_description::well_known::{Iso8601, Rfc3339};
use time::{Date, OffsetDateTime};

/// Standardizes the date format.
///
/// Attempts to parse the supported formats and converts them to
/// `YYYY-MM-DD`.
///
/// # Errors
///
/// Returns a `MetadataError::DateParseError` if the date cannot be parsed
/// or is invalid.
pub(super) fn standardize_date(
    date: &str,
    order: DateOrder,
) -> Result<String, MetadataError> {
    reject_degenerate(date)?;
    let date = normalise_slash_date(date, order)?;
    let parsed = parse_lenient(&date)?;
    // Format the date to the standardized YYYY-MM-DD format
    Ok(format!(
        "{:04}-{:02}-{:02}",
        parsed.year(),
        parsed.month() as u8,
        parsed.day()
    ))
}

/// Rejects empty and too-short inputs before any parser sees them.
fn reject_degenerate(date: &str) -> Result<(), MetadataError> {
    if date.trim().is_empty() {
        return Err(MetadataError::DateParseError(
            "Date string is empty.".to_string(),
        ));
    }
    if date.chars().count() < 8 {
        return Err(MetadataError::DateParseError(
            "Date string is too short.".to_string(),
        ));
    }
    Ok(())
}

/// Rewrites a slash-separated date as `YYYY-MM-DD` in the given order; any
/// other slash-separated ten-character shape is an error. Inputs without a
/// slash pass through.
fn normalise_slash_date(
    date: &str,
    order: DateOrder,
) -> Result<String, MetadataError> {
    if !(date.contains('/') && date.len() == 10) {
        return Ok(date.to_string());
    }
    let [first, second, year] =
        slash_parts(date).ok_or_else(|| invalid_slash_date(order))?;
    let (month, day) = match order {
        DateOrder::DayFirst => (second, first),
        DateOrder::MonthFirst => (first, second),
    };
    Ok(format!("{}-{}-{}", year, month, day))
}

/// The three parts of a `NN/NN/NNNN` date, or `None` for any other shape.
fn slash_parts(date: &str) -> Option<[&str; 3]> {
    let parts: Vec<&str> = date.split('/').collect();
    match parts.as_slice() {
        [a, b, y] if a.len() == 2 && b.len() == 2 && y.len() == 4 => {
            Some([a, b, y])
        }
        _ => None,
    }
}

/// The error for a slash-separated date that is not in `order`'s layout.
fn invalid_slash_date(order: DateOrder) -> MetadataError {
    let layout = match order {
        DateOrder::DayFirst => "DD/MM/YYYY",
        DateOrder::MonthFirst => "MM/DD/YYYY",
    };
    MetadataError::DateParseError(format!(
        "Invalid {layout} date format."
    ))
}

/// RFC 3339 first (keeps the offset's date), then a bare ISO 8601 date.
/// A string with a time component that is not RFC 3339 is rejected rather
/// than silently truncated to its date.
fn parse_lenient(date: &str) -> Result<Date, MetadataError> {
    if let Ok(odt) = OffsetDateTime::parse(date, &Rfc3339) {
        return Ok(odt.date());
    }
    if !date.contains('T') && !date.contains(' ') {
        if let Ok(d) = Date::parse(date, &Iso8601::DATE) {
            return Ok(d);
        }
    }
    Err(MetadataError::DateParseError(format!(
        "Failed to parse date: {date:?} is not RFC 3339, ISO 8601 or a slash-separated date"
    )))
}
