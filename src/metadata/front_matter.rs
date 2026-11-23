// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Front-matter parsing: each compiled-in format turns the raw block into
//! the dotted-key map `Metadata` holds, or into a typed value.
//!
//! Every parser runs under [`ParseLimits`]: the block size is checked
//! before any parser sees it, and the YAML parser additionally gets the
//! strict preset of its own resource budgets.

use super::{FrontMatterFormat, Metadata, ParseLimits};
use crate::error::MetadataError;
use crate::MetadataMap;
use alloc::format;
#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
use alloc::string::String;
use alloc::string::ToString;
#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
use alloc::vec::Vec;
#[cfg(any(feature = "yaml", feature = "json"))]
use core::ops::Range;

/// Parses `raw` as `format` into a flat [`Metadata`].
pub(super) fn parse_block(
    format: FrontMatterFormat,
    raw: &str,
    limits: &ParseLimits,
) -> Result<Metadata, MetadataError> {
    check_size(raw, limits)?;
    let map: Result<MetadataMap, MetadataError> = match format {
        #[cfg(feature = "yaml")]
        FrontMatterFormat::Yaml => yaml::flatten_block(raw, limits),
        #[cfg(feature = "toml")]
        FrontMatterFormat::Toml => toml_fm::flatten_block(raw),
        #[cfg(feature = "json")]
        FrontMatterFormat::Json => json::flatten_block(raw),
        #[cfg(not(all(
            feature = "yaml",
            feature = "toml",
            feature = "json"
        )))]
        other => Err(MetadataError::UnsupportedFormatError(
            other.to_string(),
        )),
    };
    map.map(Metadata::new)
}

/// Deserialises `raw` as `format` into an owned `T`.
pub(super) fn deserialize_block<
    T: serde::de::DeserializeOwned + 'static,
>(
    format: FrontMatterFormat,
    raw: &str,
    limits: &ParseLimits,
) -> Result<T, MetadataError> {
    check_size(raw, limits)?;
    match format {
        #[cfg(feature = "yaml")]
        FrontMatterFormat::Yaml => yaml::deserialize(raw, limits),
        #[cfg(feature = "toml")]
        FrontMatterFormat::Toml => toml_fm::deserialize(raw),
        #[cfg(feature = "json")]
        FrontMatterFormat::Json => json::deserialize(raw),
        #[cfg(not(all(
            feature = "yaml",
            feature = "toml",
            feature = "json"
        )))]
        other => Err(MetadataError::UnsupportedFormatError(
            other.to_string(),
        )),
    }
}

/// Deserialises `raw` as `format` into a `T` that may borrow from `raw`.
pub(super) fn deserialize_borrowed<'a, T: serde::Deserialize<'a>>(
    format: FrontMatterFormat,
    raw: &'a str,
    limits: &ParseLimits,
) -> Result<T, MetadataError> {
    check_size(raw, limits)?;
    match format {
        #[cfg(feature = "yaml")]
        FrontMatterFormat::Yaml => {
            yaml::deserialize_borrowed(raw, limits)
        }
        #[cfg(feature = "toml")]
        FrontMatterFormat::Toml => toml_fm::deserialize(raw),
        #[cfg(feature = "json")]
        FrontMatterFormat::Json => json::deserialize(raw),
        #[cfg(not(all(
            feature = "yaml",
            feature = "toml",
            feature = "json"
        )))]
        other => Err(MetadataError::UnsupportedFormatError(
            other.to_string(),
        )),
    }
}

pub(super) fn check_size(
    raw: &str,
    limits: &ParseLimits,
) -> Result<(), MetadataError> {
    if raw.len() > limits.max_block_bytes {
        return Err(MetadataError::ExtractionError {
            message: format!(
                "Front matter block of {} bytes exceeds the {}-byte limit.",
                raw.len(),
                limits.max_block_bytes
            ),
        });
    }
    Ok(())
}

/// Byte offset of a 1-based `line` and `column` inside `text`, clamped to
/// the text, as a one-byte span.
#[cfg(any(feature = "yaml", feature = "json"))]
fn line_col_span(
    text: &str,
    line: usize,
    column: usize,
) -> Range<usize> {
    let mut offset = 0;
    for (i, l) in text.split('\n').enumerate() {
        if i + 1 == line.max(1) {
            offset += column.saturating_sub(1).min(l.len());
            break;
        }
        offset += l.len() + 1;
    }
    let start = offset.min(text.len());
    start..(start + 1).min(text.len()).max(start)
}

/// Joins a nested key onto its dotted prefix (`author` + `name` gives
/// `author.name`); an empty prefix yields the key unchanged.
#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
fn join_key(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{}.{}", prefix, key)
    }
}

/// Renders a sequence as `[a, b, c]`, the shape every front-matter format
/// shares for lists.
#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
fn inline_list<I: Iterator<Item = String>>(items: I) -> String {
    format!("[{}]", items.collect::<Vec<String>>().join(", "))
}

#[cfg(feature = "yaml")]
pub(super) mod yaml;

#[cfg(feature = "toml")]
pub(super) mod toml_fm {
    use super::*;
    use toml::Value as TomlValue;

