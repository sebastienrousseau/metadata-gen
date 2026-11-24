// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Boundaries the rest of the suite left open, found by mutation testing
//! (#54): each test fails if the condition or arithmetic it names is
//! changed, where the suite previously passed either way.

#![cfg(all(
    feature = "std",
    feature = "yaml",
    feature = "json",
    feature = "html"
))]

use metadata_gen::metatags::{
    extract_meta_tags, extract_meta_tags_lenient,
};
use metadata_gen::{
    extract_metadata, extract_metadata_with_limits, generate_metatags,
    process_metadata, Metadata, MetadataError, ParseLimits,
};
use std::collections::HashMap;

fn parse_span(doc: &str) -> std::ops::Range<usize> {
    match extract_metadata(doc) {
        Err(MetadataError::Parse {
            span: Some(span), ..
        }) => span,
        other => {
            panic!("expected a located parse error, got {other:?}")
        }
    }
}

fn processed_date(date: &str) -> Result<String, MetadataError> {
    let mut map = HashMap::new();
    map.insert("title".to_string(), "T".to_string());
    map.insert("date".to_string(), date.to_string());
    process_metadata(&Metadata::new(map))
        .map(|m| m.get("date").cloned().unwrap_or_default())
}

#[test]
fn a_block_exactly_at_the_size_limit_is_accepted() {
    let doc = "---\ntitle: limit\n---\n";
    let raw_len = "title: limit".len();
    let at_limit = ParseLimits::default().max_block_bytes(raw_len);
    assert!(extract_metadata_with_limits(doc, &at_limit).is_ok());
    let below = ParseLimits::default().max_block_bytes(raw_len - 1);
    assert!(extract_metadata_with_limits(doc, &below).is_err());
}

#[test]
fn parse_errors_point_at_the_offending_byte() {
    // JSON: line 3, column 8 of the block is the `o` of `oops`.
    let json = "{\n  \"a\": 1,\n  \"b\": oops\n}\n";
    assert_eq!(parse_span(json), 19..20);
    // YAML: line 2, column 6 of `title: ok\n  bad: indent`.
    let yaml = "---\ntitle: ok\n  bad: indent\n---\n";
    assert_eq!(parse_span(yaml), 15..16);
}

#[test]
fn sequences_flatten_to_an_inline_list() {
    let doc = "---\ntags: [\"a b\", 'c', 3, true]\nnested: [[1, 2], {k: v}]\n---\n";
    let meta = extract_metadata(doc).unwrap();
    assert_eq!(meta.get("tags").unwrap(), "[a b, c, 3, true]");
    assert_eq!(meta.get("nested").unwrap(), "[[1, 2], {k: v}]");
}

#[test]
fn whitespace_after_the_opening_fence_is_not_part_of_the_block() {
    let meta = extract_metadata("---   \ntitle: a\n---\nBody").unwrap();
    assert_eq!(meta.get("title").unwrap(), "a");
    assert_eq!(meta.into_inner().len(), 1);
}

#[test]
fn apple_and_microsoft_tags_have_their_own_groups() {
    let map: HashMap<String, String> = [
        ("apple-mobile-web-app-title", "App"),
        ("msapplication-TileColor", "#fff"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let groups = generate_metatags(&map);
    assert_eq!(
        groups.apple,
        r#"<meta name="apple-mobile-web-app-title" content="App">"#
    );
    assert_eq!(
        groups.ms,
        r##"<meta name="msapplication-TileColor" content="#fff">"##
    );
}

#[test]
fn an_eight_character_basic_date_is_long_enough() {
    assert_eq!(processed_date("20240102").unwrap(), "2024-01-02");
    assert!(processed_date("2024012").is_err());
}

#[test]
fn a_slash_date_needs_two_two_and_four_digits() {
    let err = processed_date("1/123/2024").unwrap_err();
    assert!(
        err.to_string().contains("Invalid DD/MM/YYYY date format."),
        "{err}"
    );
}

#[test]
fn single_quotes_hide_a_closing_angle_bracket() {
    let (tags, stopped) = extract_meta_tags_lenient(
        "<meta name='a' content='x>y'><meta name=\"b\" <x>",
    );
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].content, "x>y");
    let stopped = stopped.expect("the stray '<' stops the scan");
    assert!(stopped.to_string().contains("at byte 44:"), "{stopped}");
}

#[test]
fn an_element_whose_name_only_starts_with_meta_is_not_meta() {
    let tags = extract_meta_tags(
        "<metadata name=\"a\" content=\"b\"><meta name=\"c\" content=\"d\">",
    )
    .unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "c");
}

#[test]
fn a_breadcrumb_trail_is_a_valid_json_array() {
    let json = metadata_gen::schema_org::breadcrumb_list(&[
        ("Home", "https://example.com/"),
        ("Blog", "https://example.com/blog/"),
    ]);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let items = value["itemListElement"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[1]["position"], 2);
    assert_eq!(items[1]["name"], "Blog");
}

#[test]
fn borrowed_metadata_exposes_every_entry() {
    let doc = "---\ntitle: Hello\nauthor:\n  name: Ada\n---\nbody";
    let meta = metadata_gen::extract_metadata_cow(doc).unwrap();
    assert!(!meta.is_empty());
    assert_eq!(meta.len(), 2);
    let pairs: Vec<(&str, &str)> = meta.iter().collect();
    assert_eq!(pairs, [("author.name", "Ada"), ("title", "Hello")]);
    let map = meta.into_inner();
    let owned: Vec<(String, String)> = map
        .into_iter()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    assert_eq!(
        owned,
        [
            ("author.name".to_string(), "Ada".to_string()),
            ("title".to_string(), "Hello".to_string())
        ]
    );
}
