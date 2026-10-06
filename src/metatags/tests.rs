// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for `metatags`; a child module so private items stay reachable.

use super::*;

#[test]
fn test_generate_metatags() {
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), "Test Page".to_string());
    metadata
        .insert("description".to_string(), "A test page".to_string());
    metadata.insert("og:title".to_string(), "OG Test Page".to_string());

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
    assert!(meta_tags
        .iter()
        .any(|tag| tag.name == "og:title"
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
    let names: Vec<_> = tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["a", "og:b", "c"]);
}

#[test]
fn test_extract_meta_tags_handles_self_closing() {
    // XHTML-style self-closing syntax must yield the same result as
    // HTML-style. Both shapes appear in the wild.
    let html =
        r#"<meta name="x" content="1" /><meta name="y" content="2">"#;
    let tags = extract_meta_tags(html).unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].name, "x");
    assert_eq!(tags[1].name, "y");
}

#[test]
fn test_extract_meta_tags_decodes_entities() {
    // Issue #22 acceptance: HTML entities in attribute values must
    // be decoded so consumers don't see literal `&amp;` text.
    let html = r#"<meta name="title" content="Tom &amp; Jerry &lt;3">"#;
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
    let html = r#"<meta name="orphan"><meta name="ok" content="yes">"#;
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
