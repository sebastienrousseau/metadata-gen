// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for `metadata`; a child module so private items stay reachable.

//! Paths the main test module did not reach: the pathological
//! multi-line quote, non-string leaves in each flattener, the
//! DD/MM/YYYY shape checks, and slug derivation.

use super::front_matter::*;
use super::*;
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

#[test]
fn unclosed_multiline_quote_is_emitted_as_is() {
    let block = "title: \"\n  first part\n  second part";
    let out = collapse_multiline_quoted_scalars(block);
    // No closing quote: the joined text is kept so the parser sees the
    // same broken document rather than a silently dropped key.
    assert_eq!(out, "title: \"first part second part\n");
}

#[test]
fn closed_multiline_quote_joins_onto_one_line() {
    let block = "title: \"\n  first\n  second\"\nnext: 1";
    let out = collapse_multiline_quoted_scalars(block);
    assert_eq!(out, "title: \"first second\"\nnext: 1\n");
}

#[test]
fn yaml_flattener_renders_non_string_leaves_and_sequences() {
    let value: noyalib::Value = noyalib::from_str(
        "count: 3\nflag: true\nnums: [1, 2]\nmixed: [a, 2]",
    )
    .expect("valid yaml");
    let map = flatten_yaml(&value);
    assert_eq!(map["count"], "3");
    assert_eq!(map["flag"], "true");
    assert_eq!(map["nums"], "[1, 2]");
    assert_eq!(map["mixed"], "[a, 2]");
}

#[test]
fn toml_flattener_covers_every_leaf_kind() {
    let value: TomlValue = toml::from_str(
        "name = \"x\"\nn = 7\nok = true\nwhen = 2024-01-02\n\
         ints = [1, 2]\nwords = [\"a\", \"b\"]\n[nested]\nk = 1.5",
    )
    .expect("valid toml");
    let mut map = HashMap::new();
    flatten_toml(&value, &mut map, String::new());
    assert_eq!(map["name"], "x");
    assert_eq!(map["n"], "7");
    assert_eq!(map["ok"], "true");
    assert_eq!(map["when"], "2024-01-02");
    assert_eq!(map["ints"], "[1, 2]");
    assert_eq!(map["words"], "[a, b]");
    assert_eq!(map["nested.k"], "1.5");
}

#[test]
fn json_flattener_covers_every_leaf_kind() {
    let value: JsonValue = serde_json::from_str(
        r#"{"s":"x","n":2,"b":false,"z":null,"arr":[1,"a",null],"o":{"k":{"d":1}}}"#,
    )
    .expect("valid json");
    let mut map = HashMap::new();
    flatten_json(&value, &mut map, String::new());
    assert_eq!(map["s"], "x");
    assert_eq!(map["n"], "2");
    assert_eq!(map["b"], "false");
    assert_eq!(map["z"], "null");
    assert_eq!(map["arr"], "[1, a, null]");
    assert_eq!(map["o.k.d"], "1");
}

#[test]
fn json_front_matter_syntax_error_is_reported() {
    let err = extract_metadata("{\"title\": }").unwrap_err();
    assert!(
        matches!(err, MetadataError::ExtractionError { ref message } if message.contains("JSON parse error"))
    );
}

#[test]
fn json_front_matter_with_nested_objects_and_body() {
    let meta = extract_metadata(
        "{\"title\": \"T\", \"a\": {\"b\": [1, 2]}}\n\nBody text",
    )
    .expect("json front matter");
    assert_eq!(meta.get("title").map(String::as_str), Some("T"));
    assert_eq!(meta.get("a.b").map(String::as_str), Some("[1, 2]"));
}

#[test]
fn dd_mm_yyyy_dates_are_reformatted() {
    assert_eq!(standardize_date("01/02/2024").unwrap(), "2024-02-01");
}

#[test]
fn slash_dates_with_wrong_part_widths_are_rejected() {
    // Ten characters with a slash, but not DD/MM/YYYY.
    let err = standardize_date("123/4/5678").unwrap_err();
    assert!(
        matches!(err, MetadataError::DateParseError(ref m) if m.contains("DD/MM/YYYY"))
    );
}

#[test]
fn short_and_empty_dates_are_rejected() {
    assert!(
        matches!(standardize_date("   ").unwrap_err(), MetadataError::DateParseError(ref m) if m.contains("empty"))
    );
    assert!(
        matches!(standardize_date("2024-1").unwrap_err(), MetadataError::DateParseError(ref m) if m.contains("too short"))
    );
    assert!(
        matches!(standardize_date("2024-99-99").unwrap_err(), MetadataError::DateParseError(ref m) if m.contains("Failed to parse"))
    );
}

#[test]
fn process_metadata_standardises_date_and_derives_slug() {
    let mut data = HashMap::new();
    data.insert("title".to_string(), "Hello World!".to_string());
    data.insert("date".to_string(), "01/02/2024".to_string());
    let out =
        process_metadata(&Metadata::new(data)).expect("processed");
    assert_eq!(out.get("date").map(String::as_str), Some("2024-02-01"));
    assert!(out.contains_key("slug"), "slug derived from title");
    assert_eq!(
        out.get("slug").map(String::as_str),
        Some(&*generate_slug("Hello World!"))
    );
}

#[test]
fn process_metadata_keeps_an_explicit_slug() {
    let mut data = HashMap::new();
    data.insert("title".to_string(), "T".to_string());
    data.insert("date".to_string(), "2024-02-01".to_string());
    data.insert("slug".to_string(), "keep-me".to_string());
    let out =
        process_metadata(&Metadata::new(data)).expect("processed");
    assert_eq!(out.get("slug").map(String::as_str), Some("keep-me"));
}

#[test]
fn process_metadata_reports_a_missing_required_field() {
    let mut data = HashMap::new();
    data.insert("title".to_string(), "T".to_string());
    let err = process_metadata(&Metadata::new(data)).unwrap_err();
    assert!(
        matches!(err, MetadataError::MissingFieldError(ref f) if f == "date")
    );
}
