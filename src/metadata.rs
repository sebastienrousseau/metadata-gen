//! Metadata extraction and processing module.
//!
//! This module provides functionality for extracting metadata from various formats
//! (YAML, TOML, JSON) and processing it into a standardized structure.

use crate::error::MetadataError;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::LazyLock;

mod date;
mod front_matter;

use date::standardize_date;
use front_matter::{
    collapse_multiline_quoted_scalars, extract_json_metadata,
    extract_toml_metadata, extract_yaml_metadata,
};

// One-time compiled front-matter delimiters. Calling `Regex::new` on every
// `extract_metadata` invocation cost ~30–50 µs per call plus an allocation
// per regex — entirely unnecessary at SSG scale. The patterns are static,
// so compile them once per process. Issue #25.
//
// The `expect` is unreachable in any reachable code path: the patterns are
// compile-time literals and have been validated by the test suite for
// every release. If a future edit introduces a malformed pattern, the
// startup-time panic is preferable to silently returning `None` from
// every parse call.
static YAML_FRONT_MATTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^\s*---\s*\n(.*?)\n\s*---\s*")
        .expect("YAML front-matter regex is statically valid")
});
static TOML_FRONT_MATTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^\s*\+\+\+\s*(.*?)\s*\+\+\+")
        .expect("TOML front-matter regex is statically valid")
});

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
    inner: HashMap<String, String>,
}

impl Metadata {
    /// Creates a new `Metadata` instance with the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - A `HashMap` containing the metadata key-value pairs.
    ///
    /// # Returns
    ///
    /// A new `Metadata` instance.
    pub fn new(data: HashMap<String, String>) -> Self {
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

    /// Consumes the `Metadata` instance and returns the inner `HashMap`.
    ///
    /// # Returns
    ///
    /// The inner `HashMap<String, String>` containing all metadata key-value pairs.
    pub fn into_inner(self) -> HashMap<String, String> {
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
    // YAML returns Option<Result<...>>: `Some(Ok)` = parsed OK,
    // `Some(Err)` = fence matched but YAML failed (surface that
    // specific error), `None` = no YAML fence found, fall through.
    // Issue #20.
    if let Some(yaml_result) = extract_yaml_metadata(content) {
        return yaml_result;
    }
    if let Some(toml) = extract_toml_metadata(content) {
        return Ok(toml);
    }
    if let Some(json_result) = extract_json_metadata(content) {
        return json_result;
    }
    Err(MetadataError::ExtractionError {
        message: "No valid front matter found.".to_string(),
    })
}

/// Which front-matter shape a document carries, and where its body starts.
///
/// Returned by [`detect_front_matter`]; the byte offset is the first byte
/// after the closing delimiter, so `&content[body_start..]` is the body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrontMatterFormat {
    /// `---` fenced YAML.
    Yaml,
    /// `+++` fenced TOML.
    Toml,
    /// A JSON object at the top of the document.
    Json,
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
    if let Some(caps) = YAML_FRONT_MATTER.captures(content) {
        let whole = caps.get(0)?;
        let raw = caps.get(1)?.as_str().trim();
        return Some((FrontMatterFormat::Yaml, raw, whole.end()));
    }
    if let Some(caps) = TOML_FRONT_MATTER.captures(content) {
        let whole = caps.get(0)?;
        let raw = caps.get(1)?.as_str().trim();
        return Some((FrontMatterFormat::Toml, raw, whole.end()));
    }
    let trimmed = content.trim_start();
    if trimmed.starts_with('{') {
        let lead = content.len() - trimmed.len();
        let mut stream = serde_json::Deserializer::from_str(trimmed)
            .into_iter::<serde_json::Map<String, JsonValue>>(
        );
        // A syntax error still identifies the shape; the caller's parse
        // reports it. The offset then covers the text the parser consumed.
        let _ = stream.next()?;
        let end = lead + stream.byte_offset();
        return Some((
            FrontMatterFormat::Json,
            &content[lead..end],
            end,
        ));
    }
    None
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
    match format {
        FrontMatterFormat::Yaml => {
            let collapsed = collapse_multiline_quoted_scalars(raw);
            noyalib::from_str::<T>(&collapsed)
                .map_err(MetadataError::from)
        }
        FrontMatterFormat::Toml => {
            toml::from_str::<T>(raw).map_err(MetadataError::from)
        }
        FrontMatterFormat::Json => {
            serde_json::from_str::<T>(raw).map_err(MetadataError::from)
        }
    }
}

/// Options for [`process_metadata_with`].
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::{process_metadata_with, Metadata, ProcessOptions};
/// use std::collections::HashMap;
///
/// let opts = ProcessOptions::default().required_fields(["title"]);
/// let mut m = HashMap::new();
/// m.insert("title".to_string(), "Hello".to_string());
/// let out = process_metadata_with(&Metadata::new(m), &opts).unwrap();
/// assert_eq!(out.get("slug").map(String::as_str), Some("hello"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProcessOptions {
    /// Keys that must be present after processing. Default: `title`, `date`.
    pub required_fields: Vec<String>,
    /// Derive `slug` from `title` when absent. Default: `true`.
    pub derive_slug: bool,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            required_fields: vec![
                "title".to_string(),
                "date".to_string(),
            ],
            derive_slug: true,
        }
    }
}

