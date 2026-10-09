// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for `metadata`; a child module so private items stay reachable.

use super::*;
use crate::MetadataMap;

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
        Err(MetadataError::Parse {
            format: FrontMatterFormat::Yaml,
            ..
        })
    ));
    assert!(matches!(
        extract_typed::<Front>("+++\ntitle = \n+++\n"),
        Err(MetadataError::Parse {
            format: FrontMatterFormat::Toml,
            ..
        })
    ));
    assert!(
        matches!(
            extract_typed::<Front>("{\"title\": \"T\"}"),
            Err(MetadataError::Parse {
                format: FrontMatterFormat::Json,
                ..
            })
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
        Err(MetadataError::Parse {
            format: FrontMatterFormat::Json,
            ..
        })
    ));
}

#[test]
fn process_options_control_required_fields_and_slug() {
    let mut m = MetadataMap::new();
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

    let mut m = MetadataMap::new();
    m.insert("title".to_string(), "T".to_string());
    m.insert("date".to_string(), "not a date".to_string());
    let err = process_metadata_with(
        &Metadata::new(m),
        &ProcessOptions::default(),
    )
    .unwrap_err();
    assert!(matches!(err, MetadataError::DateParseError(_)));
}

#[test]
fn parse_limits_bound_block_size_and_depth() {
    let doc = "---\ntitle: T\nnested: {a: {b: {c: 1}}}\n---\n";
    assert!(extract_metadata_with_limits(doc, &ParseLimits::default())
        .is_ok());

    let err = extract_metadata_with_limits(
        doc,
        &ParseLimits::default().max_block_bytes(8),
    )
    .unwrap_err();
    assert!(
        matches!(err, MetadataError::ExtractionError { ref message } if message.contains("exceeds the 8-byte limit")),
        "{err:?}"
    );

    let err = extract_metadata_with_limits(
        doc,
        &ParseLimits::default().max_depth(2),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            MetadataError::Parse {
                format: FrontMatterFormat::Yaml,
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn front_matter_format_names() {
    for (format, name) in [
        (FrontMatterFormat::Yaml, "yaml"),
        (FrontMatterFormat::Toml, "toml"),
        (FrontMatterFormat::Json, "json"),
    ] {
        assert_eq!(format.as_str(), name);
        assert_eq!(format.to_string(), name);
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct Borrowed<'a> {
    title: &'a str,
    tags: Vec<&'a str>,
}

#[test]
fn typed_borrowed_points_into_the_document() {
    let docs = [
        "---\ntitle: B\ntags: [x, y]\n---\nbody",
        "+++\ntitle = \"B\"\ntags = [\"x\", \"y\"]\n+++\nbody",
        "{\"title\": \"B\", \"tags\": [\"x\", \"y\"]}\nbody",
    ];
    for doc in docs {
        let got: Borrowed<'_> = extract_typed_borrowed(doc).expect(doc);
        assert_eq!(got.title, "B", "{doc:?}");
        assert_eq!(got.tags, ["x", "y"], "{doc:?}");
        let start = doc.as_ptr() as usize;
        let at = got.title.as_ptr() as usize;
        assert!(
            at >= start && at < start + doc.len(),
            "{doc:?} copied"
        );
    }
    assert!(matches!(
        extract_typed_borrowed::<Borrowed<'_>>("no front matter"),
        Err(MetadataError::ExtractionError { .. })
    ));
    assert!(matches!(
        extract_typed_borrowed::<Borrowed<'_>>("---\ntitle: [\n---\n"),
        Err(MetadataError::Parse {
            format: FrontMatterFormat::Yaml,
            ..
        })
    ));
}

#[test]
fn month_first_dates() {
    let mut m = MetadataMap::new();
    m.insert("title".to_string(), "T".to_string());
    m.insert("date".to_string(), "03/04/2024".to_string());
    let opts =
        ProcessOptions::default().date_order(DateOrder::MonthFirst);
    let out = process_metadata_with(&Metadata::new(m.clone()), &opts)
        .unwrap();
    assert_eq!(out.get("date").unwrap(), "2024-03-04");
    let out = process_metadata(&Metadata::new(m.clone())).unwrap();
    assert_eq!(out.get("date").unwrap(), "2024-04-03");

    m.insert("date".to_string(), "3/04/20245".to_string());
    let err =
        process_metadata_with(&Metadata::new(m), &opts).unwrap_err();
    assert!(
        matches!(err, MetadataError::DateParseError(ref msg) if msg.contains("MM/DD/YYYY")),
        "{err:?}"
    );
}
