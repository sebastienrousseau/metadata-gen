// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WASI 0.2 component wrapper around `metadata-gen`.
//!
//! The exported functions are thin adapters: every result comes from
//! the same `metadata-gen` call a native caller would make. The
//! [`core`] module holds those adapters as plain Rust so the native
//! `expect` binary can print what the component must return.

pub mod core;

#[cfg(target_family = "wasm")]
mod bindings {
    wit_bindgen::generate!({
        path: "../wit",
        world: "metadata-gen",
    });

    use self::exports::metadata_gen::lib::metadata::{
        Entry, Guest, MetaTag,
    };

    struct Component;

    impl Guest for Component {
        fn extract(content: String) -> Result<Vec<Entry>, String> {
            crate::core::extract(&content).map(|entries| {
                entries
                    .into_iter()
                    .map(|(key, value)| Entry { key, value })
                    .collect()
            })
        }

        fn extract_meta_tags(html: String) -> Vec<MetaTag> {
            crate::core::extract_meta_tags(&html)
                .into_iter()
                .map(|(name, content)| MetaTag { name, content })
                .collect()
        }

        fn escape_html(text: String) -> String {
            crate::core::escape_html(&text)
        }
    }

    export!(Component);
}
