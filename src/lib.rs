#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://cloudcdn.pro/metadata-gen/v1/favicon.ico",
    html_logo_url = "https://cloudcdn.pro/metadata-gen/v1/logos/metadata-gen.svg",
    html_root_url = "https://docs.rs/metadata-gen"
)]
#![crate_name = "metadata_gen"]
#![crate_type = "lib"]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// The `error` module contains error types for metadata processing.
pub mod error;
/// Synchronous readers and files (`std`).
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod io;
/// The `metadata` module contains functions for extracting and processing metadata.
pub mod metadata;
/// The `metatags` module contains functions for generating meta tags.
pub mod metatags;
/// Async file helpers on the Tokio runtime (feature `tokio`).
#[cfg(all(feature = "tokio", not(loom)))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;
/// The `utils` module contains utility functions for metadata processing.
pub mod utils;

pub use error::MetadataError;
pub use metadata::{
    detect_front_matter, extract_metadata, extract_metadata_with_body,
    extract_metadata_with_limits, extract_typed,
    extract_typed_borrowed, process_metadata, process_metadata_with,
    DateOrder, FrontMatterFormat, Metadata, ParseLimits,
    ProcessOptions,
};
pub use metatags::{generate_metatags, MetaTag, MetaTagGroups};
#[cfg(all(feature = "tokio", not(loom)))]
pub use utils::async_extract_metadata_from_file;
pub use utils::{escape_attribute, escape_html};

/// Type alias for a map of metadata key-value pairs.
///
/// With the `std` feature (the default) this is `std::collections::HashMap`;
/// in a `no_std + alloc` build it is `alloc::collections::BTreeMap`, which
/// has the same `get`, `insert`, `iter` and `contains_key` surface.
///
/// # Example
///
/// ```
/// use metadata_gen::MetadataMap;
///
/// let mut map = MetadataMap::new();
/// map.insert("title".to_string(), "My Page".to_string());
/// assert_eq!(map.get("title"), Some(&"My Page".to_string()));
/// ```
#[cfg(feature = "std")]
pub type MetadataMap = std::collections::HashMap<String, String>;
/// Type alias for a map of metadata key-value pairs (`no_std` build).
#[cfg(not(feature = "std"))]
pub type MetadataMap = alloc::collections::BTreeMap<String, String>;

/// Type alias for a list of keywords.
///
/// # Example
///
/// ```
/// use metadata_gen::Keywords;
///
/// let keywords: Keywords = vec!["rust".to_string(), "metadata".to_string()];
/// assert_eq!(keywords.len(), 2);
/// ```
pub type Keywords = Vec<String>;

/// Type alias for the result of metadata extraction and processing.
///
/// # Example
///
/// ```
/// use metadata_gen::{MetadataResult, extract_and_prepare_metadata};
///
/// let result: MetadataResult = extract_and_prepare_metadata("---\ntitle: Test\n---\n");
/// assert!(result.is_ok());
/// ```
pub type MetadataResult =
    Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError>;

/// Extracts metadata from the content, generates keywords based on the metadata,
/// and prepares meta tag groups.
///
/// This function performs three key tasks:
/// 1. It extracts metadata from the front matter of the content.
/// 2. It generates keywords based on this metadata.
/// 3. It generates various meta tags required for the page.
///
/// # Arguments
///
/// * `content` - A string slice representing the content from which to extract metadata.
///
/// # Returns
///
/// Returns a Result containing a tuple with:
/// * [`MetadataMap`]: Extracted metadata
/// * `Vec<String>`: A list of keywords
/// * `MetaTagGroups`: A structure containing various meta tags
///
/// # Errors
///
/// This function will return a `MetadataError` if metadata extraction or processing fails.
///
/// # Example
///
/// ```
/// use metadata_gen::extract_and_prepare_metadata;
///
/// let content = r#"---
/// title: My Page
/// description: A sample page
/// ---
/// # Content goes here
/// "#;
///
/// let result = extract_and_prepare_metadata(content);
/// assert!(result.is_ok());
/// ```
pub fn extract_and_prepare_metadata(content: &str) -> MetadataResult {
    // Ensure the front matter format contains key-value separators
    if !content.contains(':') && !content.contains('=') {
        return Err(MetadataError::ExtractionError {
            message: "No valid front matter found".to_string(),
        });
    }

    let metadata = extract_metadata(content)?;
    let metadata_map = metadata.into_inner();
    let keywords = extract_keywords(&metadata_map);
    let all_meta_tags = generate_metatags(&metadata_map);

    Ok((metadata_map, keywords, all_meta_tags))
}

/// Extracts keywords from the metadata.
///
/// This function looks for a "keywords" key in the metadata and splits its value into a vector of strings.
///
/// # Arguments
///
/// * `metadata` - A reference to the [`MetadataMap`] holding the metadata.
///
/// # Returns
///
/// A vector of strings representing the keywords. Returns an empty vector if no keywords are found.
///
/// # Example
///
/// ```
/// use metadata_gen::{extract_keywords, MetadataMap};
///
/// let mut metadata = MetadataMap::new();
/// metadata.insert("keywords".to_string(), "rust, metadata, parsing".to_string());
///
/// let keywords = extract_keywords(&metadata);
/// assert_eq!(keywords, vec!["rust", "metadata", "parsing"]);
/// ```
pub fn extract_keywords(metadata: &MetadataMap) -> Vec<String> {
    metadata
        .get("keywords")
        .map(|k| k.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}

#[cfg(all(test, feature = "std", feature = "yaml"))]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_extract_and_prepare_metadata() {
        let content = r#"---
title: Test Page
description: A test page for metadata extraction
keywords: test, metadata, extraction
---
# Test Content
This is a test file for metadata extraction."#;

        let result = extract_and_prepare_metadata(content);
        assert!(result.is_ok());

        let (metadata, keywords, meta_tags) = result.unwrap();
        assert_eq!(
            metadata.get("title"),
            Some(&"Test Page".to_string())
        );
        assert_eq!(
            metadata.get("description"),
            Some(&"A test page for metadata extraction".to_string())
        );
        assert_eq!(keywords, vec!["test", "metadata", "extraction"]);
        assert!(!meta_tags.primary.is_empty());
    }

    #[test]
    fn test_extract_keywords() {
        let mut metadata = MetadataMap::new();
        metadata.insert(
            "keywords".to_string(),
            "rust, programming, metadata".to_string(),
        );

        let keywords = extract_keywords(&metadata);
        assert_eq!(keywords, vec!["rust", "programming", "metadata"]);
    }

    #[test]
    fn test_extract_keywords_empty() {
        let metadata = MetadataMap::new();
        let keywords = extract_keywords(&metadata);
        assert!(keywords.is_empty());
    }
}
