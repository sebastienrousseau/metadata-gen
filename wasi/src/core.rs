// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The component's behaviour as plain Rust, shared by the WASI export
//! and the native `expect` binary.

/// Front matter as `(key, value)` pairs sorted by key, or the error's
/// display text.
pub fn extract(content: &str) -> Result<Vec<(String, String)>, String> {
    let metadata =
        metadata_gen::extract_metadata(content).map_err(|e| e.to_string())?;
    let mut entries: Vec<(String, String)> =
        metadata.into_inner().into_iter().collect();
    entries.sort();
    Ok(entries)
}

/// Every `<meta>` tag as `(name, content)`, in document order.
pub fn extract_meta_tags(html: &str) -> Vec<(String, String)> {
    metadata_gen::metatags::extract_meta_tags_lenient(html)
        .0
        .into_iter()
        .map(|tag| (tag.name, tag.content))
        .collect()
}

/// `metadata_gen::escape_html`, unchanged.
pub fn escape_html(text: &str) -> String {
    metadata_gen::escape_html(text)
}
