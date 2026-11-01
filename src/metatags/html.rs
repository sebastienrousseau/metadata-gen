// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `<meta>` extraction from HTML (feature `html`).

use super::MetaTag;
use crate::error::MetadataError;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Extracts `<meta>` tags from HTML content in one streaming pass
/// (feature `html`).
///
/// Both `<meta …>` and `<meta … />` are handled; attribute names are
/// matched case-insensitively; `name`, `property` and `http-equiv`
/// identify a tag, in that order of preference; entity references in
/// values are decoded. Malformed markup ends the scan and what was found
/// so far is returned, because a whole HTML page rarely satisfies a
/// strict reader end to end. Use [`extract_meta_tags_lenient`] to learn
/// where the scan stopped.
///
/// # Errors
///
/// None today; the signature keeps `Result` so a strict mode can report
/// without a breaking change.
///
/// # Implementation note
///
/// Scans `<…>` tags in one forward pass and parses each `<meta>` tag
/// with `oxml` (attribute parsing and XML/HTML character and entity
/// reference decoding without full-document DOM allocation). This
/// replaces the previous `scraper` / `html5ever` and `quick-xml`
/// dependencies. See issue #22.
///
/// # Example
///
/// ```
/// use metadata_gen::metatags::extract_meta_tags;
///
/// let html = r#"<head><meta name="description" content="A &amp; B"><meta property="og:title" content="T"></head>"#;
/// let tags = extract_meta_tags(html).unwrap();
/// assert_eq!(tags[0].content, "A & B");
/// assert_eq!(tags[1].name, "og:title");
/// ```
pub fn extract_meta_tags(
    html_content: &str,
) -> Result<Vec<MetaTag>, MetadataError> {
    Ok(extract_meta_tags_lenient(html_content).0)
}

/// Like [`extract_meta_tags`], but also reports where a malformed
/// document stopped the scan (feature `html`).
///
/// The second value is `None` when the whole document was read, and
/// otherwise the scanner's error with the byte offset it reached, so a
/// caller can tell "no tags" from "gave up at byte 300".
///
/// # Example
///
/// ```
/// use metadata_gen::metatags::extract_meta_tags_lenient;
///
/// let (tags, stopped) = extract_meta_tags_lenient(r#"<meta name="a" content="1"><p <broken"#);
/// assert_eq!(tags.len(), 1);
/// assert!(stopped.is_some());
/// ```
pub fn extract_meta_tags_lenient(
    html_content: &str,
) -> (Vec<MetaTag>, Option<MetadataError>) {
    let bytes = html_content.as_bytes();
    let mut pos = 0;
    let mut meta_tags = Vec::new();
    let stopped = loop {
        let Some(rel) = memchr::memchr(b'<', &bytes[pos..]) else {
            break None;
        };
        let start = pos + rel;
        match scan_tag_end(bytes, start) {
            Ok(end) => {
                if let Some(attrs) =
                    meta_body(&html_content[start..end])
                {
                    if let Some(tag) = collect_meta_tag(attrs) {
                        meta_tags.push(tag);
                    }
                }
                pos = end;
            }
            Err((offset, reason)) => {
                break Some(MetadataError::ExtractionError {
                    message: format!(
                        "HTML scan stopped at byte {offset}: {reason}"
                    ),
                });
            }
        }
    };
    (meta_tags, stopped)
}

/// Finds the byte index one past the closing `>` of the tag starting at
/// `start`, respecting single- and double-quoted attribute values.
fn scan_tag_end(
    bytes: &[u8],
    start: usize,
) -> Result<usize, (usize, &'static str)> {
    let mut quote: Option<u8> = None;
    for (idx, &b) in bytes[start + 1..].iter().enumerate() {
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
        } else if b == b'"' || b == b'\'' {
            quote = Some(b);
        } else if b == b'<' {
            return Err((start + 1 + idx, "unexpected '<' inside tag"));
        } else if b == b'>' {
            return Ok(start + 1 + idx + 1);
        }
    }
    Err((start, "unclosed tag or attribute"))
}

/// Returns the trimmed attribute slice of a `<meta …>` or `<meta … />`
/// tag, or `None` if `tag_slice` is any other tag.
fn meta_body(tag_slice: &str) -> Option<&str> {
    let inner = tag_slice[1..tag_slice.len() - 1].trim();
    let head = inner.get(..4)?;
    let rest = inner.get(4..)?;
    if !head.eq_ignore_ascii_case("meta")
        || !rest.starts_with(|c: char| c.is_ascii_whitespace())
    {
        return None;
    }
    Some(rest.trim_end_matches('/').trim())
}

/// Pulls a `MetaTag` out of a `<meta>` attribute slice if it carries
/// both an identifying attribute (`name`, then `property`, then
/// `http-equiv`) and a `content` value.
fn collect_meta_tag(attrs_src: &str) -> Option<MetaTag> {
    let xml = format!("<meta {attrs_src}/>");
    let doc = oxml::parse(&xml).ok()?;
    let root = doc.root_element()?;
    let mut slots: [Option<String>; 4] = [None, None, None, None];

    for attr in doc.attributes(root) {
        if let Some(name) = doc.name(attr.name) {
            record_meta_attr(&name.local, attr.value, &mut slots);
        }
    }

    let [name, property, http_equiv, content] = slots;
    let id = name.or(property).or(http_equiv)?;
    Some(MetaTag::new(id, content?))
}

/// Stores a recognized `<meta>` attribute (`name`, `property`,
/// `http-equiv`, or `content`) into the corresponding slot.
fn record_meta_attr(
    key: &str,
    value: &str,
    slots: &mut [Option<String>; 4],
) {
    let idx = if key.eq_ignore_ascii_case("name") {
        0
    } else if key.eq_ignore_ascii_case("property") {
        1
    } else if key.eq_ignore_ascii_case("http-equiv") {
        2
    } else if key.eq_ignore_ascii_case("content") {
        3
    } else {
        return;
    };
    slots[idx] = Some(String::from(value));
}
