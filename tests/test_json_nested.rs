// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! JSON front matter with nested objects (issue #26).
//!
//! Up to 0.0.4 the JSON detector was the lazy regex
//! `^\s*\{\s*(.*?)\s*\}`, which stopped at the first `}`: a nested object
//! lost everything after its own closing brace, and a body holding braces
//! could be read as front matter. These tests pin the balanced-value
//! behaviour, including a body that carries `{` and `}` of its own.

#![cfg(all(feature = "std", feature = "json"))]

use metadata_gen::{
    extract_metadata, extract_metadata_with_body, FrontMatterFormat,
    MetadataError,
};
use proptest::prelude::*;
use serde_json::{Map, Value};
use std::collections::HashMap;

#[test]
fn nested_object_is_not_truncated_at_first_brace() {
    let doc = "{\"a\": {\"b\": \"c\"}, \"after\": \"kept\"}\n# body";
    let meta = extract_metadata(doc).unwrap();
    assert_eq!(meta.get("a.b").map(String::as_str), Some("c"));
    assert_eq!(meta.get("after").map(String::as_str), Some("kept"));
}

#[test]
fn array_of_objects_keeps_every_element() {
    let doc = "{\"tags\": [{\"name\":\"x\"},{\"name\":\"y\"}]}\n# body";
    let meta = extract_metadata(doc).unwrap();
    let tags = meta.get("tags").unwrap();
    assert!(tags.contains("\"name\":\"x\""), "{tags}");
    assert!(tags.contains("\"name\":\"y\""), "{tags}");
}

#[test]
fn braces_in_the_body_stay_in_the_body() {
    let doc = "{\"title\": \"T\", \"a\": {\"b\": 1}}\n\
               # Body\n\nfn main() { let x = {1}; }\n{\"not\": \"meta\"}\n";
    let (meta, body) = extract_metadata_with_body(doc).unwrap();
    assert_eq!(meta.get("title").map(String::as_str), Some("T"));
    assert_eq!(meta.get("a.b").map(String::as_str), Some("1"));
    assert!(!meta.contains_key("not"));
    assert_eq!(
        body,
        "# Body\n\nfn main() { let x = {1}; }\n{\"not\": \"meta\"}\n"
    );
}

#[test]
fn unmatched_brace_reports_the_parser_error() {
    let err =
        extract_metadata("{\"a\": {\"b\": 1}\n# body").unwrap_err();
    assert!(
        matches!(
            err,
            MetadataError::Parse {
                format: FrontMatterFormat::Json,
                ..
            }
        ),
        "{err:?}"
    );
    assert!(!err.to_string().contains("No valid front matter"));
}

/// Flattens `value` the way `extract_metadata` documents it: dotted keys
/// for objects, `[a, b]` for arrays, strings without quotes.
fn expected(
    value: &Value,
    prefix: &str,
    out: &mut HashMap<String, String>,
) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                expected(v, &key, out);
            }
        }
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(scalar).collect();
            let _ = out.insert(
                prefix.to_string(),
                format!("[{}]", parts.join(", ")),
            );
        }
        other => {
            let _ = out.insert(prefix.to_string(), scalar(other));
        }
    }
}

fn scalar(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn leaf() -> impl Strategy<Value = Value> {
    prop_oneof![
        "[a-zA-Z0-9 {}\\[\\],:]{0,12}".prop_map(Value::String),
        any::<i32>().prop_map(Value::from),
        any::<bool>().prop_map(Value::Bool),
    ]
}

fn nested_object() -> impl Strategy<Value = Map<String, Value>> {
    let value = leaf().prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            proptest::collection::vec(leaf(), 0..4)
                .prop_map(Value::Array),
            proptest::collection::btree_map("[a-z]{1,6}", inner, 1..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    });
    proptest::collection::btree_map("[a-z]{1,6}", value, 1..6)
        .prop_map(|m| m.into_iter().collect())
}

proptest! {
    // Under Miri failures are not persisted: proptest's file
    // persistence calls `getcwd`, which Miri's isolation forbids.
    #![proptest_config(ProptestConfig {
        failure_persistence: if cfg!(miri) {
            None
        } else {
            ProptestConfig::default().failure_persistence
        },
        ..ProptestConfig::default()
    })]
    /// Any object, nested or not, followed by a body full of braces, is
    /// read in full and the body is left alone.
    #[test]
    fn nested_objects_round_trip(object in nested_object()) {
        let json = serde_json::to_string(&Value::Object(object.clone())).unwrap();
        let doc = format!("{json}\n}} {{ body }}\n");
        let (meta, body) = extract_metadata_with_body(&doc).unwrap();
        let mut want = HashMap::new();
        expected(&Value::Object(object), "", &mut want);
        prop_assert_eq!(meta.into_inner(), want);
        prop_assert_eq!(body, "} { body }\n");
    }
}
