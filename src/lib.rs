// src/lib.rs

#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/metadata-gen/images/favicon.ico",
    html_logo_url = "https://kura.pro/metadata-gen/images/logos/metadata-gen.svg",
    html_root_url = "https://docs.rs/metadata-gen"
)]
#![crate_name = "metadata_gen"]
#![crate_type = "lib"]

use std::collections::HashMap;

/// The `error` module contains error types for metadata processing.
pub mod error;
/// The `metadata` module contains functions for extracting and processing metadata.
pub mod metadata;
/// The `metatags` module contains functions for generating meta tags.
pub mod metatags;
/// The `utils` module contains utility functions for metadata processing.
pub mod utils;

pub use error::MetadataError;
pub use metadata::{extract_metadata, process_metadata, Metadata};
pub use metatags::{generate_metatags, MetaTagGroups};
pub use utils::{async_extract_metadata_from_file, escape_html};

/// Type alias for a map of metadata key-value pairs.
pub type MetadataMap = HashMap<String, String>;
/// Type alias for a list of keywords.
pub type Keywords = Vec<String>;
/// Type alias for the result of metadata extraction and processing.
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
/// * `HashMap<String, String>`: Extracted metadata
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
    // Ensure the front matter format is correct
    if !content.contains(":") {
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
/// * `metadata` - A reference to a HashMap containing the metadata.
///
/// # Returns
///
/// A vector of strings representing the keywords.
pub fn extract_keywords(
    metadata: &HashMap<String, String>,
) -> Vec<String> {
    metadata
        .get("keywords")
        .map(|k| k.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut metadata = HashMap::new();
        metadata.insert(
            "keywords".to_string(),
            "rust, programming, metadata".to_string(),
        );

        let keywords = extract_keywords(&metadata);
        assert_eq!(keywords, vec!["rust", "programming", "metadata"]);
    }

    #[test]
    fn test_extract_keywords_empty() {
        let metadata = HashMap::new();
        let keywords = extract_keywords(&metadata);
        assert!(keywords.is_empty());
    }
}