    fn parse_error(e: toml::de::Error) -> MetadataError {
        let span = e.span();
        MetadataError::parse(FrontMatterFormat::Toml, span, e)
    }

    pub(in crate::metadata) fn flatten_block(
        raw: &str,
    ) -> Result<MetadataMap, MetadataError> {
        let value: TomlValue =
            toml::from_str(raw).map_err(parse_error)?;
        let mut map = MetadataMap::new();
        flatten_toml(&value, &mut map, String::new());
        Ok(map)
    }

    pub(in crate::metadata) fn deserialize<
        'a,
        T: serde::Deserialize<'a>,
    >(
        raw: &'a str,
    ) -> Result<T, MetadataError> {
        toml::from_str::<T>(raw).map_err(parse_error)
    }

    /// Walks the block with the TOML deserialiser.
    pub(in crate::metadata) fn walk<'a, S>(
        raw: &'a str,
        seed: S,
    ) -> Result<(), MetadataError>
    where
        S: serde::de::DeserializeSeed<'a, Value = ()>,
    {
        let de = toml::Deserializer::parse(raw).map_err(parse_error)?;
        seed.deserialize(de).map_err(parse_error)
    }

    /// Flattens a TOML value: tables become dotted keys, arrays render as
    /// `[a, b]`, strings lose their quotes.
    pub(in crate::metadata) fn flatten_toml(
        value: &TomlValue,
        map: &mut MetadataMap,
        prefix: String,
    ) {
        match value {
            TomlValue::Table(table) => {
                for (k, v) in table {
                    flatten_toml(v, map, join_key(&prefix, k));
                }
            }
            TomlValue::Array(arr) => {
                map.insert(
                    prefix,
                    inline_list(arr.iter().map(toml_scalar)),
                );
            }
            _ => {
                map.insert(prefix, toml_scalar(value));
            }
        }
    }

    /// A TOML leaf as text: strings lose their double quotes, datetimes keep
    /// their own rendering, everything else uses the value's `Display`.
    fn toml_scalar(value: &TomlValue) -> String {
        match value {
            TomlValue::String(s) => s.clone(),
            TomlValue::Datetime(dt) => dt.to_string(),
            other => other.to_string(),
        }
    }
}

#[cfg(feature = "json")]
pub(super) mod json {
    use super::*;
    use serde_json::Value as JsonValue;

    fn parse_error(text: &str, e: serde_json::Error) -> MetadataError {
        let span = Some(line_col_span(text, e.line(), e.column()));
        MetadataError::parse(FrontMatterFormat::Json, span, e)
    }

    /// Parses the JSON object and flattens it. Nested objects and arrays
    /// of objects are preserved by flattening with dot-separated keys
    /// (`author.name`, matching the YAML/TOML shape). Issue #26.
    pub(in crate::metadata) fn flatten_block(
        raw: &str,
    ) -> Result<MetadataMap, MetadataError> {
        let object: serde_json::Map<String, JsonValue> =
            serde_json::from_str(raw)
                .map_err(|e| parse_error(raw, e))?;
        let mut map = MetadataMap::new();
        for (k, v) in object {
            flatten_json(&v, &mut map, k);
        }
        Ok(map)
    }

    pub(in crate::metadata) fn deserialize<
        'a,
        T: serde::Deserialize<'a>,
    >(
        raw: &'a str,
    ) -> Result<T, MetadataError> {
        serde_json::from_str::<T>(raw).map_err(|e| parse_error(raw, e))
    }

    /// Walks the block with the streaming JSON deserialiser.
    pub(in crate::metadata) fn walk<'a, S>(
        raw: &'a str,
        seed: S,
    ) -> Result<(), MetadataError>
    where
        S: serde::de::DeserializeSeed<'a, Value = ()>,
    {
        let mut de = serde_json::Deserializer::from_str(raw);
        seed.deserialize(&mut de)
            .and_then(|()| de.end())
            .map_err(|e| parse_error(raw, e))
    }

    /// Mirrors `flatten_toml` / `flatten_yaml`: nested objects use
    /// dotted keys, arrays render as `[a, b]`.
    pub(in crate::metadata) fn flatten_json(
        value: &JsonValue,
        map: &mut MetadataMap,
        prefix: String,
    ) {
        match value {
            JsonValue::Object(obj) => {
                for (k, v) in obj {
                    flatten_json(v, map, join_key(&prefix, k));
                }
            }
            JsonValue::Array(arr) => {
                // Arrays of scalars render as `[a, b, c]`. Arrays of objects
                // render the same; callers that need element-level access
                // should use the typed-extraction API (issue #45).
                map.insert(
                    prefix,
                    inline_list(arr.iter().map(json_scalar)),
                );
            }
            _ => {
                map.insert(prefix, json_scalar(value));
            }
        }
    }

    /// A JSON leaf as text: strings lose their double quotes, `null` is the
    /// literal word, everything else uses the value's `Display`.
    fn json_scalar(value: &JsonValue) -> String {
        match value {
            JsonValue::String(s) => s.clone(),
            JsonValue::Null => "null".to_string(),
            other => other.to_string(),
        }
    }
}
