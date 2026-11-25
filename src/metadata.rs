//! Metadata extraction and processing module.
//!
//! This module provides functionality for extracting metadata from various formats
//! (YAML, TOML, JSON) and processing it into a standardized structure.

use crate::error::MetadataError;
use alloc::string::ToString;

pub mod builder;
#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
mod cow;
mod date;
#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "yaml",
        feature = "toml",
        feature = "json"
    )))
)]
pub mod events;
mod front_matter;
mod iso_date;
mod model;
mod process;
mod scan;

#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
pub use cow::{extract_metadata_cow, CowMetadata};
#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
use date::standardize_date;
use front_matter::parse_block;
pub use model::Metadata;
#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
use process::{generate_derived_fields, generate_slug};
pub use process::{
    process_metadata, process_metadata_with, DateOrder,
    MetadataProcessor, ProcessOptions,
};
use scan::Scan;

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
///
/// # Example
///
/// ```
/// use metadata_gen::extract_metadata;
///
/// let meta = extract_metadata("---\ntitle: Hello\nauthor:\n  name: Ada\n---\nbody").unwrap();
/// assert_eq!(meta.get("title").unwrap(), "Hello");
/// assert_eq!(meta.get("author.name").unwrap(), "Ada");
/// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::{extract_metadata_with_limits, ParseLimits};
    ///
    /// let tight = ParseLimits::default().max_block_bytes(8);
    /// assert!(extract_metadata_with_limits("---\ntitle: far too long\n---\n", &tight).is_err());
    /// ```
    #[must_use]
    pub const fn max_block_bytes(mut self, bytes: usize) -> Self {
        self.max_block_bytes = bytes;
        self
    }

    /// Sets the deepest nesting the YAML parser accepts.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::ParseLimits;
    ///
    /// let limits = ParseLimits::default().max_depth(8);
    /// assert_eq!(limits.max_depth, 8);
    /// ```
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
    extract_with_offset(content, limits).map(|(metadata, _)| metadata)
}

/// One scan, one parse: the metadata and the byte offset where the body
/// starts.
fn extract_with_offset(
    content: &str,
    limits: &ParseLimits,
) -> Result<(Metadata, usize), MetadataError> {
    let (format, raw, body_offset) = find_block(content)?;
    parse_block(format, raw, limits)
        .map(|metadata| (metadata, body_offset))
}

/// The block's format, its raw text and the body offset, or the error
/// every entry point reports when there is no usable block.
fn find_block(
    content: &str,
) -> Result<(FrontMatterFormat, &str, usize), MetadataError> {
    match scan::scan(content) {
        Scan::Found {
            format,
            raw,
            body_offset,
        } => Ok((format, raw, body_offset)),
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
///
/// # Example
///
/// ```
/// use metadata_gen::{detect_front_matter, FrontMatterFormat};
///
/// let found = detect_front_matter("+++\ntitle = \"T\"\n+++\nbody").unwrap();
/// assert_eq!(found.0, FrontMatterFormat::Toml);
/// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::FrontMatterFormat;
    ///
    /// assert_eq!(FrontMatterFormat::Yaml.as_str(), "yaml");
    /// ```
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
    let (metadata, body_start) =
        extract_with_offset(content, &ParseLimits::default())?;
    let rest = &content[body_start..];
    let body = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .unwrap_or(rest);
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
/// [`MetadataError::ExtractionError`] when no front matter is found or
/// the block exceeds the default [`ParseLimits`];
/// [`MetadataError::Parse`] (with `format` set to the block's format and
/// the parser's message, which names the offending field) when the block
/// does not parse or does not deserialise into `T`; and
/// [`MetadataError::UnsupportedFormatError`] when the block's format was
/// not compiled in.
///
/// # Example
///
/// The same struct reads YAML, TOML and JSON front matter; serde
/// attributes such as `default` and `rename` are honoured.
///
/// ```
/// use metadata_gen::metadata::extract_typed;
///
/// #[derive(serde::Deserialize)]
/// struct Front {
///     title: String,
///     tags: Vec<String>,
///     #[serde(default)]
///     draft: bool,
///     #[serde(rename = "sort-order", default)]
///     sort_order: u32,
/// }
///
/// let yaml = "---\ntitle: T\ntags: [a, b]\nsort-order: 2\n---\nBody";
/// let toml = "+++\ntitle = \"T\"\ntags = [\"a\", \"b\"]\n+++\nBody";
/// let json = "{\"title\": \"T\", \"tags\": [\"a\", \"b\"], \"draft\": true}\nBody";
///
/// let from_yaml: Front = extract_typed(yaml).unwrap();
/// assert_eq!(from_yaml.tags, ["a", "b"]);
/// assert_eq!(from_yaml.sort_order, 2);
/// assert!(!from_yaml.draft);
///
/// let from_toml: Front = extract_typed(toml).unwrap();
/// assert_eq!(from_toml.title, "T");
///
/// let from_json: Front = extract_typed(json).unwrap();
/// assert!(from_json.draft);
/// ```
///
/// A value of the wrong type is a [`MetadataError::Parse`]:
///
/// ```
/// use metadata_gen::{extract_typed, FrontMatterFormat, MetadataError};
///
/// #[derive(Debug, serde::Deserialize)]
/// struct Front { tags: Vec<String> }
///
/// let err = extract_typed::<Front>("{\"tags\": [\"a\", 2]}\n").unwrap_err();
/// assert!(matches!(
///     err,
///     MetadataError::Parse { format: FrontMatterFormat::Json, .. }
/// ));
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
