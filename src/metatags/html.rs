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
/// Backed by `quick-xml` configured in HTML-tolerant mode (mismatched
/// end tags allowed, no DTD validation). This replaces the previous
/// `scraper` / `html5ever` dependency tree, which dragged ~30 transitive
/// crates including `fxhash` (RUSTSEC-2025-0057) and a vulnerable
/// `phf_generator` / `rand 0.8` path (RUSTSEC-2026-0097). See issue #22.
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
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(html_content);
    let config = reader.config_mut();
    // HTML is not XML: be lenient so doctypes, unquoted attrs, and
    // mismatched end tags don't abort the scan.
    config.check_end_names = false;
    config.trim_text(false);

    let mut meta_tags = Vec::new();
    let mut buf = Vec::new();
    let stopped = loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break None,
            // Both Start (`<meta …>`) and Empty (`<meta … />`) shapes are
            // produced for `<meta>` depending on author style. Treat them
            // identically.
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if e.name().as_ref().eq_ignore_ascii_case("meta") =>
            {
                if let Some(tag) = collect_meta_tag(e) {
                    meta_tags.push(tag);
                }
            }
            Ok(_) => {}
            Err(e) => {
                // Per #22 acceptance: tolerate malformed regions and
                // return what we found so far, now with where it stopped.
                // The remaining content may simply be the body of an HTML
                // page that quick-xml doesn't fully understand.
                let offset = reader.buffer_position();
                break Some(MetadataError::ExtractionError {
                    message: format!(
                        "HTML scan stopped at byte {offset}: {e}"
                    ),
                });
            }
        }
        buf.clear();
    };
    (meta_tags, stopped)
}

/// Pulls a `MetaTag` out of a `<meta>` start/empty element if it carries
/// both an identifying attribute (`name`, then `property`, then
/// `http-equiv`) and a `content` value.
///
/// HTML entities in attribute values are decoded with `quick-xml`'s
/// `escape::unescape`, so `&amp;`, `&quot;`, numeric refs, etc. round-trip
/// to the same byte sequence the previous `scraper` implementation
/// produced.
fn collect_meta_tag(
    e: &quick_xml::events::BytesStart<'_>,
) -> Option<MetaTag> {
    let mut name: Option<String> = None;
    let mut property: Option<String> = None;
    let mut http_equiv: Option<String> = None;
    let mut content: Option<String> = None;

    for attr_res in e.attributes() {
        let Ok(attr) = attr_res else { continue };
        // quick-xml 0.42 hands attribute names and values out as `str`
        // (its reader validates UTF-8 up front), so there is no decode
        // step here any more: unescape HTML entities and match the name.
        // `unescape_value` was deprecated in quick-xml 0.40; driving the
        // static `escape::unescape` helper directly is the replacement.
        let raw: &str = attr.value.as_ref();
        let value = match quick_xml::escape::unescape(raw) {
            Ok(v) => v.into_owned(),
            Err(_) => continue,
        };
        let key: &str = attr.key.as_ref();
        if key.eq_ignore_ascii_case("name") {
            name = Some(value);
        } else if key.eq_ignore_ascii_case("property") {
            property = Some(value);
        } else if key.eq_ignore_ascii_case("http-equiv") {
            http_equiv = Some(value);
        } else if key.eq_ignore_ascii_case("content") {
            content = Some(value);
        }
    }

    let id = name.or(property).or(http_equiv)?;
    let content = content?;
    Some(MetaTag::new(id, content))
}
