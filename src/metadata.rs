//! Metadata extraction and processing module.
//!
//! This module provides functionality for extracting metadata from various formats
//! (YAML, TOML, JSON) and processing it into a standardized structure.

use crate::error::MetadataError;
use dtt::datetime::DateTime;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::LazyLock;
use toml::Value as TomlValue;

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

/// Extracts YAML metadata from the content.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to extract YAML metadata from.
///
/// # Returns
///
/// An `Option<Metadata>` containing the extracted metadata if successful, or `None` if extraction fails.
fn extract_yaml_metadata(
    content: &str,
) -> Option<Result<Metadata, MetadataError>> {
    let captures = YAML_FRONT_MATTER.captures(content)?;

    let yaml_str = captures.get(1)?.as_str().trim();

    // noyalib enforces YAML 1.2.2 §7.3.2 strictly: continuation lines
    // of a multi-line double-quoted scalar must be indented more than
    // the parent block. PyYAML / serde_yaml relax this. Real-world
    // frontmatter (esp. human-edited URLs that picked up an
    // accidental newline) routinely violates the strict rule.
    // Collapse the offending shape upstream of noyalib so consumers
    // don't need to re-implement this each. Issue #20.
    let collapsed = collapse_multiline_quoted_scalars(yaml_str);

    match noyalib::from_str::<noyalib::Value>(&collapsed) {
        Ok(v) => {
            let metadata: HashMap<String, String> = flatten_yaml(&v);
            Some(Ok(Metadata::new(metadata)))
        }
        Err(e) => Some(Err(MetadataError::ExtractionError {
            message: format!("YAML parse error in frontmatter: {e}"),
        })),
    }
}

