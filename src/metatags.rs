//! Meta tag generation and extraction module.
//!
//! This module provides functionality for generating HTML meta tags from metadata
//! and extracting meta tags from HTML content.

use crate::error::MetadataError;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::{collections::HashMap, fmt};

/// Holds collections of meta tags for different platforms and categories.
///
/// # Example
///
/// ```
/// use metadata_gen::metatags::generate_metatags;
/// use std::collections::HashMap;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("description".to_string(), "A sample page".to_string());
/// metadata.insert("og:title".to_string(), "Sample".to_string());
///
/// let tags = generate_metatags(&metadata);
/// assert!(tags.primary.contains("description"));
/// assert!(tags.og.contains("og:title"));
/// ```
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct MetaTagGroups {
    /// The `apple` meta tags.
    pub apple: String,
    /// The primary meta tags.
    pub primary: String,
    /// The `og` meta tags.
    pub og: String,
    /// The `ms` meta tags.
    pub ms: String,
    /// The `twitter` meta tags.
    pub twitter: String,
}

/// Represents a single meta tag.
///
/// # Example
///
/// ```
/// use metadata_gen::metatags::MetaTag;
///
/// let tag = MetaTag {
///     name: "description".to_string(),
///     content: "A sample page".to_string(),
/// };
/// assert_eq!(tag.name, "description");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaTag {
    /// The name or property of the meta tag.
    pub name: String,
    /// The content of the meta tag.
    pub content: String,
}

impl MetaTagGroups {
    /// Adds a custom meta tag to the appropriate group.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the meta tag.
    /// * `content` - The content of the meta tag.
    pub fn add_custom_tag(&mut self, name: &str, content: &str) {
        let formatted_tag = self.format_meta_tag(name, content);

        // Match based on specific prefixes for Apple, MS, OG, Twitter, etc.
        if name.starts_with("apple-")
            || name == "mobile-web-app-capable"
        {
            self.apple.push_str(&formatted_tag);
        } else if name.starts_with("msapplication-") {
            // println!("Adding MS meta tag: {}", formatted_tag);  // Debugging output
            self.ms.push_str(&formatted_tag);
        } else if name.starts_with("og:") {
            // println!("Adding OG meta tag: {}", formatted_tag);  // Debugging output
            self.og.push_str(&formatted_tag);
        } else if name.starts_with("twitter:") {
            // println!("Adding Twitter meta tag: {}", formatted_tag);  // Debugging output
            self.twitter.push_str(&formatted_tag);
        } else {
            // println!("Adding Primary meta tag: {}", formatted_tag);  // Debugging output
            self.primary.push_str(&formatted_tag);
        }
    }

    /// Formats a single meta tag.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the meta tag.
    /// * `content` - The content of the meta tag.
    ///
    /// # Returns
    ///
    /// A formatted meta tag string.
    pub fn format_meta_tag(&self, name: &str, content: &str) -> String {
        format!(
            r#"<meta name="{}" content="{}">"#,
            name,
            content.replace('"', "&quot;")
        )
    }

