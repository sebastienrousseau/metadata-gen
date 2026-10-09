// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Post-processing of extracted metadata: date normalisation, required
//! fields and derived fields such as `slug`.

use super::date::standardize_date;
use super::Metadata;
use crate::error::MetadataError;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Options for [`process_metadata_with`].
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::{process_metadata_with, Metadata, ProcessOptions};
/// use std::collections::HashMap;
///
/// let opts = ProcessOptions::default().required_fields(["title"]);
/// let mut m = HashMap::new();
/// m.insert("title".to_string(), "Hello".to_string());
/// let out = process_metadata_with(&Metadata::new(m), &opts).unwrap();
/// assert_eq!(out.get("slug").map(String::as_str), Some("hello"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProcessOptions {
    /// Keys that must be present after processing. Default: `title`, `date`.
    pub required_fields: Vec<String>,
    /// Derive `slug` from `title` when absent. Default: `true`.
    pub derive_slug: bool,
    /// How a slash-separated date (`01/02/2024`) is read.
    pub date_order: DateOrder,
}

/// The reading of a slash-separated date such as `01/02/2024`.
///
/// ISO 8601 and RFC 3339 dates are unambiguous and are accepted whatever
/// this is set to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum DateOrder {
    /// `DD/MM/YYYY` (the crate's long-standing default).
    #[default]
    DayFirst,
    /// `MM/DD/YYYY`.
    MonthFirst,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            required_fields: vec![
                "title".to_string(),
                "date".to_string(),
            ],
            derive_slug: true,
            date_order: DateOrder::DayFirst,
        }
    }
}

impl ProcessOptions {
    /// Replaces the required-field list.
    #[must_use]
    pub fn required_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_fields =
            fields.into_iter().map(Into::into).collect();
        self
    }

    /// Turns slug derivation on or off.
    #[must_use]
    pub const fn derive_slug(mut self, on: bool) -> Self {
        self.derive_slug = on;
        self
    }

    /// Sets how slash-separated dates are read.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::{process_metadata_with, DateOrder, Metadata, MetadataMap, ProcessOptions};
    ///
    /// let mut map = MetadataMap::new();
    /// map.insert("title".into(), "T".into());
    /// map.insert("date".into(), "03/04/2024".into());
    /// let opts = ProcessOptions::default().date_order(DateOrder::MonthFirst);
    /// let out = process_metadata_with(&Metadata::new(map), &opts).unwrap();
    /// assert_eq!(out.get("date").unwrap(), "2024-03-04");
    /// ```
    #[must_use]
    pub const fn date_order(mut self, order: DateOrder) -> Self {
        self.date_order = order;
        self
    }
}

/// [`process_metadata`] with caller-chosen required fields and derivations.
///
/// # Errors
///
/// [`MetadataError::DateParseError`] for an unparseable `date`;
/// [`MetadataError::MissingFieldError`] naming the first required field
/// that is absent.
pub fn process_metadata_with(
    metadata: &Metadata,
    options: &ProcessOptions,
) -> Result<Metadata, MetadataError> {
    let mut processed = metadata.clone();
    if let Some(date) = processed.get("date").cloned() {
        let standardized_date =
            standardize_date(&date, options.date_order)?;
        processed.insert("date".to_string(), standardized_date);
    }
    for field in &options.required_fields {
        if !processed.contains_key(field) {
            return Err(MetadataError::MissingFieldError(
                field.clone(),
            ));
        }
    }
    if options.derive_slug {
        generate_derived_fields(&mut processed);
    }
    Ok(processed)
}

/// Processes the extracted metadata.
///
/// This function standardizes dates, ensures required fields are present, and generates derived fields.
///
/// # Arguments
///
/// * `metadata` - A reference to the `Metadata` instance to process.
///
/// # Returns
///
/// A `Result` containing the processed `Metadata` if successful, or a `MetadataError` if processing fails.
///
/// # Errors
///
/// Returns a `MetadataError` if date standardization fails or if required fields are missing.
pub fn process_metadata(
    metadata: &Metadata,
) -> Result<Metadata, MetadataError> {
    process_metadata_with(metadata, &ProcessOptions::default())
}

/// Generates derived fields for the metadata.
///
/// Currently, this function generates a URL slug from the title if not already present.
///
/// # Arguments
///
/// * `metadata` - A mutable reference to the `Metadata` instance to update.
pub(super) fn generate_derived_fields(metadata: &mut Metadata) {
    if !metadata.contains_key("slug") {
        if let Some(title) = metadata.get("title") {
            let slug = generate_slug(title);
            metadata.insert("slug".to_string(), slug);
        }
    }
}

/// Generates a URL slug from the given title.
///
/// # Arguments
///
/// * `title` - A string slice containing the title to convert to a slug.
///
/// # Returns
///
/// A `String` containing the generated slug.
pub(super) fn generate_slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut pending_separator = false;
    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            if pending_separator && !slug.is_empty() {
                slug.push('-');
            }
            pending_separator = false;
            slug.push(ch);
        } else {
            pending_separator = true;
        }
    }
    slug
}
