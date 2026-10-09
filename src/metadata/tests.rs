// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for `metadata`; a child module so private items stay reachable.

use super::*;
use crate::MetadataMap;

#[test]
fn test_standardize_date() {
    let test_cases = vec![
        ("2023-05-20T15:30:00Z", "2023-05-20"),
        ("2023-05-20", "2023-05-20"),
        ("20/05/2023", "2023-05-20"), // European format DD/MM/YYYY
    ];

    for (input, expected) in test_cases {
        let result = standardize_date(input, DateOrder::DayFirst);
        assert!(result.is_ok(), "Failed for input: {}", input);
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn test_standardize_date_errors() {
    assert!(standardize_date("", DateOrder::DayFirst).is_err());
    assert!(standardize_date("invalid", DateOrder::DayFirst).is_err());
    assert!(standardize_date("20/05/23", DateOrder::DayFirst).is_err());
    // Invalid DD/MM/YY format
}

#[test]
fn test_generate_slug() {
    assert_eq!(generate_slug("Hello World"), "hello-world");
    assert_eq!(generate_slug("Test 123"), "test-123");
    assert_eq!(generate_slug("  Spaces  "), "spaces");
}

#[test]
fn test_process_metadata() {
    let mut metadata = Metadata::new(MetadataMap::new());
    metadata.insert("title".to_string(), "Test Title".to_string());
    metadata
        .insert("date".to_string(), "2023-05-20T15:30:00Z".to_string());

    let processed = process_metadata(&metadata).unwrap();
    assert_eq!(processed.get("title").unwrap(), "Test Title");
    assert_eq!(processed.get("date").unwrap(), "2023-05-20");
    assert_eq!(processed.get("slug").unwrap(), "test-title");
}

#[test]
fn test_extract_metadata() {
    let yaml_content = r#"---
title: YAML Test
date: 2023-05-20
---
Content here"#;

    let toml_content = r#"+++
title = "TOML Test"
date = "2023-05-20"
+++
Content here"#;

    let json_content = r#"{
"title": "JSON Test",
"date": "2023-05-20"
}
Content here"#;

    let yaml_metadata = extract_metadata(yaml_content).unwrap();
    assert_eq!(yaml_metadata.get("title").unwrap(), "YAML Test");

    let toml_metadata = extract_metadata(toml_content).unwrap();
    assert_eq!(toml_metadata.get("title").unwrap(), "TOML Test");

    let json_metadata = extract_metadata(json_content).unwrap();
    assert_eq!(json_metadata.get("title").unwrap(), "JSON Test");
}

#[test]
fn test_extract_metadata_failure() {
    let invalid_content = "This content has no metadata";
    assert!(extract_metadata(invalid_content).is_err());
}

#[test]
fn test_ensure_required_fields() {
    let mut metadata = Metadata::new(MetadataMap::new());
    metadata.insert("title".to_string(), "Test".to_string());
    metadata.insert("date".to_string(), "2023-05-20".to_string());

    assert!(process_metadata(&metadata).is_ok());

    let mut incomplete_metadata = Metadata::new(MetadataMap::new());
    incomplete_metadata.insert("title".to_string(), "Test".to_string());

    assert!(process_metadata(&incomplete_metadata).is_err());
}

#[test]
fn test_generate_derived_fields() {
    let mut metadata = Metadata::new(MetadataMap::new());
    metadata.insert("title".to_string(), "Test Title".to_string());

    generate_derived_fields(&mut metadata);

    assert_eq!(metadata.get("slug").unwrap(), "test-title");
}

#[test]
fn test_metadata_methods() {
    let mut metadata = Metadata::new(MetadataMap::new());
    metadata.insert("key".to_string(), "value".to_string());

    assert_eq!(metadata.get("key"), Some(&"value".to_string()));
    assert!(metadata.contains_key("key"));
    assert!(!metadata.contains_key("nonexistent"));

    let old_value =
        metadata.insert("key".to_string(), "new_value".to_string());
    assert_eq!(old_value, Some("value".to_string()));
    assert_eq!(metadata.get("key"), Some(&"new_value".to_string()));

    let inner = metadata.into_inner();
    assert_eq!(inner.get("key"), Some(&"new_value".to_string()));
}

#[test]
fn test_process_metadata_with_invalid_date() {
    let mut metadata = Metadata::new(MetadataMap::new());
    metadata.insert("title".to_string(), "Test Title".to_string());
    metadata.insert("date".to_string(), "invalid_date".to_string());

    assert!(process_metadata(&metadata).is_err());
}

#[test]
fn test_extract_yaml_metadata_with_complex_structure() {
    let yaml_content = r#"---
title: Complex YAML Test
date: 2023-05-20
author:
  name: John Doe
  email: john@example.com
tags:
  - rust
  - metadata
  - testing
---
Content here"#;

    let metadata = extract_metadata(yaml_content).unwrap();
    assert_eq!(metadata.get("title").unwrap(), "Complex YAML Test");
    assert_eq!(metadata.get("date").unwrap(), "2023-05-20");
    assert_eq!(metadata.get("author.name").unwrap(), "John Doe");
    assert_eq!(
        metadata.get("author.email").unwrap(),
        "john@example.com"
    );
    assert_eq!(
        metadata.get("tags").unwrap(),
        "[rust, metadata, testing]"
    );
}

#[test]
fn test_extract_toml_metadata_with_complex_structure() {
    let toml_content = r#"+++
title = "Complex TOML Test"
date = 2023-05-20

[author]
name = "John Doe"
email = "john@example.com"

tags = ["rust", "metadata", "testing"]
+++
Content here"#;

    let metadata = extract_metadata(toml_content).unwrap();
    assert_eq!(
        metadata.get("title").expect("Missing 'title' key"),
        "Complex TOML Test"
    );
    assert_eq!(
        metadata.get("date").expect("Missing 'date' key"),
        "2023-05-20"
    );
    assert_eq!(
        metadata
            .get("author.name")
            .expect("Missing 'author.name' key"),
        "John Doe"
    );
    assert_eq!(
        metadata
            .get("author.email")
            .expect("Missing 'author.email' key"),
        "john@example.com"
    );
    assert_eq!(
        metadata
            .get("author.tags")
            .expect("Missing 'author.tags' key"),
        "[rust, metadata, testing]"
    );
}

#[test]
fn test_generate_slug_with_special_characters() {
    assert_eq!(generate_slug("Hello, World! 123"), "hello-world-123");
    assert_eq!(generate_slug("Test: Ästhetik"), "test-ästhetik");
    assert_eq!(
        generate_slug("  Multiple   Spaces  "),
        "multiple-spaces"
    );
}

#[test]
fn test_extract_metadata_collapses_multiline_quoted_scalar() {
    // Regression for sebastienrousseau/metadata-gen#20.
    // A literal newline immediately after `: "` would previously
    // make noyalib reject the frontmatter and the user would see
    // the misleading "No valid front matter found" message.
    // The collapse step joins continuation lines so noyalib sees
    // valid input.
    let content = "---\n\
                   title: Test\n\
                   twitter_url: \"\n\
                   https://example.com/post\"\n\
                   ---\n\
                   body";
    let metadata = extract_metadata(content)
        .expect("multi-line quoted scalar should now parse");
    assert_eq!(metadata.get("title"), Some(&"Test".to_string()));
    assert!(
        metadata
            .get("twitter_url")
            .expect("twitter_url present")
            .contains("https://example.com/post"),
        "twitter_url should retain the URL after collapse"
    );
}

#[test]
fn test_extract_json_metadata_with_nested_object() {
    // Regression for #26: the previous regex-based JSON detector
    // matched the first `}` it saw, so any nested object lost data
    // silently. The serde_json streaming path preserves it.
    let content = r#"{"title": "T", "author": {"name": "Ada", "handle": "ada@example.com"}}
# body"#;
    let meta = extract_metadata(content).expect("nested JSON parses");
    assert_eq!(meta.get("title"), Some(&"T".to_string()));
    assert_eq!(meta.get("author.name"), Some(&"Ada".to_string()));
    assert_eq!(
        meta.get("author.handle"),
        Some(&"ada@example.com".to_string())
    );
}

#[test]
fn test_extract_json_metadata_with_array_of_objects() {
    // Issue #26 acceptance criterion: arrays of objects are preserved.
    let content = r#"{"tags": [{"name":"x"},{"name":"y"}]}
# body"#;
    let meta = extract_metadata(content).expect("array parses");
    // The flattened representation lists each element via its JSON
    // `Display` form; the important property is no data is lost.
    let tags = meta.get("tags").expect("tags key present");
    assert!(tags.contains("\"name\":\"x\""), "got: {tags}");
    assert!(tags.contains("\"name\":\"y\""), "got: {tags}");
}

#[test]
fn test_extract_json_metadata_malformed_surfaces_error() {
    // Issue #26 acceptance criterion: malformed JSON surfaces the
    // underlying serde_json message, not the generic "No valid front
    // matter found", and the parser sees the whole malformed object.
    let content = r#"{"title": "unterminated"#; // intentionally malformed
    let err = extract_metadata(content).expect_err("must error");
    let msg = err.to_string();
    assert!(
        matches!(
            err,
            MetadataError::Parse {
                format: FrontMatterFormat::Json,
                ..
            }
        ),
        "expected a JSON parse error, got: {err:?}"
    );
    assert!(
        msg.contains("json front matter failed to parse")
            && msg.contains("EOF while parsing a string"),
        "expected surfaced JSON error, got: {msg}"
    );
    assert!(
        !msg.contains("No valid front matter found"),
        "should not fall back to the generic message: {msg}"
    );
}

#[test]
fn test_extract_metadata_surfaces_yaml_parse_error() {
    // Regression for sebastienrousseau/metadata-gen#20.
    // A genuinely malformed YAML body (after the collapse step
    // can't help) should surface the noyalib parse error, not
    // the misleading "No valid front matter found" fallback.
    let content = "---\n\
                   title: [unclosed sequence\n\
                   ---\n\
                   body";
    let err = extract_metadata(content)
        .expect_err("malformed YAML should error");
    let msg = format!("{err}");
    assert!(
        msg.contains("yaml front matter failed to parse"),
        "expected surfaced YAML error, got: {msg}"
    );
    assert!(
        !msg.contains("No valid front matter found"),
        "should not fall back to the generic message: {msg}"
    );
}

#[test]
fn empty_front_matter_is_empty_metadata() {
    // An empty block is an empty map in every format, not a single
    // entry under the empty key.
    for doc in [
        "---\n---\nbody",
        "---\n\n---\nbody",
        "+++\n+++\nbody",
        "{}\nbody",
    ] {
        let metadata = extract_metadata(doc).expect(doc);
        assert!(
            metadata.clone().into_inner().is_empty(),
            "{doc:?} gave {metadata:?}"
        );
    }
}