    /// Generates meta tags for Apple devices.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_apple_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const APPLE_TAGS: [&str; 4] = [
            "apple-mobile-web-app-capable",
            "mobile-web-app-capable",
            "apple-mobile-web-app-status-bar-style",
            "apple-mobile-web-app-title",
        ];
        self.apple = self.generate_tags(metadata, &APPLE_TAGS);
    }

    /// Generates primary meta tags like `author`, `description`, and `keywords`.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_primary_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const PRIMARY_TAGS: [&str; 4] =
            ["author", "description", "keywords", "viewport"];
        self.primary = self.generate_tags(metadata, &PRIMARY_TAGS);
    }

    /// Generates Open Graph (`og`) meta tags for social media.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_og_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const OG_TAGS: [&str; 5] = [
            "og:title",
            "og:description",
            "og:image",
            "og:url",
            "og:type",
        ];
        self.og = self.generate_tags(metadata, &OG_TAGS);
    }

    /// Generates Microsoft-specific meta tags.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_ms_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const MS_TAGS: [&str; 2] =
            ["msapplication-TileColor", "msapplication-TileImage"];
        self.ms = self.generate_tags(metadata, &MS_TAGS);
    }

    /// Generates Twitter meta tags for embedding rich media in tweets.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_twitter_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const TWITTER_TAGS: [&str; 5] = [
            "twitter:card",
            "twitter:site",
            "twitter:title",
            "twitter:description",
            "twitter:image",
        ];
        self.twitter = self.generate_tags(metadata, &TWITTER_TAGS);
    }

    /// Generates meta tags based on the provided list of tag names.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a `HashMap` containing the metadata.
    /// * `tags` - A reference to an array of tag names.
    ///
    /// # Returns
    ///
    /// A string containing the generated meta tags.
    pub fn generate_tags(
        &self,
        metadata: &HashMap<String, String>,
        tags: &[&str],
    ) -> String {
        tags.iter()
            .filter_map(|&tag| {
                metadata
                    .get(tag)
                    .map(|value| self.format_meta_tag(tag, value))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Implement `Display` for `MetaTagGroups`.
impl fmt::Display for MetaTagGroups {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}",
            self.apple, self.primary, self.og, self.ms, self.twitter
        )
    }
}

/// Generates HTML meta tags based on the provided metadata.
///
/// This function takes metadata from a `HashMap` and generates meta tags for various platforms (e.g., Apple, Open Graph, Twitter).
///
/// # Arguments
///
/// * `metadata` - A reference to a `HashMap` containing the metadata.
///
/// # Returns
///
/// A `MetaTagGroups` structure with meta tags grouped by platform.
pub fn generate_metatags(
    metadata: &HashMap<String, String>,
) -> MetaTagGroups {
    let mut meta_tag_groups = MetaTagGroups::default();
    meta_tag_groups.generate_apple_meta_tags(metadata);
    meta_tag_groups.generate_primary_meta_tags(metadata);
    meta_tag_groups.generate_og_meta_tags(metadata);
    meta_tag_groups.generate_ms_meta_tags(metadata);
    meta_tag_groups.generate_twitter_meta_tags(metadata);
    meta_tag_groups
}

/// Extracts every `<meta>` tag from an HTML document.
///
/// Walks the input in document order, yielding one `MetaTag` per
/// `<meta>` element that carries both an identifying attribute (`name`,
/// `property`, or `http-equiv`, in that fallback order) and a `content`
/// attribute. Self-closing (`<meta … />`) and HTML-style (`<meta …>`)
/// shapes are both accepted.
///
/// # Arguments
///
/// * `html_content` - A string slice containing the HTML content to parse.
///
/// # Returns
///
/// Returns a `Result` containing a `Vec<MetaTag>` in document order if
/// parsing reached the end of the input, or a `MetadataError` if the
/// underlying scanner could not recover from a malformed region.
///
/// # Errors
///
/// Returns `MetadataError::ExtractionError` only when the input is so
/// malformed that no further events can be produced. Per-element issues
/// (missing `content`, unknown attributes, unrecognized escape) are
/// tolerated silently.
///
/// # Implementation note
///
/// Backed by `quick-xml` configured in HTML-tolerant mode (mismatched
/// end tags allowed, no DTD validation). This replaces the previous
/// `scraper` / `html5ever` dependency tree, which dragged ~30 transitive
/// crates including `fxhash` (RUSTSEC-2025-0057) and a vulnerable
/// `phf_generator` / `rand 0.8` path (RUSTSEC-2026-0097). See issue #22.
pub fn extract_meta_tags(
    html_content: &str,
) -> Result<Vec<MetaTag>, MetadataError> {
    let mut reader = Reader::from_str(html_content);
    let config = reader.config_mut();
    // HTML is not XML — be lenient so doctypes, unquoted attrs, and
    // mismatched end tags don't abort the scan.
    config.check_end_names = false;
    config.trim_text(false);

    let mut meta_tags = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            // Both Start (`<meta …>`) and Empty (`<meta … />`) shapes are
            // produced for `<meta>` depending on author style. Treat them
            // identically.
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if name_eq_ignore_case(e.name().as_ref(), b"meta") =>
            {
                if let Some(tag) = collect_meta_tag(e) {
                    meta_tags.push(tag);
                }
            }
            Ok(_) => {}
            Err(e) => {
                // Per #22 acceptance: tolerate malformed regions and
                // return what we found so far. The remaining content may
                // simply be the body of an HTML page that quick-xml
                // doesn't fully understand.
                let _ = e;
                break;
            }
        }
        buf.clear();
    }

    Ok(meta_tags)
}

