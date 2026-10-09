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
fn context_boxes_other_errors_with_a_source_chain() {
    let inner: Box<dyn std::error::Error + Send + Sync> =
        "root cause".into();
    let e = MetadataError::Other(inner).context("outer");
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

#[test]
fn context_wraps_a_parse_error_and_keeps_its_span() {
    let source = serde_json::from_str::<u8>("x").unwrap_err();
    let e = MetadataError::parse(
        FrontMatterFormat::Json,
        Some(0..1),
        source,
    )
    .context("post.md");
    let MetadataError::Parse {
        format,
        ref span,
        ref source,
    } = e
    else {
        panic!("expected Parse, got {e:?}");
    };
    assert_eq!(format, FrontMatterFormat::Json);
    assert_eq!(span.clone(), Some(0..1));
    assert!(source.to_string().starts_with("post.md: "), "{source}");
    assert!(e.to_string().contains("at bytes 0..1"), "{e}");
}

#[test]
fn parse_error_without_a_span_has_no_suffix() {
    let source = serde_json::from_str::<u8>("x").unwrap_err();
    let e = MetadataError::parse(FrontMatterFormat::Yaml, None, source);
    let text = e.to_string();
    assert!(
        text.starts_with("yaml front matter failed to parse: "),
        "{text}"
    );
    assert!(!text.contains("at bytes"), "{text}");
}

#[test]
fn other_boxes_any_error() {
    let e = MetadataError::other(core::fmt::Error);
    assert!(matches!(e, MetadataError::Other(_)));
    assert!(e.to_string().starts_with("Unexpected error: "), "{e}");
}

#[test]
fn prefix_message_leaves_wrapped_errors_to_context() {
    // `context` handles these variants itself; `prefix_message` must hand
    // them back untouched rather than lose them.
    let e =
        MetadataError::other(core::fmt::Error).prefix_message(&"ctx");
    assert!(matches!(e, MetadataError::Other(_)));
    assert!(!e.to_string().contains("ctx"), "{e}");
}

#[test]
fn context_leaves_utf8_errors_untouched() {
    let bytes = vec![0xffu8];
    let utf8 = core::str::from_utf8(&bytes).unwrap_err();
    let e = MetadataError::from(utf8).context("ignored");
    assert!(e.to_string().starts_with("UTF-8 decoding error:"), "{e}");
    assert!(!e.to_string().contains("ignored"));
}
