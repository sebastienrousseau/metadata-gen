// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for `error`; a child module so private items stay reachable.

//! `MetadataError::context` rewrites every variant. Each arm is a
//! separate branch, so each needs its own case: a missed arm here is
//! a variant whose context silently disappears.

use super::*;

fn assert_prefixed(err: &MetadataError, ctx: &str) {
    let text = err.to_string();
    assert!(
        text.contains(ctx),
        "context {ctx:?} missing from {text:?}"
    );
}

#[test]
fn context_prefixes_extraction_and_processing() {
    let e = MetadataError::new_extraction_error("bad").context("front");
    assert!(
        matches!(e, MetadataError::ExtractionError { ref message } if message == "front: bad")
    );
    let e = MetadataError::new_processing_error("bad").context("front");
    assert!(
        matches!(e, MetadataError::ProcessingError { ref message } if message == "front: bad")
    );
}

#[test]
fn context_prefixes_string_payload_variants() {
    let e = MetadataError::MissingFieldError("title".into())
        .context("page");
    assert!(
        matches!(e, MetadataError::MissingFieldError(ref s) if s == "page: title")
    );
    let e = MetadataError::DateParseError("x".into()).context("page");
    assert!(
        matches!(e, MetadataError::DateParseError(ref s) if s == "page: x")
    );
    let e = MetadataError::UnsupportedFormatError("ini".into())
        .context("page");
    assert!(
        matches!(e, MetadataError::UnsupportedFormatError(ref s) if s == "page: ini")
    );
    let e = MetadataError::new_validation_error("title", "empty")
        .context("page");
    assert!(matches!(
        e,
        MetadataError::ValidationError { ref field, ref message }
            if field == "title" && message == "page: empty"
    ));
}

#[test]
fn context_wraps_io_error_and_keeps_its_kind() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
    let e = MetadataError::from(io).context("reading");
    match e {
        MetadataError::IoError(inner) => {
            assert_eq!(inner.kind(), std::io::ErrorKind::NotFound);
            assert_eq!(inner.to_string(), "reading: gone");
        }
        other => panic!("expected IoError, got {other:?}"),
    }
}

#[test]
fn context_rewrites_parser_errors_as_custom_messages() {
    let yaml = noyalib::from_str::<noyalib::Value>("a: [").unwrap_err();
    let e = MetadataError::from(yaml).context("yaml");
    assert!(matches!(e, MetadataError::YamlError(_)));
    assert_prefixed(&e, "yaml");

    let json =
        serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let e = MetadataError::from(json).context("json");
    assert!(matches!(e, MetadataError::JsonError(_)));
    assert_prefixed(&e, "json");

    let toml = toml::from_str::<toml::Value>("a = ").unwrap_err();
    let e = MetadataError::from(toml).context("toml");
    assert!(matches!(e, MetadataError::TomlError(_)));
    assert_prefixed(&e, "toml");
}

#[test]
fn context_leaves_utf8_errors_untouched() {
    let bytes = vec![0xffu8];
    let utf8 = std::str::from_utf8(&bytes).unwrap_err();
    let e = MetadataError::from(utf8).context("ignored");
    assert!(matches!(e, MetadataError::Utf8Error(_)));
    assert!(!e.to_string().contains("ignored"));
}

#[test]
fn context_boxes_other_errors_with_a_source_chain() {
    let inner: Box<dyn std::error::Error + Send + Sync> =
        "root cause".into();
    let e = MetadataError::from(inner).context("outer");
    let MetadataError::Other(boxed) = &e else {
        panic!("expected Other, got {e:?}");
    };
    // ContextError renders "context: source" and keeps the source
    // reachable for callers walking the chain.
    assert_eq!(boxed.to_string(), "outer: root cause");
    let source = boxed.source().expect("ContextError keeps its source");
    assert_eq!(source.to_string(), "root cause");
    assert_eq!(e.to_string(), "Unexpected error: outer: root cause");
}