/// Case-insensitive ASCII equality for element names.
fn name_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(x, y)| x.eq_ignore_ascii_case(y))
}

/// Pulls a `MetaTag` out of a `<meta>` start/empty element if it carries
/// both an identifying attribute (`name` → fallback `property` →
/// fallback `http-equiv`) and a `content` value.
///
/// HTML entities in attribute values are decoded via `quick-xml`'s
/// `unescape_value` so `&amp;`, `&quot;`, numeric refs, etc. round-trip
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
        // Decode as UTF-8 then unescape HTML entities. `unescape_value`
        // was deprecated in quick-xml 0.40; the recommended replacement
        // is to drive the static `escape::unescape` helper directly.
        let Ok(raw) = std::str::from_utf8(attr.value.as_ref()) else {
            continue;
        };
        let value = match quick_xml::escape::unescape(raw) {
            Ok(v) => v.into_owned(),
            Err(_) => continue,
        };
        match attr.key.as_ref() {
            k if name_eq_ignore_case(k, b"name") => name = Some(value),
            k if name_eq_ignore_case(k, b"property") => {
                property = Some(value)
            }
            k if name_eq_ignore_case(k, b"http-equiv") => {
                http_equiv = Some(value)
            }
            k if name_eq_ignore_case(k, b"content") => {
                content = Some(value)
            }
            _ => {}
        }
    }

    let id = name.or(property).or(http_equiv)?;
    let content = content?;
    Some(MetaTag { name: id, content })
}

