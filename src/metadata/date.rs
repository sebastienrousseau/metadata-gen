// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Date normalisation for the `date` field.

use crate::error::MetadataError;
use dtt::datetime::DateTime;

/// Standardizes the date format.
///
/// This function attempts to parse various date formats and convert them to the YYYY-MM-DD format.
///
/// # Arguments
///
/// * `date` - A string slice containing the date to standardize.
///
/// # Returns
///
/// A `Result` containing the standardized date string if successful, or a `MetadataError` if parsing fails.
///
/// # Errors
///
/// Returns a `MetadataError::DateParseError` if the date cannot be parsed or is invalid.
pub(super) fn standardize_date(
    date: &str,
) -> Result<String, MetadataError> {
    reject_degenerate(date)?;
    let date = normalise_slash_date(date)?;
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
    if date.len() < 8 {
        return Err(MetadataError::DateParseError(
            "Date string is too short.".to_string(),
        ));
    }
    Ok(())
}

/// Rewrites a `DD/MM/YYYY` date as `YYYY-MM-DD`; any other slash-separated
/// ten-character shape is an error. Inputs without a slash pass through.
fn normalise_slash_date(date: &str) -> Result<String, MetadataError> {
    if !(date.contains('/') && date.len() == 10) {
        return Ok(date.to_string());
    }
    let parts: Vec<&str> = date.split('/').collect();
    let well_formed = parts.len() == 3
        && parts[0].len() == 2
        && parts[1].len() == 2
        && parts[2].len() == 4;
    if !well_formed {
        return Err(MetadataError::DateParseError(
            "Invalid DD/MM/YYYY date format.".to_string(),
        ));
    }
    Ok(format!("{}-{}-{}", parts[2], parts[1], parts[0]))
}

/// Tries the ISO parser first, then the two custom layouts the crate has
/// always accepted.
fn parse_lenient(date: &str) -> Result<DateTime, MetadataError> {
    DateTime::parse(date)
        .or_else(|_| {
            DateTime::parse_custom_format(date, "[year]-[month]-[day]")
        })
        .or_else(|_| {
            DateTime::parse_custom_format(date, "[month]/[day]/[year]")
        })
        .map_err(|e| {
            MetadataError::DateParseError(format!(
                "Failed to parse date: {}",
                e
            ))
        })
}
