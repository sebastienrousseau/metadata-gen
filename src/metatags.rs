//! Meta tag generation and extraction module.
//!
//! This module provides functionality for generating HTML meta tags from metadata
//! and extracting meta tags from HTML content.

use crate::error::MetadataError;
use scraper::{Html, Selector};
use std::{collections::HashMap, fmt};

/// Holds collections of meta tags for different platforms and categories.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct MetaTagGroups {
    /// The `apple` meta tags.
    pub apple: String,
    /// The primary meta tags.
    pub primary: String,
    /// The `og` meta tags.
    pub og: String,
    /// The `ms` meta tags.
    pub ms: String,
    /// The `twitter` meta tags.
    pub twitter: String,
}

/// Represents a single meta tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaTag {
    /// The name or property of the meta tag
    pub name: String,
    /// The content of the meta tag
    pub content: String,
}

impl MetaTagGroups {
    /// Adds a custom meta tag to the appropriate group.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the meta tag.
    /// * `content` - The content of the meta tag.
    pub fn add_custom_tag(&mut self, name: &str, content: &str) {
        let formatted_tag = self.format_meta_tag(name, content);

        // Match based on specific prefixes for Apple, MS, OG, Twitter, etc.
        if name.starts_with("apple-") {
            // println!("Adding Apple meta tag: {}", formatted_tag);  // Debugging output
            self.apple.push_str(&formatted_tag);
        } else if name.starts_with("msapplication-") {
            // println!("Adding MS meta tag: {}", formatted_tag);  // Debugging output
            self.ms.push_str(&formatted_tag);
        } else if name.starts_with("og:") {
            // println!("Adding OG meta tag: {}", formatted_tag);  // Debugging output
            self.og.push_str(&formatted_tag);
        } else if name.starts_with("twitter:") {
            // println!("Adding Twitter meta tag: {}", formatted_tag);  // Debugging output
            self.twitter.push_str(&formatted_tag);
        } else {
            // println!("Adding Primary meta tag: {}", formatted_tag);  // Debugging output
            self.primary.push_str(&formatted_tag);
        }
    }

    /// Formats a single meta tag.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the meta tag.
    /// * `content` - The content of the meta tag.
    ///
    /// # Returns
    ///
    /// A formatted meta tag string.
    pub fn format_meta_tag(&self, name: &str, content: &str) -> String {
        format!(
            r#"<meta name="{}" content="{}">"#,
            name,
            content.replace('"', "&quot;")
        )
    }

    /// Generates meta tags for Apple devices.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_apple_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const APPLE_TAGS: [&str; 3] = [
            "apple-mobile-web-app-capable",
            "apple-mobile-web-app-status-bar-style",
            "apple-mobile-web-app-title",
        ];
        self.apple = self.generate_tags(metadata, &APPLE_TAGS);
    }

    /// Generates primary meta tags like `author`, `description`, and `keywords`.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_primary_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const PRIMARY_TAGS: [&str; 4] =
            ["author", "description", "keywords", "viewport"];
        self.primary = self.generate_tags(metadata, &PRIMARY_TAGS);
    }

    /// Generates Open Graph (`og`) meta tags for social media.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_og_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const OG_TAGS: [&str; 5] = [
            "og:title",
            "og:description",
            "og:image",
            "og:url",
            "og:type",
        ];
        self.og = self.generate_tags(metadata, &OG_TAGS);
    }

    /// Generates Microsoft-specific meta tags.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_ms_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const MS_TAGS: [&str; 2] =
            ["msapplication-TileColor", "msapplication-TileImage"];
        self.ms = self.generate_tags(metadata, &MS_TAGS);
    }

    /// Generates Twitter meta tags for embedding rich media in tweets.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a HashMap containing the metadata.
    pub fn generate_twitter_meta_tags(
        &mut self,
        metadata: &HashMap<String, String>,
    ) {
        const TWITTER_TAGS: [&str; 5] = [
            "twitter:card",
            "twitter:site",
            "twitter:title",
            "twitter:description",
            "twitter:image",
        ];
        self.twitter = self.generate_tags(metadata, &TWITTER_TAGS);
    }

    /// Generates meta tags based on the provided list of tag names.
    ///
    /// # Arguments
    ///
    /// * `metadata` - A reference to a `HashMap` containing the metadata.
    /// * `tags` - A reference to an array of tag names.
    ///
    /// # Returns
    ///
    /// A string containing the generated meta tags.
    pub fn generate_tags(
        &self,
        metadata: &HashMap<String, String>,
        tags: &[&str],
    ) -> String {
        tags.iter()
            .filter_map(|&tag| {
                metadata
                    .get(tag)
                    .map(|value| self.format_meta_tag(tag, value))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Implement `Display` for `MetaTagGroups`.
impl fmt::Display for MetaTagGroups {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}",
            self.apple, self.primary, self.og, self.ms, self.twitter
        )
    }
}

/// Generates HTML meta tags based on the provided metadata.
///
/// This function takes metadata from a `HashMap` and generates meta tags for various platforms (e.g., Apple, Open Graph, Twitter).
///
/// # Arguments
///
/// * `metadata` - A reference to a `HashMap` containing the metadata.
///
/// # Returns
///
/// A `MetaTagGroups` structure with meta tags grouped by platform.
pub fn generate_metatags(
    metadata: &HashMap<String, String>,
) -> MetaTagGroups {
    let mut meta_tag_groups = MetaTagGroups::default();
    meta_tag_groups.generate_apple_meta_tags(metadata);
    meta_tag_groups.generate_primary_meta_tags(metadata);
    meta_tag_groups.generate_og_meta_tags(metadata);
    meta_tag_groups.generate_ms_meta_tags(metadata);
    meta_tag_groups.generate_twitter_meta_tags(metadata);
    meta_tag_groups
}