/// Converts a vector of MetaTags into a HashMap for easier access.
///
/// # Arguments
///
/// * `meta_tags` - A vector of MetaTag structs.
///
/// # Returns
///
/// A HashMap where the keys are the meta tag names and the values are the contents.
pub fn meta_tags_to_hashmap(
    meta_tags: Vec<MetaTag>,
) -> HashMap<String, String> {
    meta_tags
        .into_iter()
        .map(|tag| (tag.name, tag.content))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_metatags() {
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Page".to_string());
        metadata.insert(
            "description".to_string(),
            "A test page".to_string(),
        );
        metadata
            .insert("og:title".to_string(), "OG Test Page".to_string());

        let meta_tags = generate_metatags(&metadata);

        assert!(meta_tags.primary.contains("description"));
        assert!(meta_tags.og.contains("og:title"));
    }

    #[test]
    fn test_extract_meta_tags() {
        let html = r#"
        <html>
          <head>
            <meta name="description" content="A sample page">
            <meta property="og:title" content="Sample Title">
            <meta http-equiv="content-type" content="text/html; charset=UTF-8">
          </head>
          <body>
            <p>Some content</p>
          </body>
        </html>
        "#;

        let meta_tags = extract_meta_tags(html).unwrap();
        assert_eq!(meta_tags.len(), 3);
        assert!(meta_tags.iter().any(|tag| tag.name == "description"
            && tag.content == "A sample page"));
        assert!(meta_tags.iter().any(|tag| tag.name == "og:title"
            && tag.content == "Sample Title"));
        assert!(meta_tags.iter().any(|tag| tag.name == "content-type"
            && tag.content == "text/html; charset=UTF-8"));
    }

    #[test]
    fn test_extract_meta_tags_preserves_document_order() {
        // Issue #22 acceptance: document order must match the previous
        // scraper-backed implementation. Three tags, deterministic order.
        let html = r#"
        <html><head>
          <meta name="a" content="1">
          <meta property="og:b" content="2">
          <meta name="c" content="3">
        </head><body></body></html>
        "#;
        let tags = extract_meta_tags(html).unwrap();
        let names: Vec<_> =
            tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["a", "og:b", "c"]);
    }

    #[test]
    fn test_extract_meta_tags_handles_self_closing() {
        // XHTML-style self-closing syntax must yield the same result as
        // HTML-style. Both shapes appear in the wild.
        let html = r#"<meta name="x" content="1" /><meta name="y" content="2">"#;
        let tags = extract_meta_tags(html).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "x");
        assert_eq!(tags[1].name, "y");
    }

    #[test]
    fn test_extract_meta_tags_decodes_entities() {
        // Issue #22 acceptance: HTML entities in attribute values must
        // be decoded so consumers don't see literal `&amp;` text.
        let html =
            r#"<meta name="title" content="Tom &amp; Jerry &lt;3">"#;
        let tags = extract_meta_tags(html).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].content, "Tom & Jerry <3");
    }

    #[test]
    fn test_extract_meta_tags_does_not_panic_on_malformed() {
        // Issue #22 acceptance: a malformed HTML fragment with an
        // unclosed tag must not panic; whatever was already parsed is
        // returned to the caller. We don't pin the exact count because
        // recovery behaviour is intentionally implementation-defined.
        let html = r#"
        <html><head>
          <meta name="first" content="ok">
          <meta name="broken" content="oops
          <meta name="second" content="probably-lost">
        </head>
        "#;
        let _ = extract_meta_tags(html).expect("must not panic");
    }

    #[test]
    fn test_extract_meta_tags_ignores_meta_without_content() {
        // A <meta> with no content attr is dropped (parity with the
        // previous scraper-based behaviour).
        let html =
            r#"<meta name="orphan"><meta name="ok" content="yes">"#;
        let tags = extract_meta_tags(html).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "ok");
        assert_eq!(tags[0].content, "yes");
    }

    #[test]
    fn test_extract_meta_tags_empty_html() {
        let html = "<html><head></head><body></body></html>";
        let meta_tags = extract_meta_tags(html).unwrap();
        assert_eq!(meta_tags.len(), 0);
    }

    #[test]
    fn test_meta_tags_to_hashmap() {
        let meta_tags = vec![
            MetaTag {
                name: "description".to_string(),
                content: "A sample page".to_string(),
            },
            MetaTag {
                name: "og:title".to_string(),
                content: "Sample Title".to_string(),
            },
        ];

        let hashmap = meta_tags_to_hashmap(meta_tags);
        assert_eq!(hashmap.len(), 2);
        assert_eq!(
            hashmap.get("description"),
            Some(&"A sample page".to_string())
        );
        assert_eq!(
            hashmap.get("og:title"),
            Some(&"Sample Title".to_string())
        );
    }

    #[test]
    fn test_meta_tag_groups_display() {
        let groups = MetaTagGroups {
    apple: "<meta name=\"apple-mobile-web-app-capable\" content=\"yes\">".to_string(),
    primary: "<meta name=\"description\" content=\"A test page\">".to_string(),
    og: "<meta property=\"og:title\" content=\"Test Page\">".to_string(),
    ms: "<meta name=\"msapplication-TileColor\" content=\"#ffffff\">".to_string(),
    twitter: "<meta name=\"twitter:card\" content=\"summary\">".to_string(),
};

        let display = groups.to_string();
        assert!(display.contains("apple-mobile-web-app-capable"));
        assert!(display.contains("description"));
        assert!(display.contains("og:title"));
        assert!(display.contains("msapplication-TileColor"));
        assert!(display.contains("twitter:card"));
    }

    #[test]
    fn test_format_meta_tag() {
        let groups = MetaTagGroups::default();
        let tag = groups.format_meta_tag("test", "Test \"Value\"");
        assert_eq!(
            tag,
            r#"<meta name="test" content="Test &quot;Value&quot;">"#
        );
    }
}
