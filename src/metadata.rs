//! Metadata extraction and processing module.
//!
//! This module provides functionality for extracting metadata from various formats
//! (YAML, TOML, JSON) and processing it into a standardized structure.

use crate::error::MetadataError;
use crate::MetadataMap;
use alloc::string::{String, ToString};

mod date;
mod front_matter;
mod process;
mod scan;

#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
use date::standardize_date;
use front_matter::parse_block;
#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
use process::{generate_derived_fields, generate_slug};
pub use process::{
    process_metadata, process_metadata_with, DateOrder, ProcessOptions,
};
use scan::Scan;

/// Represents metadata for a page or content item.
///
/// # Example
///
/// ```
/// use metadata_gen::Metadata;
/// use std::collections::HashMap;
///
/// let mut data = HashMap::new();
/// data.insert("title".to_string(), "My Page".to_string());
/// let metadata = Metadata::new(data);
/// assert_eq!(metadata.get("title"), Some(&"My Page".to_string()));
/// ```
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    /// The underlying key-value store for metadata fields.
    inner: MetadataMap,
}

impl Metadata {
    /// Creates a new `Metadata` instance with the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - A [`MetadataMap`] containing the metadata key-value pairs.
    ///
    /// # Returns
    ///
    /// A new `Metadata` instance.
    pub fn new(data: MetadataMap) -> Self {
        Metadata { inner: data }
    }

    /// Retrieves the value associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key to look up.
    ///
    /// # Returns
    ///
    /// An `Option<&String>` containing the value if the key exists, or `None` otherwise.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    /// Inserts a key-value pair into the metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert.
    /// * `value` - The value to associate with the key.
    ///
    /// # Returns
    ///
    /// The old value associated with the key, if it existed.
    pub fn insert(
        &mut self,
        key: String,
        value: String,
    ) -> Option<String> {
        self.inner.insert(key, value)
    }

    /// Checks if the metadata contains the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key to check for.
    ///
    /// # Returns
    ///
    /// `true` if the key exists, `false` otherwise.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Consumes the `Metadata` instance and returns the inner map.
    ///
    /// # Returns
    ///
    /// The inner [`MetadataMap`] containing all metadata key-value pairs.
    pub fn into_inner(self) -> MetadataMap {
        self.inner
    }
}

/// Extracts metadata from the content string.
///
/// This function attempts to extract metadata from YAML, TOML, or JSON formats.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to extract metadata from.
///
/// # Returns
///
/// A `Result` containing the extracted `Metadata` if successful, or a `MetadataError` if extraction fails.
///
/// # Errors
///
/// Returns a `MetadataError::ExtractionError` if no valid front matter is found.
pub fn extract_metadata(
    content: &str,
) -> Result<Metadata, MetadataError> {
    extract_metadata_with_limits(content, &ParseLimits::default())
}

/// Resource limits applied while a front-matter block is parsed.
///
/// Front matter is untrusted input in any pipeline that ingests
/// contributed documents, so every parse runs under a budget. The
/// defaults are the YAML parser's strict preset narrowed to what a
/// document header plausibly needs; raise them with the builder methods
/// for manifests that are genuinely large.
///
/// # Example
///
/// ```
/// use metadata_gen::{extract_metadata_with_limits, ParseLimits};
///
/// let limits = ParseLimits::default().max_block_bytes(4096);
/// let err = extract_metadata_with_limits(
///     &format!("---\ntitle: {}\n---\n", "x".repeat(5000)),
///     &limits,
/// )
/// .unwrap_err();
/// assert!(err.to_string().contains("exceeds"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseLimits {
    /// Largest front-matter block, in bytes, that will be handed to a parser.
    pub max_block_bytes: usize,
    /// Deepest nesting of mappings and sequences the YAML parser accepts.
    pub max_depth: usize,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_block_bytes: 1024 * 1024,
            max_depth: 64,
        }
    }
}

impl ParseLimits {
    /// Sets the largest block size, in bytes, a parser will be given.
    #[must_use]
    pub const fn max_block_bytes(mut self, bytes: usize) -> Self {
        self.max_block_bytes = bytes;
        self
    }

    /// Sets the deepest nesting the YAML parser accepts.
    #[must_use]
    pub const fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
}

/// Extracts metadata like [`extract_metadata`], under explicit [`ParseLimits`].
///
/// # Errors
///
/// Returns [`MetadataError::ExtractionError`] when no front matter is
/// found, the block exceeds `limits.max_block_bytes`, or an opening fence
/// has no closing fence (the message carries the byte offset);
/// [`MetadataError::Parse`] when a parser rejects the block; and
/// [`MetadataError::UnsupportedFormatError`] when the block's format was
/// not compiled in.
///
/// # Example
///
/// ```
/// use metadata_gen::{extract_metadata_with_limits, ParseLimits};
///
/// let doc = "---\ntitle: Limits\n---\n";
/// let metadata = extract_metadata_with_limits(doc, &ParseLimits::default()).unwrap();
/// assert_eq!(metadata.get("title").unwrap(), "Limits");
/// ```
pub fn extract_metadata_with_limits(
    content: &str,
    limits: &ParseLimits,
) -> Result<Metadata, MetadataError> {
    match scan::scan(content) {
        Scan::Found { format, raw, .. } => parse_block(format, raw, limits),
        Scan::Unterminated { format, offset } => {
            Err(MetadataError::ExtractionError {
                message: alloc::format!(
                    "{format} front matter opened at byte {offset} has no closing fence."
                ),
            })
        }
        Scan::Unsupported(format) => {
            Err(MetadataError::UnsupportedFormatError(format.to_string()))
        }
        Scan::None => Err(MetadataError::ExtractionError {
            message: "No valid front matter found.".to_string(),
        }),
    }
}

