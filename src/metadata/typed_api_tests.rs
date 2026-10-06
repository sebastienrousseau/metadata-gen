// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for `metadata`; a child module so private items stay reachable.

use super::*;

#[derive(Debug, serde::Deserialize, PartialEq)]
struct Front {
    title: String,
    count: u32,
    tags: Vec<String>,
}

#[test]
fn typed_extraction_covers_all_three_formats() {
    let yaml = "---\ntitle: T\ncount: 3\ntags: [a, b]\n---\nbody";
    let toml = "+++\ntitle = \"T\"\ncount = 3\ntags = [\"a\", \"b\"]\n+++\nbody";
    let json = "{\"title\": \"T\", \"count\": 3, \"tags\": [\"a\", \"b\"]}\nbody";
    let want = Front {
        title: "T".into(),
        count: 3,
        tags: vec!["a".into(), "b".into()],
    };
    for doc in [yaml, toml, json] {
        assert_eq!(
            extract_typed::<Front>(doc).unwrap(),
            want,
            "{doc:?}"
        );
    }
}

#[test]
fn typed_extraction_reports_the_format_error() {
    assert!(matches!(
        extract_typed::<Front>("---\ntitle: [\n---\n"),
        Err(MetadataError::YamlError(_))
    ));
    assert!(matches!(
        extract_typed::<Front>("+++\ntitle = \n+++\n"),
        Err(MetadataError::TomlError(_))
    ));
    assert!(
        matches!(
            extract_typed::<Front>("{\"title\": \"T\"}"),
            Err(MetadataError::JsonError(_))
        ),
        "missing fields"
    );
    assert!(matches!(
        extract_typed::<Front>("no front matter"),
        Err(MetadataError::ExtractionError { .. })
    ));
}

#[test]
fn body_follows_each_delimiter_shape() {
    let (_, body) =
        extract_metadata_with_body("---\ntitle: T\n---\nBody").unwrap();
    assert_eq!(body, "Body");
    let (_, body) =
        extract_metadata_with_body("+++\ntitle = \"T\"\n+++\r\nBody")
            .unwrap();
    assert_eq!(body, "Body");
    let (_, body) =
        extract_metadata_with_body("  {\"title\": \"T\"}\n\nBody")
            .unwrap();
    assert_eq!(body, "\nBody");
    let (_, body) =
        extract_metadata_with_body("{\"title\": \"T\"}").unwrap();
    assert_eq!(body, "");
}

#[test]
fn detection_reports_the_raw_block_and_offset() {
    let (f, raw, at) =
        detect_front_matter("+++\nx = 1\n+++\nrest").unwrap();
    assert_eq!((f, raw), (FrontMatterFormat::Toml, "x = 1"));
    assert_eq!(at, "+++\nx = 1\n+++".len());
    assert!(detect_front_matter("plain").is_none());
    // A lone `{` is JSON-shaped with a syntax error: the shape is
    // reported so the caller's parse can report the error, exactly
    // as extract_metadata does.
    assert!(matches!(
        detect_front_matter("{"),
        Some((FrontMatterFormat::Json, _, _))
    ));
    assert!(matches!(
        extract_metadata("{"),
        Err(MetadataError::ExtractionError { .. })
    ));
}

#[test]
fn process_options_control_required_fields_and_slug() {
    let mut m = HashMap::new();
    m.insert("title".to_string(), "Hello World".to_string());
    let meta = Metadata::new(m);
    let err = process_metadata(&meta).unwrap_err();
    assert!(
        matches!(err, MetadataError::MissingFieldError(ref f) if f == "date")
    );

    let opts = ProcessOptions::default()
        .required_fields(["title"])
        .derive_slug(false);
    let out = process_metadata_with(&meta, &opts).unwrap();
    assert!(!out.contains_key("slug"));

    let opts =
        ProcessOptions::default().required_fields(["title", "author"]);
    let err = process_metadata_with(&meta, &opts).unwrap_err();
    assert!(
        matches!(err, MetadataError::MissingFieldError(ref f) if f == "author")
    );

    let mut m = HashMap::new();
    m.insert("title".to_string(), "T".to_string());
    m.insert("date".to_string(), "not a date".to_string());
    let err = process_metadata_with(
        &Metadata::new(m),
        &ProcessOptions::default(),
    )
    .unwrap_err();
    assert!(matches!(err, MetadataError::DateParseError(_)));
}
