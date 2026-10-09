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

fn check_size(
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
pub(super) mod yaml {
    use super::*;
    use noyalib::ParserConfig;

    fn config(limits: &ParseLimits) -> ParserConfig {
        ParserConfig::strict()
            .max_depth(limits.max_depth)
            .max_document_length(limits.max_block_bytes.max(1))
    }

    fn parse_error(text: &str, e: noyalib::Error) -> MetadataError {
        let span = e
            .location()
            .map(|loc| line_col_span(text, loc.line(), loc.column()));
        MetadataError::parse(FrontMatterFormat::Yaml, span, e)
    }

    pub(in crate::metadata) fn flatten_block(
        raw: &str,
        limits: &ParseLimits,
    ) -> Result<MetadataMap, MetadataError> {
        let collapsed = collapse_multiline_quoted_scalars(raw);
        let value: noyalib::Value =
            noyalib::from_str_with_config(&collapsed, &config(limits))
                .map_err(|e| parse_error(&collapsed, e))?;
        Ok(flatten_yaml(&value))
    }

    pub(in crate::metadata) fn deserialize<
        T: serde::de::DeserializeOwned + 'static,
    >(
        raw: &str,
        limits: &ParseLimits,
    ) -> Result<T, MetadataError> {
        let collapsed = collapse_multiline_quoted_scalars(raw);
        noyalib::from_str_with_config::<T>(&collapsed, &config(limits))
            .map_err(|e| parse_error(&collapsed, e))
    }

    pub(in crate::metadata) fn deserialize_borrowed<
        'a,
        T: serde::Deserialize<'a>,
    >(
        raw: &'a str,
        limits: &ParseLimits,
    ) -> Result<T, MetadataError> {
        // Collapsing would allocate a new buffer nothing could borrow
        // from, so the borrowed path parses the block as written.
        noyalib::from_str_borrowing_with_config::<T>(
            raw,
            &config(limits),
        )
        .map_err(|e| parse_error(raw, e))
    }

    /// Collapses multi-line double-quoted YAML scalars onto a single line,
    /// so `title: "a\n  b"` reads as `title: "a b"`.
    ///
    /// noyalib correctly enforces YAML 1.2.2 §7.3.2 (continuation must be
    /// indented more than the parent block). Human-edited frontmatter
    /// often violates this, e.g. a `url: "\n<value>"` shape where a
    /// literal newline crept in after the opening quote. PyYAML and
    /// serde_yaml fold those onto one line; this helper does the same so
    /// noyalib never sees the offending shape. Issue #20.
    ///
    /// The scan is line-based and deliberately simple: when a line ends
    /// with `: "` (key + opening quote with nothing after), walk forward
    /// joining subsequent lines until the closing `"` is found. Comments
    /// and other quote styles are not transformed.
    pub(in crate::metadata) fn collapse_multiline_quoted_scalars(
        block: &str,
    ) -> String {
        let mut out = String::with_capacity(block.len());
        let lines: Vec<&str> = block.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            match open_quoted_scalar(line) {
                Some(head) => {
                    let (joined, next) =
                        join_quoted_continuation(head, &lines, i + 1);
                    out.push_str(&joined);
                    i = next;
                }
                None => {
                    out.push_str(line);
                    i += 1;
                }
            }
            out.push('\n');
        }
        out
    }

    /// Returns the `key: "` head of a line whose double-quoted scalar opens
    /// but does not close on that line.
    fn open_quoted_scalar(line: &str) -> Option<&str> {
        let eq_pos = line.find(": \"")?;
        let after_quote = &line[eq_pos + 3..];
        after_quote.trim().is_empty().then(|| &line[..eq_pos + 3])
    }

    /// Joins continuation lines onto `head` up to and including the one that
    /// carries the closing quote. Returns the joined line and the index of
    /// the first line not consumed.
    ///
    /// Without a closing quote (pathological input) the partial text is
    /// returned as is, so the downstream parser sees the same broken content
    /// rather than silently swallowing it.
    fn join_quoted_continuation(
        head: &str,
        lines: &[&str],
        mut i: usize,
    ) -> (String, usize) {
        let mut joined = String::from(head);
        while i < lines.len() {
            let next = lines[i];
            i += 1;
            if let Some(close) = next.find('"') {
                joined.push_str(next[..close].trim_start());
                joined.push_str(&next[close..]);
                return (joined, i);
            }
            joined.push_str(next.trim_start());
            joined.push(' ');
        }
        (joined.trim_end().to_string(), i)
    }

    /// Flattens a nested YAML value into a flat key-value map.
    ///
    /// Nested keys are joined with `.` (e.g., `author.name`).
    /// Sequences are rendered as comma-separated lists wrapped in brackets.
    pub(in crate::metadata) fn flatten_yaml(
        value: &noyalib::Value,
    ) -> MetadataMap {
        let mut map = MetadataMap::new();
        flatten_yaml_recursive(value, String::new(), &mut map);
        map
    }

    pub(in crate::metadata) fn flatten_yaml_recursive(
        value: &noyalib::Value,
        prefix: String,
        map: &mut MetadataMap,
    ) {
        match value {
            noyalib::Value::Mapping(m) => {
                // In noyalib, mapping keys are `String`, so `k.as_str()`
                // already yields `&str` directly.
                for (k, v) in m {
                    flatten_yaml_recursive(
                        v,
                        join_key(&prefix, k.as_str()),
                        map,
                    );
                }
            }
            noyalib::Value::Sequence(seq) => {
                map.insert(
                    prefix,
                    inline_list(seq.iter().map(yaml_scalar)),
                );
            }
            _ => {
                map.insert(prefix, yaml_scalar(value));
            }
        }
    }

    /// A YAML leaf as text. `as_str()` returns `Some` only for
    /// `Value::String`; other scalars (numbers, bools, dates rendered as
    /// numbers, etc.) fall back to their `Display` representation.
    fn yaml_scalar(value: &noyalib::Value) -> String {
        match value.as_str() {
            Some(s) => s.to_string(),
            None => value.to_string(),
        }
    }
}

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