/// Collapses multi-line double-quoted YAML scalars onto a single line.
///
/// noyalib correctly enforces YAML 1.2.2 §7.3.2 (continuation must be
/// indented more than the parent block). Human-edited frontmatter
/// often violates this — e.g. a `url: "\n<value>"` shape where a
/// literal newline crept in after the opening quote. PyYAML and
/// serde_yaml fold those onto one line; this helper does the same so
/// noyalib never sees the offending shape.
///
/// The scan is line-based and deliberately simple: when a line ends
/// with `: "` (key + opening quote with nothing after), walk forward
/// joining subsequent lines until the closing `"` is found. Comments
/// and other quote styles are not transformed.
fn collapse_multiline_quoted_scalars(block: &str) -> String {
    let mut out = String::with_capacity(block.len());
    let lines: Vec<&str> = block.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(eq_pos) = line.find(": \"") {
            let after_quote = &line[eq_pos + 3..];
            if after_quote.trim().is_empty() {
                let mut joined = String::from(&line[..eq_pos + 3]);
                let mut closed = false;
                i += 1;
                while i < lines.len() {
                    let next = lines[i];
                    if let Some(close) = next.find('"') {
                        joined.push_str(next[..close].trim_start());
                        joined.push_str(&next[close..]);
                        out.push_str(&joined);
                        out.push('\n');
                        i += 1;
                        closed = true;
                        break;
                    }
                    joined.push_str(next.trim_start());
                    joined.push(' ');
                    i += 1;
                }
                if !closed {
                    // Pathological — no closing quote. Emit what we
                    // have so the downstream parser sees the same
                    // broken content rather than silently swallowing.
                    out.push_str(joined.trim_end());
                    out.push('\n');
                }
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
        i += 1;
    }
    out
}

/// Flattens a nested YAML value into a flat key-value map.
///
/// Nested keys are joined with `.` (e.g., `author.name`).
/// Sequences are rendered as comma-separated lists wrapped in brackets.
fn flatten_yaml(value: &noyalib::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    flatten_yaml_recursive(value, String::new(), &mut map);
    map
}

/// Recursively walks a YAML value tree, inserting leaf values into the map
/// with dot-separated keys for nested mappings.
fn flatten_yaml_recursive(
    value: &noyalib::Value,
    prefix: String,
    map: &mut HashMap<String, String>,
) {
    match value {
        noyalib::Value::Mapping(m) => {
            for (k, v) in m {
                // In noyalib, mapping keys are `String`, so `k.as_str()`
                // already yields `&str` directly.
                let new_prefix = if prefix.is_empty() {
                    k.as_str().to_string()
                } else {
                    format!("{}.{}", prefix, k.as_str())
                };
                flatten_yaml_recursive(v, new_prefix, map);
            }
        }
        noyalib::Value::Sequence(seq) => {
            let inline_list = seq
                .iter()
                .map(|item| match item.as_str() {
                    Some(s) => s.to_string(),
                    None => item.to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ");
            map.insert(prefix, format!("[{}]", inline_list));
        }
        _ => {
            // `as_str()` returns `Some` only for `Value::String`; for
            // scalars (numbers, bools, dates rendered as numbers, etc.)
            // we fall back to the `Display` representation.
            let leaf = match value.as_str() {
                Some(s) => s.to_string(),
                None => value.to_string(),
            };
            map.insert(prefix, leaf);
        }
    }
}

/// Extracts TOML metadata from the content.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to extract TOML metadata from.
///
/// # Returns
///
/// An `Option<Metadata>` containing the extracted metadata if successful, or `None` if extraction fails.
fn extract_toml_metadata(content: &str) -> Option<Metadata> {
    let captures = TOML_FRONT_MATTER.captures(content)?;
    let toml_str = captures.get(1)?.as_str().trim();

    let toml_value: TomlValue = toml::from_str(toml_str).ok()?;

    let mut metadata = HashMap::new();
    flatten_toml(&toml_value, &mut metadata, String::new());

    Some(Metadata::new(metadata))
}

/// Recursively flattens a TOML value tree into a flat key-value map.
///
/// Nested keys are joined with `.` (e.g., `author.name`).
/// Arrays are rendered as comma-separated lists wrapped in brackets.
fn flatten_toml(
    value: &TomlValue,
    map: &mut HashMap<String, String>,
    prefix: String,
) {
    match value {
        TomlValue::Table(table) => {
            for (k, v) in table {
                let new_prefix = if prefix.is_empty() {
                    k.to_string()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_toml(v, map, new_prefix);
            }
        }
        TomlValue::Array(arr) => {
            let inline_list = arr
                .iter()
                .map(|v| {
                    // Remove double quotes for string elements
                    match v {
                        TomlValue::String(s) => s.clone(),
                        _ => v.to_string(),
                    }
                })
                .collect::<Vec<String>>()
                .join(", ");
            map.insert(prefix, format!("[{}]", inline_list));
        }
        TomlValue::String(s) => {
            map.insert(prefix, s.clone());
        }
        TomlValue::Datetime(dt) => {
            map.insert(prefix, dt.to_string());
        }
        _ => {
            map.insert(prefix, value.to_string());
        }
    }
}

/// Extracts JSON front-matter from the start of `content`.
///
/// Returns `Some(Ok(_))` when a balanced JSON object is found and parsed,
/// `Some(Err(_))` when an opening `{` appears but the JSON is malformed
/// (so the caller can surface a useful error instead of the misleading
/// "no front-matter" fallback), and `None` only when no opening `{`
/// appears at the start of the content.
///
/// Nested objects and arrays of objects are preserved by flattening them
/// with dot-separated keys (e.g. `author.name`, matching the YAML/TOML
/// shape). Issue #26.
fn extract_json_metadata(
    content: &str,
) -> Option<Result<Metadata, MetadataError>> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with('{') {
        return None;
    }

    // `Deserializer::into_iter` consumes one balanced JSON value at a
    // time. We take the first one — that's the front-matter — and let
    // anything after it be the document body. This replaces the old
    // non-greedy regex that silently truncated nested objects at the
    // first `}` it saw.
    let mut stream =
        serde_json::Deserializer::from_str(trimmed).into_iter::<JsonValue>();
    let first = stream.next()?; // None only if input is empty after `{`.

    let value = match first {
        Ok(v) => v,
        Err(e) => {
            return Some(Err(MetadataError::ExtractionError {
                message: format!(
                    "JSON parse error in frontmatter: {e}"
                ),
            }))
        }
    };

    let object = match value.as_object() {
        Some(obj) => obj,
        None => {
            return Some(Err(MetadataError::ExtractionError {
                message: "JSON frontmatter must be an object at the \
                          document root"
                    .to_string(),
            }))
        }
    };

    let mut metadata: HashMap<String, String> = HashMap::new();
    for (k, v) in object {
        flatten_json(v, &mut metadata, k.clone());
    }
    Some(Ok(Metadata::new(metadata)))
}

/// Recursively flattens a JSON value tree into a flat key-value map.
///
/// Mirrors `flatten_toml` / `flatten_yaml`: nested objects use
/// dot-separated keys; arrays render as comma-separated lists wrapped in
/// brackets. Strings are stored as-is; numbers, booleans, and `null`
/// fall back to their JSON `Display` form.
fn flatten_json(
    value: &JsonValue,
    map: &mut HashMap<String, String>,
    prefix: String,
) {
    match value {
        JsonValue::Object(obj) => {
            for (k, v) in obj {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(v, map, new_prefix);
            }
        }
        JsonValue::Array(arr) => {
            // Arrays of scalars render as `[a, b, c]`. Arrays of objects
            // render the same — callers that need element-level access
            // should use the v0.0.6 typed-extraction API (issue #45).
            let inline = arr
                .iter()
                .map(|v| match v {
                    JsonValue::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ");
            map.insert(prefix, format!("[{}]", inline));
        }
        JsonValue::String(s) => {
            map.insert(prefix, s.clone());
        }
        JsonValue::Null => {
            map.insert(prefix, "null".to_string());
        }
        _ => {
            map.insert(prefix, value.to_string());
        }
    }
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

/// Standardizes the date format.
///
/// This function attempts to parse various date formats and convert them to the YYYY-MM-DD format.
///
/// # Arguments
///
/// * `date` - A string slice containing the date to standardize.
///
/// # Returns
///
/// A `Result` containing the standardized date string if successful, or a `MetadataError` if parsing fails.
///
/// # Errors
///
/// Returns a `MetadataError::DateParseError` if the date cannot be parsed or is invalid.
fn standardize_date(date: &str) -> Result<String, MetadataError> {
    // Handle edge cases with empty or too-short dates
    if date.trim().is_empty() {
        return Err(MetadataError::DateParseError(
            "Date string is empty.".to_string(),
        ));
    }

    if date.len() < 8 {
        return Err(MetadataError::DateParseError(
            "Date string is too short.".to_string(),
        ));
    }

    // Check if the date is in the DD/MM/YYYY format and reformat to YYYY-MM-DD
    let date = if date.contains('/') && date.len() == 10 {
        let parts: Vec<&str> = date.split('/').collect();
        if parts.len() == 3
            && parts[0].len() == 2
            && parts[1].len() == 2
            && parts[2].len() == 4
        {
            format!("{}-{}-{}", parts[2], parts[1], parts[0]) // Reformat to YYYY-MM-DD
        } else {
            return Err(MetadataError::DateParseError(
                "Invalid DD/MM/YYYY date format.".to_string(),
            ));
        }
    } else {
        date.to_string()
    };

    // Attempt to parse the date in different formats using DateTime methods
    let parsed_date = DateTime::parse(&date)
        .or_else(|_| {
            DateTime::parse_custom_format(&date, "[year]-[month]-[day]")
        })
        .or_else(|_| {
            DateTime::parse_custom_format(&date, "[month]/[day]/[year]")
        })
        .map_err(|e| {
            MetadataError::DateParseError(format!(
                "Failed to parse date: {}",
                e
            ))
        })?;

    // Format the date to the standardized YYYY-MM-DD format
    Ok(format!(
        "{:04}-{:02}-{:02}",
        parsed_date.year(),
        parsed_date.month() as u8,
        parsed_date.day()
    ))
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
mod tests {
    use super::*;
    use dtt::dtt_parse;

    #[test]
    fn test_standardize_date() {
        let test_cases = vec![
            ("2023-05-20T15:30:00Z", "2023-05-20"),
            ("2023-05-20", "2023-05-20"),
            ("20/05/2023", "2023-05-20"), // European format DD/MM/YYYY
        ];

        for (input, expected) in test_cases {
            let result = standardize_date(input);
            assert!(result.is_ok(), "Failed for input: {}", input);
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn test_standardize_date_errors() {
        assert!(standardize_date("").is_err());
        assert!(standardize_date("invalid").is_err());
        assert!(standardize_date("20/05/23").is_err()); // Invalid DD/MM/YY format
    }

    #[test]
    fn test_date_format() {
        let dt = dtt_parse!("2023-01-01T12:00:00+00:00").unwrap();
        let formatted = format!(
            "{:04}-{:02}-{:02}",
            dt.year(),
            dt.month() as u8,
            dt.day()
        );
        assert_eq!(formatted, "2023-01-01");
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
        assert_eq!(generate_slug("Test 123"), "test-123");
        assert_eq!(generate_slug("  Spaces  "), "--spaces--");
    }

    #[test]
    fn test_process_metadata() {
        let mut metadata = Metadata::new(HashMap::new());
        metadata.insert("title".to_string(), "Test Title".to_string());
        metadata.insert(
            "date".to_string(),
            "2023-05-20T15:30:00Z".to_string(),
        );

        let processed = process_metadata(&metadata).unwrap();
        assert_eq!(processed.get("title").unwrap(), "Test Title");
        assert_eq!(processed.get("date").unwrap(), "2023-05-20");
        assert_eq!(processed.get("slug").unwrap(), "test-title");
    }

    #[test]
    fn test_extract_metadata() {
        let yaml_content = r#"---
title: YAML Test
date: 2023-05-20
---
Content here"#;

        let toml_content = r#"+++
title = "TOML Test"
date = "2023-05-20"
+++
Content here"#;

        let json_content = r#"{
"title": "JSON Test",
"date": "2023-05-20"
}
Content here"#;

        let yaml_metadata = extract_metadata(yaml_content).unwrap();
        assert_eq!(yaml_metadata.get("title").unwrap(), "YAML Test");

        let toml_metadata = extract_metadata(toml_content).unwrap();
        assert_eq!(toml_metadata.get("title").unwrap(), "TOML Test");

        let json_metadata = extract_metadata(json_content).unwrap();
        assert_eq!(json_metadata.get("title").unwrap(), "JSON Test");
    }

    #[test]
    fn test_extract_metadata_failure() {
        let invalid_content = "This content has no metadata";
        assert!(extract_metadata(invalid_content).is_err());
    }

    #[test]
    fn test_ensure_required_fields() {
        let mut metadata = Metadata::new(HashMap::new());
        metadata.insert("title".to_string(), "Test".to_string());
        metadata.insert("date".to_string(), "2023-05-20".to_string());

        assert!(ensure_required_fields(&metadata).is_ok());

        let mut incomplete_metadata = Metadata::new(HashMap::new());
        incomplete_metadata
            .insert("title".to_string(), "Test".to_string());

        assert!(ensure_required_fields(&incomplete_metadata).is_err());
    }

    #[test]
    fn test_generate_derived_fields() {
        let mut metadata = Metadata::new(HashMap::new());
        metadata.insert("title".to_string(), "Test Title".to_string());

        generate_derived_fields(&mut metadata);

        assert_eq!(metadata.get("slug").unwrap(), "test-title");
    }

    #[test]
    fn test_metadata_methods() {
        let mut metadata = Metadata::new(HashMap::new());
        metadata.insert("key".to_string(), "value".to_string());

        assert_eq!(metadata.get("key"), Some(&"value".to_string()));
        assert!(metadata.contains_key("key"));
        assert!(!metadata.contains_key("nonexistent"));

        let old_value =
            metadata.insert("key".to_string(), "new_value".to_string());
        assert_eq!(old_value, Some("value".to_string()));
        assert_eq!(metadata.get("key"), Some(&"new_value".to_string()));

        let inner = metadata.into_inner();
        assert_eq!(inner.get("key"), Some(&"new_value".to_string()));
    }

    #[test]
    fn test_process_metadata_with_invalid_date() {
        let mut metadata = Metadata::new(HashMap::new());
        metadata.insert("title".to_string(), "Test Title".to_string());
        metadata.insert("date".to_string(), "invalid_date".to_string());

        assert!(process_metadata(&metadata).is_err());
    }

    #[test]
    fn test_extract_yaml_metadata_with_complex_structure() {
        let yaml_content = r#"---
title: Complex YAML Test
date: 2023-05-20
author:
  name: John Doe
  email: john@example.com
tags:
  - rust
  - metadata
  - testing
---
Content here"#;

        let metadata = extract_metadata(yaml_content).unwrap();
        assert_eq!(metadata.get("title").unwrap(), "Complex YAML Test");
        assert_eq!(metadata.get("date").unwrap(), "2023-05-20");
        assert_eq!(metadata.get("author.name").unwrap(), "John Doe");
        assert_eq!(
            metadata.get("author.email").unwrap(),
            "john@example.com"
        );
        assert_eq!(
            metadata.get("tags").unwrap(),
            "[rust, metadata, testing]"
        );
    }

    #[test]
    fn test_extract_toml_metadata_with_complex_structure() {
        let toml_content = r#"+++
title = "Complex TOML Test"
date = 2023-05-20

[author]
name = "John Doe"
email = "john@example.com"

tags = ["rust", "metadata", "testing"]
+++
Content here"#;

        let metadata = extract_metadata(toml_content).unwrap();
        assert_eq!(
            metadata.get("title").expect("Missing 'title' key"),
            "Complex TOML Test"
        );
        assert_eq!(
            metadata.get("date").expect("Missing 'date' key"),
            "2023-05-20"
        );
        assert_eq!(
            metadata
                .get("author.name")
                .expect("Missing 'author.name' key"),
            "John Doe"
        );
        assert_eq!(
            metadata
                .get("author.email")
                .expect("Missing 'author.email' key"),
            "john@example.com"
        );
        assert_eq!(
            metadata
                .get("author.tags")
                .expect("Missing 'author.tags' key"),
            "[rust, metadata, testing]"
        );
    }

    #[test]
    fn test_generate_slug_with_special_characters() {
        assert_eq!(
            generate_slug("Hello, World! 123"),
            "hello,-world!-123"
        );
        assert_eq!(generate_slug("Test: Ästhetik"), "test:-ästhetik");
        assert_eq!(
            generate_slug("  Multiple   Spaces  "),
            "--multiple---spaces--"
        );
    }

    #[test]
    fn test_extract_metadata_collapses_multiline_quoted_scalar() {
        // Regression for sebastienrousseau/metadata-gen#20.
        // A literal newline immediately after `: "` would previously
        // make noyalib reject the frontmatter and the user would see
        // the misleading "No valid front matter found" message.
        // The collapse step joins continuation lines so noyalib sees
        // valid input.
        let content = "---\n\
                       title: Test\n\
                       twitter_url: \"\n\
                       https://example.com/post\"\n\
                       ---\n\
                       body";
        let metadata = extract_metadata(content)
            .expect("multi-line quoted scalar should now parse");
        assert_eq!(metadata.get("title"), Some(&"Test".to_string()));
        assert!(
            metadata
                .get("twitter_url")
                .expect("twitter_url present")
                .contains("https://example.com/post"),
            "twitter_url should retain the URL after collapse"
        );
    }

    #[test]
    fn test_extract_json_metadata_with_nested_object() {
        // Regression for #26: the previous regex-based JSON detector
        // matched the first `}` it saw, so any nested object lost data
        // silently. The serde_json streaming path preserves it.
        let content = r#"{"title": "T", "author": {"name": "Ada", "handle": "ada@example.com"}}
# body"#;
        let meta = extract_metadata(content).expect("nested JSON parses");
        assert_eq!(meta.get("title"), Some(&"T".to_string()));
        assert_eq!(meta.get("author.name"), Some(&"Ada".to_string()));
        assert_eq!(
            meta.get("author.handle"),
            Some(&"ada@example.com".to_string())
        );
    }

    #[test]
    fn test_extract_json_metadata_with_array_of_objects() {
        // Issue #26 acceptance criterion: arrays of objects are preserved.
        let content = r#"{"tags": [{"name":"x"},{"name":"y"}]}
# body"#;
        let meta = extract_metadata(content).expect("array parses");
        // The flattened representation lists each element via its JSON
        // `Display` form; the important property is no data is lost.
        let tags = meta.get("tags").expect("tags key present");
        assert!(tags.contains("\"name\":\"x\""), "got: {tags}");
        assert!(tags.contains("\"name\":\"y\""), "got: {tags}");
    }

    #[test]
    fn test_extract_json_metadata_malformed_surfaces_error() {
        // Issue #26 acceptance criterion: malformed JSON returns
        // ExtractionError with the underlying serde_json message —
        // not the generic "No valid front matter found".
        let content = r#"{"title": "unterminated"# ; // intentionally malformed
        let err = extract_metadata(content).expect_err("must error");
        let msg = err.to_string();
        assert!(
            msg.contains("JSON parse error in frontmatter"),
            "expected surfaced JSON error, got: {msg}"
        );
        assert!(
            !msg.contains("No valid front matter found"),
            "should not fall back to the generic message: {msg}"
        );
    }

    #[test]
    fn test_extract_metadata_surfaces_yaml_parse_error() {
        // Regression for sebastienrousseau/metadata-gen#20.
        // A genuinely malformed YAML body (after the collapse step
        // can't help) should surface the noyalib parse error, not
        // the misleading "No valid front matter found" fallback.
        let content = "---\n\
                       title: [unclosed sequence\n\
                       ---\n\
                       body";
        let err = extract_metadata(content)
            .expect_err("malformed YAML should error");
        let msg = format!("{err}");
        assert!(
            msg.contains("YAML parse error in frontmatter"),
            "expected surfaced YAML error, got: {msg}"
        );
        assert!(
            !msg.contains("No valid front matter found"),
            "should not fall back to the generic message: {msg}"
        );
    }
}