impl ProcessOptions {
    /// Replaces the required-field list.
    #[must_use]
    pub fn required_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_fields =
            fields.into_iter().map(Into::into).collect();
        self
    }

    /// Turns slug derivation on or off.
    #[must_use]
    pub const fn derive_slug(mut self, on: bool) -> Self {
        self.derive_slug = on;
        self
    }
}

/// [`process_metadata`] with caller-chosen required fields and derivations.
///
/// # Errors
///
/// [`MetadataError::DateParseError`] for an unparseable `date`;
/// [`MetadataError::MissingFieldError`] naming the first required field
/// that is absent.
pub fn process_metadata_with(
    metadata: &Metadata,
    options: &ProcessOptions,
) -> Result<Metadata, MetadataError> {
    let mut processed = metadata.clone();
    if let Some(date) = processed.get("date").cloned() {
        let standardized_date = standardize_date(&date)?;
        processed.insert("date".to_string(), standardized_date);
    }
    for field in &options.required_fields {
        if !processed.contains_key(field) {
            return Err(MetadataError::MissingFieldError(
                field.clone(),
            ));
        }
    }
    if options.derive_slug {
        generate_derived_fields(&mut processed);
    }
    Ok(processed)
}

/// Processes the extracted metadata.
///
/// This function standardizes dates, ensures required fields are present, and generates derived fields.
///
/// # Arguments
///
/// * `metadata` - A reference to the `Metadata` instance to process.
///
/// # Returns
///
/// A `Result` containing the processed `Metadata` if successful, or a `MetadataError` if processing fails.
///
/// # Errors
///
/// Returns a `MetadataError` if date standardization fails or if required fields are missing.
pub fn process_metadata(
    metadata: &Metadata,
) -> Result<Metadata, MetadataError> {
    let mut processed = metadata.clone();

    // Convert dates to a standard format
    if let Some(date) = processed.get("date").cloned() {
        let standardized_date = standardize_date(&date)?;
        processed.insert("date".to_string(), standardized_date);
    }

    // Ensure required fields are present
    ensure_required_fields(&processed)?;

    // Generate derived fields
    generate_derived_fields(&mut processed);

    Ok(processed)
}

/// Ensures that all required fields are present in the metadata.
///
/// # Arguments
///
/// * `metadata` - A reference to the `Metadata` instance to check.
///
/// # Returns
///
/// A `Result<()>` if all required fields are present, or a `MetadataError` if any are missing.
///
/// # Errors
///
/// Returns a `MetadataError::MissingFieldError` if any required field is missing.
fn ensure_required_fields(
    metadata: &Metadata,
) -> Result<(), MetadataError> {
    let required_fields = ["title", "date"];

    for &field in &required_fields {
        if !metadata.contains_key(field) {
            return Err(MetadataError::MissingFieldError(
                field.to_string(),
            ));
        }
    }

    Ok(())
}

/// Generates derived fields for the metadata.
///
/// Currently, this function generates a URL slug from the title if not already present.
///
/// # Arguments
///
/// * `metadata` - A mutable reference to the `Metadata` instance to update.
fn generate_derived_fields(metadata: &mut Metadata) {
    if !metadata.contains_key("slug") {
        if let Some(title) = metadata.get("title") {
            let slug = generate_slug(title);
            metadata.insert("slug".to_string(), slug);
        }
    }
}

/// Generates a URL slug from the given title.
///
/// # Arguments
///
/// * `title` - A string slice containing the title to convert to a slug.
///
/// # Returns
///
/// A `String` containing the generated slug.
fn generate_slug(title: &str) -> String {
    title.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod coverage_tests;

#[cfg(test)]
mod typed_api_tests;
