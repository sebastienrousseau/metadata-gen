// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The streaming events API and the zero-copy map built on it
//! (issues #46 and #50).

#![cfg(all(
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]

use metadata_gen::metadata::events::{events, events_into, Scalar};
use metadata_gen::{
    extract_metadata, extract_metadata_cow, for_each_event, Event,
    MetadataError,
};
use std::borrow::Cow;
use std::ops::ControlFlow;

fn key(k: &str) -> Event<'_> {
    Event::Key(Cow::Borrowed(k))
}

fn text(s: &str) -> Event<'_> {
    Event::Scalar(Scalar::Str(Cow::Borrowed(s)))
}

#[test]
fn yaml_events_in_document_order() {
    let doc =
        "---\ntitle: T\ntags: [a, 2]\nauthor:\n  name: Ada\n---\n";
    assert_eq!(
        events(doc).unwrap(),
        vec![
            Event::MapStart,
            key("title"),
            text("T"),
            key("tags"),
            Event::SeqStart,
            text("a"),
            Event::Scalar(Scalar::Int(2)),
            Event::SeqEnd,
            key("author"),
            Event::MapStart,
            key("name"),
            text("Ada"),
            Event::MapEnd,
            Event::MapEnd,
        ]
    );
}

#[test]
fn toml_datetime_is_one_scalar() {
    let doc = "+++\ndate = 2024-01-02\nf = 1.5\nbig = true\n+++\n";
    // The `toml` crate orders table keys alphabetically.
    let all = events(doc).unwrap();
    assert_eq!(all.len(), 8);
    assert_eq!(all[1], key("big"));
    assert_eq!(all[2], Event::Scalar(Scalar::Bool(true)));
    assert_eq!(all[3], key("date"));
    assert_eq!(all[4], Event::Scalar(Scalar::Str("2024-01-02".into())));
    assert_eq!(all[6], Event::Scalar(Scalar::Float(1.5)));
}

#[test]
fn json_scalars_cover_every_kind() {
    let doc = "{\"n\": null, \"u\": 18446744073709551615, \"i\": -1}";
    let all = events(doc).unwrap();
    assert_eq!(all[2], Event::Scalar(Scalar::Null));
    assert_eq!(all[4], Event::Scalar(Scalar::UInt(u64::MAX)));
    assert_eq!(all[6], Event::Scalar(Scalar::Int(-1)));
    assert_eq!(Scalar::UInt(7).to_text(), "7");
    assert_eq!(Scalar::Null.to_string(), "null");
}

#[test]
fn breaking_stops_the_walk() {
    let doc = "{\"a\": 1, \"b\": 2, \"c\": 3}";
    let mut seen = 0;
    let flow = for_each_event(doc, |e| {
        seen += 1;
        if e == key("b") {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .unwrap();
    assert!(flow.is_break());
    assert_eq!(seen, 4); // MapStart, a, 1, b
}

#[test]
fn breaking_on_the_yaml_fallback_path_stops_too() {
    let doc = "---\nbase: &b x\ncopy: *b\nmore: y\n---\n";
    let mut seen = Vec::new();
    let flow = for_each_event(doc, |e| {
        let stop = e == Event::Key(Cow::Owned("more".into()));
        seen.push(e);
        if stop {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .unwrap();
    assert!(flow.is_break());
    assert_eq!(seen.len(), 6);
}

#[test]
fn aliases_replay_without_duplicates() {
    let doc = "---\nbase: &b {k: v}\ncopy: *b\n---\n";
    let flat = extract_metadata(doc).unwrap().into_inner();
    let cow =
        extract_metadata_cow(doc).unwrap().into_owned().into_inner();
    assert_eq!(cow, flat);
    let all = events(doc).unwrap();
    assert_eq!(
        all.iter().filter(|e| **e == Event::MapStart).count(),
        3
    );
}

#[test]
fn tagged_and_merged_values_flatten() {
    let doc = "---\nbase: &b {k: v}\nm:\n  <<: *b\n  own: 1\nt: !custom 5\n---\n";
    let cow = extract_metadata_cow(doc).unwrap();
    assert_eq!(cow.get("m.k"), Some("v"));
    assert_eq!(cow.get("m.own"), Some("1"));
    assert_eq!(cow.get("t"), Some("5"));
}

#[test]
fn parse_error_mid_stream_is_reported_after_earlier_events() {
    let doc = "{\"a\": 1, \"b\": }";
    let mut seen = 0;
    let err = for_each_event(doc, |_| {
        seen += 1;
        ControlFlow::Continue(())
    })
    .unwrap_err();
    assert!(matches!(err, MetadataError::Parse { .. }), "{err:?}");
    assert_eq!(seen, 4);
    assert!(events("---\na: [\n---\n").is_err());
    assert!(events("+++\na = \n+++\n").is_err());
}

#[test]
fn missing_and_unclosed_blocks_are_errors() {
    assert!(matches!(
        events("plain text"),
        Err(MetadataError::ExtractionError { .. })
    ));
    let err = events("---\ntitle: x\n").unwrap_err();
    assert!(err.to_string().contains("no closing fence"), "{err}");
    let big = format!("---\nk: {}\n---\n", "x".repeat(2 * 1024 * 1024));
    assert!(events(&big).unwrap_err().to_string().contains("exceeds"));
}

#[test]
fn caller_buffer_is_appended_to() {
    let mut buffer = vec![Event::MapEnd];
    events_into("---\na: 1\n---\n", &mut buffer).unwrap();
    assert_eq!(buffer.len(), 5);
}

#[test]
fn most_plain_yaml_values_are_borrowed() {
    let mut doc = String::from("---\n");
    for i in 0..100 {
        doc.push_str(&format!("key{i}: value number {i}\n"));
    }
    doc.push_str("escaped: \"line1\\nline2\"\n---\nbody");
    let cow = extract_metadata_cow(&doc).unwrap();
    assert_eq!(cow.len(), 101);
    assert!(cow.borrowed_values() >= 100, "{}", cow.borrowed_values());
    assert!(matches!(cow.get_cow("escaped"), Some(Cow::Owned(_))));
    assert_eq!(cow.get("escaped"), Some("line1\nline2"));
}
