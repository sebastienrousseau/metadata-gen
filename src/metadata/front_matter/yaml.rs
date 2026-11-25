// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! YAML front matter on noyalib.

use super::*;
use alloc::vec::Vec;
use noyalib::ParserConfig;

fn config(limits: &ParseLimits) -> ParserConfig {
    // `check_size` has already held the block to `max_block_bytes`.
    // Collapsing ends every line with a newline, the last one included,
    // so the text noyalib sees can be one byte longer than the block;
    // without the extra byte a block exactly at the limit was rejected.
    ParserConfig::strict()
        .max_depth(limits.max_depth)
        .max_document_length(limits.max_block_bytes.saturating_add(1))
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
    let value = parse_value(&collapsed, &config(limits))
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
    noyalib::from_str_borrowing_with_config::<T>(raw, &config(limits))
        .map_err(|e| borrowed_error(raw, limits, e))
}

/// noyalib's borrowing deserialiser cannot lend from a tag, an alias
/// of some shapes, or a merge key, and says so with an internal
/// fallback signal; name the owned entry point instead.
///
/// The signal also covers some malformed input; the value path tells the
/// two apart and carries the real parse error.
fn borrowed_error(
    raw: &str,
    limits: &ParseLimits,
    e: noyalib::Error,
) -> MetadataError {
    if is_fallback(&e) {
        if let Err(parse) = value(raw, limits) {
            return parse;
        }
        return MetadataError::ExtractionError {
            message: "This YAML front matter uses a tag, alias or merge key that cannot be borrowed; use extract_typed.".to_string(),
        };
    }
    parse_error(raw, e)
}

/// Walks the block with the streaming deserialiser. noyalib answers
/// some shapes (aliases, merge keys, tags, an empty document) with a
/// fallback signal instead; [`is_fallback`] recognises it.
pub(in crate::metadata) fn stream<'a, S>(
    raw: &'a str,
    limits: &ParseLimits,
    seed: S,
) -> Result<(), noyalib::Error>
where
    S: serde::de::DeserializeSeed<'a, Value = ()>,
{
    let config = config(limits);
    let mut de =
        noyalib::StreamingDeserializer::with_config(raw, &config);
    seed.deserialize(&mut de).and_then(|()| de.end())
}

/// True for noyalib's "use the value path" signal.
pub(in crate::metadata) fn is_fallback(e: &noyalib::Error) -> bool {
    matches!(e, noyalib::Error::Custom(m) if m == "$__noyalib_streaming_fallback")
}

/// Parses the block into a value tree, for the fallback path.
pub(in crate::metadata) fn value(
    raw: &str,
    limits: &ParseLimits,
) -> Result<noyalib::Value, MetadataError> {
    parse_value(raw, &config(limits)).map_err(|e| parse_error(raw, e))
}

/// `text` as one value tree, as `from_str_with_config::<Value>` reads it.
#[cfg(feature = "std")]
fn parse_value(
    text: &str,
    config: &ParserConfig,
) -> Result<noyalib::Value, noyalib::Error> {
    noyalib::from_str_with_config(text, config)
}

/// `text` as one value tree, through noyalib's non-generic document
/// loader (#61, code size): `from_str_with_config::<Value>` compiles
/// serde's streaming and tree deserialisers for `Value` into this
/// crate, though a `Value` target never runs them.
///
/// Without `std`, both entry points run the same checks in the same
/// order (noyalib 0.0.64): the `max_document_length` check, the
/// span-free loader over every document, then the post-parse pass,
/// which is a no-op here because [`config`] sets no include resolver,
/// properties or policies. What is left is `from_str`'s one-document
/// rule: more than one is `MoreThanOneDocument`, none is `Null`.
/// `value_path_matches_from_str` holds the two to the same answers.
#[cfg(not(feature = "std"))]
fn parse_value(
    text: &str,
    config: &ParserConfig,
) -> Result<noyalib::Value, noyalib::Error> {
    let mut docs =
        noyalib::document::load_all_with_config(text, config)?;
    if docs.len() > 1 {
        return Err(noyalib::Error::MoreThanOneDocument);
    }
    docs.next().unwrap_or(Ok(noyalib::Value::Null))
}

/// The crate error for a noyalib error on `raw`.
pub(in crate::metadata) fn error(
    raw: &str,
    e: noyalib::Error,
) -> MetadataError {
    parse_error(raw, e)
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
    let eq_pos = crate::utils::find_str(line, ": \"")?;
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
    // An empty block parses as null: no keys, as for TOML and JSON.
    if !value.is_null() {
        flatten_yaml_recursive(value, String::new(), &mut map);
    }
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

#[cfg(test)]
mod parse_value_tests {
    use super::{config, parse_value, ParseLimits};
    use alloc::string::ToString;

    /// The value path answers exactly as noyalib's own
    /// `from_str_with_config::<Value>`, in the `std` build and the
    /// `no_std` one: same tree, or the same error message.
    #[test]
    fn value_path_matches_from_str() {
        let small = ParseLimits::default().max_block_bytes(16);
        let cases: [(&str, &ParseLimits); 10] = [
            ("a: 1\nb: [x, y]\n", &ParseLimits::default()),
            ("", &ParseLimits::default()),
            ("# only a comment\n", &ParseLimits::default()),
            ("a: 1\n...\nb: 2\n", &ParseLimits::default()),
            ("a: 1\n---\nb: 2\n", &ParseLimits::default()),
            ("a: [1, 2\n", &ParseLimits::default()),
            ("a: 1\na: 2\n", &ParseLimits::default()),
            (
                "base: &b {x: 1}\nc:\n  <<: *b\n",
                &ParseLimits::default(),
            ),
            ("a: !!str 1\nb: yes\n", &ParseLimits::default()),
            ("title: longer than sixteen bytes\n", &small),
        ];
        for (text, limits) in cases {
            let cfg = config(limits);
            let ours =
                parse_value(text, &cfg).map_err(|e| e.to_string());
            let theirs =
                noyalib::from_str_with_config::<noyalib::Value>(
                    text, &cfg,
                )
                .map_err(|e| e.to_string());
            assert_eq!(ours, theirs, "input: {text:?}");
        }
    }
}
