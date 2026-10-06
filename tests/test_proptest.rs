// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Property-based tests for metadata-gen.
//!
//! Covers:
//! 1. HTML escape involution: `unescape_html(escape_html(s)) == s`
//! 2. Front-matter round-trip for YAML, TOML, and JSON formats
//! 3. Replay of persisted regression inputs in `tests/regression_corpus/`

use metadata_gen::extract_metadata;
use metadata_gen::utils::{escape_html, unescape_html};
use proptest::collection::hash_map;
use proptest::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Key strategy: non-empty alphanumeric identifiers suitable for all 3 formats.
fn valid_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}"
        .prop_filter("keys must not be empty", |k| !k.is_empty())
}

// Value strategy: printable strings without control characters, quotes, or newlines.
fn valid_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,!?-]{1,30}"
}

fn metadata_map_strategy(
) -> impl Strategy<Value = HashMap<String, String>> {
    hash_map(valid_key(), valid_value(), 1..=8)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        .. ProptestConfig::default()
    })]

    /// HTML escape involution property on ASCII strings:
    /// unescape_html(escape_html(s)) == s
    #[test]
    fn test_html_escape_involution_ascii(ref s in "[\\x20-\\x7E]{0,256}") {
        let escaped = escape_html(s);
        let unescaped = unescape_html(&escaped);
        prop_assert_eq!(s.as_str(), unescaped.as_str());
    }

    /// HTML escape involution property on general Unicode strings:
    /// unescape_html(escape_html(s)) == s
    #[test]
    fn test_html_escape_involution_unicode(ref s in "\\PC{0,128}") {
        let escaped = escape_html(s);
        let unescaped = unescape_html(&escaped);
        prop_assert_eq!(s.as_str(), unescaped.as_str());
    }

    /// YAML front-matter round-trip property:
    /// Generating valid key-value pairs formatted as YAML front-matter
    /// round-trips through extract_metadata to an equivalent map.
    #[test]
    fn test_yaml_front_matter_round_trip(map in metadata_map_strategy()) {
        let mut yaml_text = String::from("---\n");
        for (k, v) in &map {
            // Escape any embedded quotes for safe YAML string literal
            let escaped_v = v.replace('"', "\\\"");
            yaml_text.push_str(&format!("{}: \"{}\"\n", k, escaped_v));
        }
        yaml_text.push_str("---\n# Body follows\nContent.\n");

        let extracted = extract_metadata(&yaml_text);
        prop_assert!(extracted.is_ok(), "YAML extraction failed for: {}", yaml_text);
        let result_map = extracted.unwrap().into_inner();
        for (k, v) in &map {
            prop_assert_eq!(result_map.get(k), Some(v));
        }
    }

    /// TOML front-matter round-trip property:
    /// Generating valid key-value pairs formatted as TOML front-matter
    /// round-trips through extract_metadata to an equivalent map.
    #[test]
    fn test_toml_front_matter_round_trip(map in metadata_map_strategy()) {
        let mut toml_text = String::from("+++\n");
        for (k, v) in &map {
            let escaped_v = v.replace('"', "\\\"");
            toml_text.push_str(&format!("{} = \"{}\"\n", k, escaped_v));
        }
        toml_text.push_str("+++\n# Body follows\nContent.\n");

        let extracted = extract_metadata(&toml_text);
        prop_assert!(extracted.is_ok(), "TOML extraction failed for: {}", toml_text);
        let result_map = extracted.unwrap().into_inner();
        for (k, v) in &map {
            prop_assert_eq!(result_map.get(k), Some(v));
        }
    }

    /// JSON front-matter round-trip property:
    /// Generating valid key-value pairs formatted as JSON front-matter
    /// round-trips through extract_metadata to an equivalent map.
    #[test]
    fn test_json_front_matter_round_trip(map in metadata_map_strategy()) {
        let json_obj = serde_json::to_string_pretty(&map).unwrap();
        let doc = format!("{}\n# Body follows\nContent.\n", json_obj);

        let extracted = extract_metadata(&doc);
        prop_assert!(extracted.is_ok(), "JSON extraction failed for: {}", doc);
        let result_map = extracted.unwrap().into_inner();
        for (k, v) in &map {
            prop_assert_eq!(result_map.get(k), Some(v));
        }
    }
}

/// Regression corpus replay:
/// Every committed counterexample in `tests/regression_corpus/` must pass
/// the HTML escape involution property.
#[test]
fn test_regression_corpus_replay() {
    let corpus_dir = Path::new("tests/regression_corpus");
    if !corpus_dir.exists() {
        return;
    }

    let entries = fs::read_dir(corpus_dir)
        .expect("failed to read regression corpus directory");
    let mut replayed = 0;

    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.is_file()
            && !path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with('.')
        {
            let content =
                fs::read_to_string(&path).unwrap_or_else(|_| {
                    panic!("failed to read corpus file: {:?}", path)
                });
            let escaped = escape_html(&content);
            let unescaped = unescape_html(&escaped);
            assert_eq!(
                content, unescaped,
                "Regression failed on corpus file {:?}",
                path
            );
            replayed += 1;
        }
    }

    println!("Replayed {} regression corpus cases.", replayed);
}
