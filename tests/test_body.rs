// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `extract_metadata_with_body` returns the exact rest of the document
//! (issue #39): the body is a suffix slice of the input, so front matter
//! plus body reproduces the input byte for byte.

#![cfg(all(
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]

use metadata_gen::extract_metadata_with_body;
use proptest::prelude::*;

fn body() -> impl Strategy<Value = String> {
    // Starts with a non-space so the YAML rule (whitespace after the
    // closing fence is skipped) does not apply; may hold fences and braces.
    "[#a-z{}][a-z0-9 {}\n+-]{0,40}"
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
    #[test]
    fn body_is_the_rest_of_the_document(
        value in "[a-z]{1,10}",
        rest in body(),
        format in 0_usize..3,
    ) {
        let head = match format {
            0 => format!("---\nk: {value}\n---\n"),
            1 => format!("+++\nk = \"{value}\"\n+++\n"),
            _ => format!("{{\"k\": \"{value}\"}}\n"),
        };
        let doc = format!("{head}{rest}");
        let (meta, body) = extract_metadata_with_body(&doc).unwrap();
        prop_assert_eq!(meta.get("k").map(String::as_str), Some(value.as_str()));
        prop_assert_eq!(body, rest.as_str());
        // The body borrows from the input: same bytes, same address.
        let offset = body.as_ptr() as usize - doc.as_ptr() as usize;
        prop_assert_eq!(&doc[..offset], head.as_str());
    }
}

#[test]
fn crlf_after_the_fence_is_dropped_once() {
    let (_, body) =
        extract_metadata_with_body("+++\na = 1\n+++\r\n\r\nText")
            .unwrap();
    assert_eq!(body, "\r\nText");
}