/// Extracts meta tags from HTML content.
///
/// This function parses the given HTML content and extracts all meta tags,
/// including both `name` and `property` attributes.
///
/// # Arguments
///
/// * `html_content` - A string slice containing the HTML content to parse.
///
/// # Returns
///
/// Returns a `Result` containing a `Vec<MetaTag>` if successful, or a `MetadataError` if parsing fails.
///
/// # Errors
///
/// This function will return a `MetadataError` if:
/// - The HTML content cannot be parsed.
/// - The meta tag selector cannot be created.
pub fn extract_meta_tags(
    html_content: &str,
) -> Result<Vec<MetaTag>, MetadataError> {
    let document = Html::parse_document(html_content);

    let meta_selector = Selector::parse("meta").map_err(|e| {
        MetadataError::ExtractionError {
            message: format!(
                "Failed to create meta tag selector: {}",
                e
            ),
        }
    })?;

    let mut meta_tags = Vec::new();

    for element in document.select(&meta_selector) {
        let name = element
            .value()
            .attr("name")
            .or_else(|| element.value().attr("property"))
            .or_else(|| element.value().attr("http-equiv"));

        let content = element.value().attr("content");

        if let (Some(name), Some(content)) = (name, content) {
            meta_tags.push(MetaTag {
                name: name.to_string(),
                content: content.to_string(),
            });
        }
    }

    Ok(meta_tags)
}

/// Converts a vector of MetaTags into a HashMap for easier access.
///
/// # Arguments
///
/// * `meta_tags` - A vector of MetaTag structs.
///
/// # Returns
///
/// A HashMap where the keys are the meta tag names and the values are the contents.
pub fn meta_tags_to_hashmap(
    meta_tags: Vec<MetaTag>,
) -> HashMap<String, String> {
    meta_tags
        .into_iter()
        .map(|tag| (tag.name, tag.content))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_metatags() {
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Page".to_string());
        metadata.insert(
            "description".to_string(),
            "A test page".to_string(),
        );
        metadata
            .insert("og:title".to_string(), "OG Test Page".to_string());

        let meta_tags = generate_metatags(&metadata);

        assert!(meta_tags.primary.contains("description"));
        assert!(meta_tags.og.contains("og:title"));
    }

    #[test]
    fn test_extract_meta_tags() {
        let html = r#"
        <html>
          <head>
            <meta name="description" content="A sample page">
            <meta property="og:title" content="Sample Title">
            <meta http-equiv="content-type" content="text/html; charset=UTF-8">
          </head>
          <body>
            <p>Some content</p>
          </body>
        </html>
        "#;

        let meta_tags = extract_meta_tags(html).unwrap();
        assert_eq!(meta_tags.len(), 3);
        assert!(meta_tags.iter().any(|tag| tag.name == "description"
            && tag.content == "A sample page"));
        assert!(meta_tags.iter().any(|tag| tag.name == "og:title"
            && tag.content == "Sample Title"));
        assert!(meta_tags.iter().any(|tag| tag.name == "content-type"
            && tag.content == "text/html; charset=UTF-8"));
    }

    #[test]
    fn test_extract_meta_tags_empty_html() {
        let html = "<html><head></head><body></body></html>";
        let meta_tags = extract_meta_tags(html).unwrap();
        assert_eq!(meta_tags.len(), 0);
    }

    #[test]
    fn test_meta_tags_to_hashmap() {
        let meta_tags = vec![
            MetaTag {
                name: "description".to_string(),
                content: "A sample page".to_string(),
            },
            MetaTag {
                name: "og:title".to_string(),
                content: "Sample Title".to_string(),
            },
        ];

        let hashmap = meta_tags_to_hashmap(meta_tags);
        assert_eq!(hashmap.len(), 2);
        assert_eq!(
            hashmap.get("description"),
            Some(&"A sample page".to_string())
        );
        assert_eq!(
            hashmap.get("og:title"),
            Some(&"Sample Title".to_string())
        );
    }

    #[test]
    fn test_meta_tag_groups_display() {
        let groups = MetaTagGroups {
    apple: "<meta name=\"apple-mobile-web-app-capable\" content=\"yes\">".to_string(),
    primary: "<meta name=\"description\" content=\"A test page\">".to_string(),
    og: "<meta property=\"og:title\" content=\"Test Page\">".to_string(),
    ms: "<meta name=\"msapplication-TileColor\" content=\"#ffffff\">".to_string(),
    twitter: "<meta name=\"twitter:card\" content=\"summary\">".to_string(),
};

        let display = groups.to_string();
        assert!(display.contains("apple-mobile-web-app-capable"));
        assert!(display.contains("description"));
        assert!(display.contains("og:title"));
        assert!(display.contains("msapplication-TileColor"));
        assert!(display.contains("twitter:card"));
    }

    #[test]
    fn test_format_meta_tag() {
        let groups = MetaTagGroups::default();
        let tag = groups.format_meta_tag("test", "Test \"Value\"");
        assert_eq!(
            tag,
            r#"<meta name="test" content="Test &quot;Value&quot;">"#
        );
    }
}
