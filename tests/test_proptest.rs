// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Property-based tests for metadata-gen.
//!
//! Covers:
//! 1. HTML escape involution: `unescape_html(escape_html(s)) == s`
//! 2. Front-matter round-trip for YAML, TOML, and JSON formats
//! 3. Replay of persisted regression inputs in `tests/regression_corpus/`

// The suites exercise the default feature set.
#![cfg(all(
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]

use metadata_gen::extract_metadata;
use metadata_gen::utils::{escape_html, unescape_html};
use proptest::collection::hash_map;
use proptest::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Key strategy: identifiers every format accepts as a bare key.
fn valid_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}"
}

// Value strategy: any printable Unicode, quotes and backslashes
// included, so the serialisers below must escape correctly. No
// leading or trailing whitespace: a quoted scalar keeps it, but the
// flat map is documented as trimmed text.
fn valid_value() -> impl Strategy<Value = String> {
    "\\PC{0,30}"
        .prop_filter("no surrounding whitespace", |v| v.trim() == v)
}

fn metadata_map_strategy(
) -> impl Strategy<Value = HashMap<String, String>> {
    hash_map(valid_key(), valid_value(), 1..=8)
}

/// A double-quoted scalar for YAML and a basic string for TOML: both
/// take `\\` and `\"` as escapes, and everything else printable as is.
fn quoted(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `PROPTEST_CASES`, when set. CI's proptest job sets 10 000.
fn configured_cases() -> Option<u32> {
    std::env::var("PROPTEST_CASES").ok()?.parse().ok()
}

/// Cases per round-trip property: `PROPTEST_CASES` when set, otherwise
/// 1 000 so a plain `cargo test` stays quick. Under Miri a handful is
/// enough to exercise the code paths.
fn round_trip_cases() -> u32 {
    if cfg!(miri) {
        return 4;
    }
    configured_cases().unwrap_or(1_000)
}

/// The ASCII involution is cheap per case, so a configured run gives it
/// ten times as many: 100 000 when CI sets `PROPTEST_CASES=10000`.
fn involution_cases() -> u32 {
    match configured_cases() {
        Some(cases) if !cfg!(miri) => cases.saturating_mul(10),
        _ => round_trip_cases(),
    }
}

/// The suite's configuration with `cases` cases. Under Miri failures are
/// not persisted: proptest's file persistence calls `getcwd`, which
/// Miri's isolation forbids.
fn config(cases: u32) -> ProptestConfig {
    let mut config = ProptestConfig::with_cases(cases);
    if cfg!(miri) {
        config.failure_persistence = None;
    }
    config
}

/// Extracts `doc` and asserts the flat map equals `map` exactly: no key
/// lost, none invented, every value unchanged.
fn assert_round_trip(
    doc: &str,
    map: &HashMap<String, String>,
) -> Result<(), TestCaseError> {
    let extracted = extract_metadata(doc);
    prop_assert!(extracted.is_ok(), "extraction failed for: {doc}");
    let result_map: HashMap<String, String> =
        extracted.unwrap().into_inner().into_iter().collect();
    prop_assert_eq!(&result_map, map, "document: {}", doc);
    Ok(())
}

proptest! {
    #![proptest_config(config(involution_cases()))]

    /// HTML escape involution property on ASCII strings:
    /// unescape_html(escape_html(s)) == s
    #[test]
    fn test_html_escape_involution_ascii(ref s in "[\\x00-\\x7F]{0,256}") {
        let escaped = escape_html(s);
        let unescaped = unescape_html(&escaped);
        prop_assert_eq!(s.as_str(), unescaped.as_str());
    }
}

proptest! {
    #![proptest_config(config(round_trip_cases()))]

    /// HTML escape involution property on general Unicode strings:
    /// unescape_html(escape_html(s)) == s
    #[test]
    fn test_html_escape_involution_unicode(ref s in "\\PC{0,128}") {
        let escaped = escape_html(s);
        let unescaped = unescape_html(&escaped);
        prop_assert_eq!(s.as_str(), unescaped.as_str());
    }

    /// Escaping never leaves a raw markup character behind.
    #[test]
    fn test_escape_html_output_has_no_raw_specials(ref s in "\\PC{0,128}") {
        let escaped = escape_html(s);
        prop_assert!(!escaped.contains(['<', '>', '"', '\'']));
    }

    /// YAML: a map serialised as double-quoted front matter extracts to
    /// the same map.
    #[test]
    fn test_yaml_front_matter_round_trip(map in metadata_map_strategy()) {
        let mut doc = String::from("---\n");
        for (k, v) in &map {
            doc.push_str(&format!("{k}: {}\n", quoted(v)));
        }
        doc.push_str("---\n# Body follows\nContent.\n");
        assert_round_trip(&doc, &map)?;
    }

    /// TOML: a map serialised as basic strings extracts to the same map.
    #[test]
    fn test_toml_front_matter_round_trip(map in metadata_map_strategy()) {
        let mut doc = String::from("+++\n");
        for (k, v) in &map {
            doc.push_str(&format!("{k} = {}\n", quoted(v)));
        }
        doc.push_str("+++\n# Body follows\nContent.\n");
        assert_round_trip(&doc, &map)?;
    }

    /// JSON: a map serialised by serde_json extracts to the same map.
    #[test]
    fn test_json_front_matter_round_trip(map in metadata_map_strategy()) {
        let json_obj = serde_json::to_string_pretty(&map).unwrap();
        let doc = format!("{json_obj}\n# Body follows\nContent.\n");
        assert_round_trip(&doc, &map)?;
    }
}

/// The five-`replace` chain `escape_html` replaced; its output is the
/// contract (#47), so the single-pass version must match it byte for byte.
fn replace_chain(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

proptest! {
    #![proptest_config(config(round_trip_cases()))]

    /// `escape_html` is byte-identical to the replace chain it replaced.
    #[test]
    fn test_escape_html_matches_replace_chain(ref s in "\\PC{0,128}") {
        prop_assert_eq!(escape_html(s), replace_chain(s));
    }
}

/// Every file in `dir`, skipping dotfiles, as `(path, contents)`.
fn corpus(dir: &str) -> Vec<(std::path::PathBuf, String)> {
    let mut files: Vec<_> = fs::read_dir(Path::new(dir))
        .unwrap_or_else(|e| panic!("reading {dir}: {e}"))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| {
            path.is_file()
                && !path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with('.')
        })
        .map(|path| {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reading {path:?}: {e}"));
            (path, text)
        })
        .collect();
    files.sort();
    files
}

/// Escape corpus replay (#47): each committed input escapes exactly as
/// the replace chain did and unescapes back to itself.
#[test]
#[cfg_attr(miri, ignore = "reads tests/; Miri isolation forbids it")]
fn test_escape_corpus_replay() {
    let files = corpus("tests/escape_corpus");
    assert!(!files.is_empty(), "tests/escape_corpus is empty");
    for (path, content) in files {
        let escaped = escape_html(&content);
        assert_eq!(escaped, replace_chain(&content), "{path:?}");
        assert_eq!(unescape_html(&escaped), content, "{path:?}");
    }
}

/// Regression corpus replay:
/// Every committed counterexample in `tests/regression_corpus/` must pass
/// the HTML escape involution property.
#[test]
#[cfg_attr(miri, ignore = "reads tests/; Miri isolation forbids it")]
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
