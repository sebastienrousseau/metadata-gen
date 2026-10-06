// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Front-matter extraction: locating the YAML, TOML or JSON block and
//! flattening it into the dotted-key map `Metadata` holds.

use super::{Metadata, TOML_FRONT_MATTER, YAML_FRONT_MATTER};
use crate::error::MetadataError;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use toml::Value as TomlValue;

/// Extracts YAML metadata from the content.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to extract YAML metadata from.
///
/// # Returns
///
/// An `Option<Metadata>` containing the extracted metadata if successful, or `None` if extraction fails.
pub(super) fn extract_yaml_metadata(
    content: &str,
) -> Option<Result<Metadata, MetadataError>> {
    let captures = YAML_FRONT_MATTER.captures(content)?;

    let yaml_str = captures.get(1)?.as_str().trim();

    // noyalib enforces YAML 1.2.2 §7.3.2 strictly: continuation lines
    // of a multi-line double-quoted scalar must be indented more than
    // the parent block. PyYAML / serde_yaml relax this. Real-world
    // frontmatter (esp. human-edited URLs that picked up an
    // accidental newline) routinely violates the strict rule.
    // Collapse the offending shape upstream of noyalib so consumers
    // don't need to re-implement this each. Issue #20.
    let collapsed = collapse_multiline_quoted_scalars(yaml_str);

    match noyalib::from_str::<noyalib::Value>(&collapsed) {
        Ok(v) => {
            let metadata: HashMap<String, String> = flatten_yaml(&v);
            Some(Ok(Metadata::new(metadata)))
        }
        Err(e) => Some(Err(MetadataError::ExtractionError {
            message: format!("YAML parse error in frontmatter: {e}"),
        })),
    }
}

/// Collapses multi-line double-quoted YAML scalars onto a single line.
///
/// noyalib correctly enforces YAML 1.2.2 §7.3.2 (continuation must be
/// indented more than the parent block). Human-edited frontmatter
/// often violates this — e.g. a `url: "\n<value>"` shape where a
/// literal newline crept in after the opening quote. PyYAML and
/// serde_yaml fold those onto one line; this helper does the same so
/// noyalib never sees the offending shape.
///
/// The scan is line-based and deliberately simple: when a line ends
/// with `: "` (key + opening quote with nothing after), walk forward
/// joining subsequent lines until the closing `"` is found. Comments
/// and other quote styles are not transformed.
pub(super) fn collapse_multiline_quoted_scalars(block: &str) -> String {
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

/// Joins a nested key onto its dotted prefix (`author` + `name` gives
/// `author.name`); an empty prefix yields the key unchanged.
fn join_key(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{}.{}", prefix, key)
    }
}

/// Renders a sequence as `[a, b, c]`, the shape every front-matter format
/// shares for lists.
fn inline_list<I: Iterator<Item = String>>(items: I) -> String {
    format!("[{}]", items.collect::<Vec<String>>().join(", "))
}

/// Flattens a nested YAML value into a flat key-value map.
///
/// Nested keys are joined with `.` (e.g., `author.name`).
/// Sequences are rendered as comma-separated lists wrapped in brackets.
pub(super) fn flatten_yaml(
    value: &noyalib::Value,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    flatten_yaml_recursive(value, String::new(), &mut map);
    map
}

/// Recursively walks a YAML value tree, inserting leaf values into the map
/// with dot-separated keys for nested mappings.
pub(super) fn flatten_yaml_recursive(
    value: &noyalib::Value,
    prefix: String,
    map: &mut HashMap<String, String>,
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

/// Extracts TOML metadata from the content.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to extract TOML metadata from.
///
/// # Returns
///
/// An `Option<Metadata>` containing the extracted metadata if successful, or `None` if extraction fails.
pub(super) fn extract_toml_metadata(content: &str) -> Option<Metadata> {
    let captures = TOML_FRONT_MATTER.captures(content)?;
    let toml_str = captures.get(1)?.as_str().trim();

    let toml_value: TomlValue = toml::from_str(toml_str).ok()?;

    let mut metadata = HashMap::new();
    flatten_toml(&toml_value, &mut metadata, String::new());

    Some(Metadata::new(metadata))
}

/// Recursively flattens a TOML value tree into a flat key-value map.
///
/// Nested keys are joined with `.` (e.g., `author.name`).
/// Arrays are rendered as comma-separated lists wrapped in brackets.
pub(super) fn flatten_toml(
    value: &TomlValue,
    map: &mut HashMap<String, String>,
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

/// Extracts JSON front-matter from the start of `content`.
///
/// Returns `Some(Ok(_))` when a balanced JSON object is found and parsed,
/// `Some(Err(_))` when an opening `{` appears but the JSON is malformed
/// (so the caller can surface a useful error instead of the misleading
/// "no front-matter" fallback), and `None` only when no opening `{`
/// appears at the start of the content.
///
/// Nested objects and arrays of objects are preserved by flattening them
/// with dot-separated keys (e.g. `author.name`, matching the YAML/TOML
/// shape). Issue #26.
pub(super) fn extract_json_metadata(
    content: &str,
) -> Option<Result<Metadata, MetadataError>> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with('{') {
        return None;
    }

    // `Deserializer::into_iter` consumes one balanced JSON value at a
    // time. We take the first one — that's the front-matter — and let
    // anything after it be the document body. This replaces the old
    // non-greedy regex that silently truncated nested objects at the
    // first `}` it saw.
    // Deserialising straight into a map makes "the root must be an
    // object" part of the parse: the text starts with `{`, so the only
    // way to get anything else is a syntax error, which is reported as
    // one instead of through a branch no input can reach.
    let mut stream = serde_json::Deserializer::from_str(trimmed)
        .into_iter::<serde_json::Map<String, JsonValue>>();
    let first = stream.next()?; // None only if input is empty after `{`.

    let object = match first {
        Ok(v) => v,
        Err(e) => {
            return Some(Err(MetadataError::ExtractionError {
                message: format!(
                    "JSON parse error in frontmatter: {e}"
                ),
            }))
        }
    };

    let mut metadata: HashMap<String, String> = HashMap::new();
    for (k, v) in object {
        flatten_json(&v, &mut metadata, k);
    }
    Some(Ok(Metadata::new(metadata)))
}

/// Recursively flattens a JSON value tree into a flat key-value map.
///
/// Mirrors `flatten_toml` / `flatten_yaml`: nested objects use
/// dot-separated keys; arrays render as comma-separated lists wrapped in
/// brackets. Strings are stored as-is; numbers, booleans, and `null`
/// fall back to their JSON `Display` form.
pub(super) fn flatten_json(
    value: &JsonValue,
    map: &mut HashMap<String, String>,
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
            // render the same — callers that need element-level access
            // should use the v0.0.6 typed-extraction API (issue #45).
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