/// Which front-matter shape a document carries, and where its body starts.
///
/// Returned by [`detect_front_matter`]; the byte offset is the first byte
/// after the closing delimiter, so `&content[body_start..]` is the body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FrontMatterFormat {
    /// `---` fenced YAML.
    Yaml,
    /// `+++` fenced TOML.
    Toml,
    /// A JSON object at the top of the document.
    Json,
}

impl FrontMatterFormat {
    /// The format's lower-case name: `yaml`, `toml` or `json`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Json => "json",
        }
    }
}

impl core::fmt::Display for FrontMatterFormat {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Locates the front-matter block without parsing it.
///
/// Returns the format, the raw block, and the byte offset at which the
/// document body begins. Detection order is YAML, TOML, JSON, the same
/// order [`extract_metadata`] uses; `None` means no shape matched.
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::{detect_front_matter, FrontMatterFormat};
///
/// let doc = "---\ntitle: T\n---\nBody";
/// let (format, raw, body_start) = detect_front_matter(doc).unwrap();
/// assert_eq!(format, FrontMatterFormat::Yaml);
/// assert_eq!(raw, "title: T");
/// assert_eq!(&doc[body_start..], "Body");
/// ```
pub fn detect_front_matter(
    content: &str,
) -> Option<(FrontMatterFormat, &str, usize)> {
    match scan::scan(content) {
        Scan::Found {
            format,
            raw,
            body_offset,
        } => Some((format, raw, body_offset)),
        _ => None,
    }
}

/// Extracts metadata and returns the document body alongside it.
///
/// The body is the text after the closing delimiter, with one leading
/// newline removed so a `---` fence on its own line does not leave an
/// empty first line. [`extract_metadata`] is this without the body.
///
/// # Errors
///
/// The same errors as [`extract_metadata`].
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::extract_metadata_with_body;
///
/// let doc = "---\ntitle: T\n---\n# Heading\n\nText";
/// let (meta, body) = extract_metadata_with_body(doc).unwrap();
/// assert_eq!(meta.get("title").map(String::as_str), Some("T"));
/// assert_eq!(body, "# Heading\n\nText");
/// ```
pub fn extract_metadata_with_body(
    content: &str,
) -> Result<(Metadata, &str), MetadataError> {
    let metadata = extract_metadata(content)?;
    let body_start = detect_front_matter(content)
        .map_or(content.len(), |(_, _, start)| start);
    let body = content[body_start..]
        .strip_prefix("\r\n")
        .or_else(|| content[body_start..].strip_prefix('\n'))
        .unwrap_or(&content[body_start..]);
    Ok((metadata, body))
}

/// Deserialises the front matter into a typed value.
///
/// Where [`extract_metadata`] flattens everything to strings, this hands
/// the raw block to the format's own `serde` deserialiser, so integers
/// stay integers, sequences stay sequences and nested tables become
/// nested structs. The document body is ignored.
///
/// # Errors
///
/// [`MetadataError::ExtractionError`] when no front matter is found;
/// the format's parse error (`YamlError`, `TomlError`, `JsonError`) when
/// the block does not deserialise into `T`.
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::extract_typed;
///
/// #[derive(serde::Deserialize)]
/// struct Front { title: String, tags: Vec<String>, draft: bool }
///
/// let doc = "---\ntitle: T\ntags: [a, b]\ndraft: false\n---\nBody";
/// let front: Front = extract_typed(doc).unwrap();
/// assert_eq!(front.tags, ["a", "b"]);
/// assert!(!front.draft);
/// ```
pub fn extract_typed<T: serde::de::DeserializeOwned + 'static>(
    content: &str,
) -> Result<T, MetadataError> {
    let (format, raw, _) =
        detect_front_matter(content).ok_or_else(|| {
            MetadataError::ExtractionError {
                message: "No valid front matter found.".to_string(),
            }
        })?;
    front_matter::deserialize_block::<T>(
        format,
        raw,
        &ParseLimits::default(),
    )
}

/// Deserialises the front matter into `T` while borrowing from `content`.
///
/// The zero-copy counterpart of [`extract_typed`]: a field typed
/// `&'a str` (or `Cow<'a, str>`) points into `content` instead of being
/// copied, for every format whose text already holds the value verbatim.
/// Values that need unescaping (a YAML `"line\nbreak"`, a JSON `\u00e9`)
/// are materialised by the parser as owned strings, which is why a
/// `Cow<'a, str>` field is the general choice.
///
/// # Errors
///
/// As [`extract_typed`].
///
/// # Example
///
/// ```
/// use metadata_gen::extract_typed_borrowed;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Post<'a> {
///     title: &'a str,
///     tags: Vec<&'a str>,
/// }
///
/// let doc = "{\"title\": \"Borrowed\", \"tags\": [\"a\", \"b\"]}\nbody";
/// let post: Post<'_> = extract_typed_borrowed(doc).unwrap();
/// assert_eq!(post.title, "Borrowed");
/// assert_eq!(post.tags, ["a", "b"]);
/// ```
pub fn extract_typed_borrowed<'a, T: serde::Deserialize<'a>>(
    content: &'a str,
) -> Result<T, MetadataError> {
    let (format, raw, _) =
        detect_front_matter(content).ok_or_else(|| {
            MetadataError::ExtractionError {
                message: "No valid front matter found.".to_string(),
            }
        })?;
    front_matter::deserialize_borrowed::<T>(
        format,
        raw,
        &ParseLimits::default(),
    )
}

#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
mod tests;

#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
mod coverage_tests;

#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
mod typed_api_tests;
