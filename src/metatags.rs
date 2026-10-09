//! Meta tag generation and extraction module.
//!
//! This module provides functionality for generating HTML meta tags from metadata
//! and extracting meta tags from HTML content.
//!
//! Rendering follows the specifications the tags come from: Open Graph
//! (`og:*`, `article:*`, `fb:*`, `profile:*`, `book:*`, `music:*`,
//! `video:*`) uses the `property` attribute, everything else uses `name`,
//! and both attribute values go through [`crate::escape_attribute`].

use crate::utils::escape_attribute;
use crate::MetadataMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Holds collections of meta tags for different platforms and categories.
///
/// Each field is the rendered `<meta>` elements of one group, one per
/// line. [`MetaTagGroups::iter`] gives the same tags as values.
///
/// # Example
///
/// ```
/// use metadata_gen::metatags::generate_metatags;
/// use metadata_gen::MetadataMap;
///
/// let mut metadata = MetadataMap::new();
/// metadata.insert("description".to_string(), "A sample page".to_string());
/// metadata.insert("og:title".to_string(), "Sample".to_string());
///
/// let tags = generate_metatags(&metadata);
/// assert!(tags.primary.contains("description"));
/// assert!(tags.og.contains(r#"property="og:title""#));
/// ```
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
#[non_exhaustive]
pub struct MetaTagGroups {
    /// Meta tags specific to Apple devices.
    pub apple: String,
    /// Primary meta tags (description, keywords, author, viewport).
    pub primary: String,
    /// Open Graph meta tags.
    pub og: String,
    /// Microsoft-specific meta tags.
    pub ms: String,
    /// Twitter meta tags.
    pub twitter: String,
}

/// Which attribute carries a meta tag's identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MetaAttribute {
    /// `<meta name="…">`: the HTML default, used by Twitter cards too.
    Name,
    /// `<meta property="…">`: what the Open Graph protocol requires.
    Property,
    /// `<meta http-equiv="…">`.
    HttpEquiv,
}

impl MetaAttribute {
    /// The attribute name as written in HTML.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Property => "property",
            Self::HttpEquiv => "http-equiv",
        }
    }
}

/// One `<meta>` element: its identifier and content.
///
/// # Example
///
/// ```
/// use metadata_gen::MetaTag;
///
/// let tag = MetaTag::new("og:title", r#"Fish & "Chips""#);
/// assert_eq!(
///     tag.render(),
///     r#"<meta property="og:title" content="Fish &amp; &quot;Chips&quot;">"#
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct MetaTag {
    /// The tag's identifier (`description`, `og:title`, `twitter:card`).
    pub name: String,
    /// The tag's content.
    pub content: String,
}

/// Prefixes whose tags the Open Graph protocol identifies with `property=`.
const PROPERTY_PREFIXES: [&str; 7] = [
    "og:", "article:", "fb:", "profile:", "book:", "music:", "video:",
];

impl MetaTag {
    /// Creates a tag from its identifier and content.
    pub fn new(
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }

    /// The attribute this tag's identifier belongs in.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::metatags::{MetaAttribute, MetaTag};
    ///
    /// assert_eq!(MetaTag::new("og:image", "x").attribute(), MetaAttribute::Property);
    /// assert_eq!(MetaTag::new("twitter:card", "x").attribute(), MetaAttribute::Name);
    /// assert_eq!(MetaTag::new("refresh", "x").attribute(), MetaAttribute::Name);
    /// ```
    pub fn attribute(&self) -> MetaAttribute {
        if PROPERTY_PREFIXES.iter().any(|p| self.name.starts_with(p)) {
            MetaAttribute::Property
        } else {
            MetaAttribute::Name
        }
    }

    /// Renders the element with both attribute values escaped.
    pub fn render(&self) -> String {
        format!(
            r#"<meta {}="{}" content="{}">"#,
            self.attribute().as_str(),
            escape_attribute(&self.name),
            escape_attribute(&self.content)
        )
    }
}

impl fmt::Display for MetaTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

/// The group a tag renders into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    Apple,
    Primary,
    Og,
    Ms,
    Twitter,
}

/// The tags each group renders, in order. Adding a tag is one row.
const TAG_TABLE: [(Group, &str); 20] = [
    (Group::Apple, "apple-mobile-web-app-capable"),
    (Group::Apple, "mobile-web-app-capable"),
    (Group::Apple, "apple-mobile-web-app-status-bar-style"),
    (Group::Apple, "apple-mobile-web-app-title"),
    (Group::Primary, "author"),
    (Group::Primary, "description"),
    (Group::Primary, "keywords"),
    (Group::Primary, "viewport"),
    (Group::Og, "og:title"),
    (Group::Og, "og:description"),
    (Group::Og, "og:image"),
    (Group::Og, "og:url"),
    (Group::Og, "og:type"),
    (Group::Ms, "msapplication-TileColor"),
    (Group::Ms, "msapplication-TileImage"),
    (Group::Twitter, "twitter:card"),
    (Group::Twitter, "twitter:site"),
    (Group::Twitter, "twitter:title"),
    (Group::Twitter, "twitter:description"),
    (Group::Twitter, "twitter:image"),
];

/// The group a custom tag name routes to.
fn group_for(name: &str) -> Group {
    if name.starts_with("apple-") || name == "mobile-web-app-capable" {
        Group::Apple
    } else if name.starts_with("msapplication-") {
        Group::Ms
    } else if PROPERTY_PREFIXES.iter().any(|p| name.starts_with(p)) {
        Group::Og
    } else if name.starts_with("twitter:") {
        Group::Twitter
    } else {
        Group::Primary
    }
}

impl MetaTagGroups {
    fn field_mut(&mut self, group: Group) -> &mut String {
        match group {
            Group::Apple => &mut self.apple,
            Group::Primary => &mut self.primary,
            Group::Og => &mut self.og,
            Group::Ms => &mut self.ms,
            Group::Twitter => &mut self.twitter,
        }
    }

    /// Adds a custom meta tag to the appropriate group, chosen by its
    /// name: `apple-*` and `mobile-web-app-capable` go to `apple`,
    /// `msapplication-*` to `ms`, Open Graph prefixes to `og`,
    /// `twitter:*` to `twitter`, everything else to `primary`.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::MetaTagGroups;
    ///
    /// let mut groups = MetaTagGroups::default();
    /// groups.add_custom_tag("og:locale", "en_GB");
    /// assert_eq!(groups.og, r#"<meta property="og:locale" content="en_GB">"#);
    /// ```
    pub fn add_custom_tag(&mut self, name: &str, content: &str) {
        let rendered = MetaTag::new(name, content).render();
        let field = self.field_mut(group_for(name));
        if !field.is_empty() {
            field.push('\n');
        }
        field.push_str(&rendered);
    }

    /// Formats one meta tag as [`MetaTag::render`] does.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::MetaTagGroups;
    ///
    /// let groups = MetaTagGroups::default();
    /// assert_eq!(
    ///     groups.format_meta_tag("description", r#"say "hi""#),
    ///     r#"<meta name="description" content="say &quot;hi&quot;">"#
    /// );
    /// ```
    pub fn format_meta_tag(&self, name: &str, content: &str) -> String {
        MetaTag::new(name, content).render()
    }

    /// Generates the Apple-specific group from `metadata`.
    pub fn generate_apple_meta_tags(&mut self, metadata: &MetadataMap) {
        self.apple = self.generate_group(metadata, Group::Apple);
    }

    /// Generates the primary group (author, description, keywords,
    /// viewport) from `metadata`.
    pub fn generate_primary_meta_tags(
        &mut self,
        metadata: &MetadataMap,
    ) {
        self.primary = self.generate_group(metadata, Group::Primary);
    }

    /// Generates the Open Graph group from `metadata`.
    pub fn generate_og_meta_tags(&mut self, metadata: &MetadataMap) {
        self.og = self.generate_group(metadata, Group::Og);
    }

    /// Generates the Microsoft-specific group from `metadata`.
    pub fn generate_ms_meta_tags(&mut self, metadata: &MetadataMap) {
        self.ms = self.generate_group(metadata, Group::Ms);
    }

    /// Generates the Twitter group from `metadata`.
    pub fn generate_twitter_meta_tags(
        &mut self,
        metadata: &MetadataMap,
    ) {
        self.twitter = self.generate_group(metadata, Group::Twitter);
    }

    fn generate_group(
        &self,
        metadata: &MetadataMap,
        group: Group,
    ) -> String {
        let names: Vec<&str> = TAG_TABLE
            .iter()
            .filter(|(g, _)| *g == group)
            .map(|(_, name)| *name)
            .collect();
        self.generate_tags(metadata, &names)
    }

    /// Renders every tag in `tags` that has a value in `metadata`, one
    /// per line, in the order given.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::{MetaTagGroups, MetadataMap};
    ///
    /// let mut metadata = MetadataMap::new();
    /// metadata.insert("author".into(), "Ada".into());
    /// let groups = MetaTagGroups::default();
    /// assert_eq!(
    ///     groups.generate_tags(&metadata, &["author", "missing"]),
    ///     r#"<meta name="author" content="Ada">"#
    /// );
    /// ```
    pub fn generate_tags(
        &self,
        metadata: &MetadataMap,
        tags: &[&str],
    ) -> String {
        tags.iter()
            .filter_map(|&tag| {
                metadata.get(tag).map(|value| {
                    MetaTag::new(tag, value.as_str()).render()
                })
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Every rendered element across the five groups, in group order,
    /// as [`MetaTag`] values again.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::MetaTagGroups;
    ///
    /// let mut groups = MetaTagGroups::default();
    /// groups.add_custom_tag("og:title", "T");
    /// groups.add_custom_tag("author", "A");
    /// let names: Vec<String> = groups.iter().map(|t| t.name).collect();
    /// assert_eq!(names, ["author", "og:title"]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = MetaTag> + '_ {
        [
            &self.apple,
            &self.primary,
            &self.og,
            &self.ms,
            &self.twitter,
        ]
        .into_iter()
        .flat_map(|group| group.lines())
        .filter_map(parse_rendered)
    }
}

/// Reads a rendered element back into a [`MetaTag`]; `None` for a line
/// that is not one of ours.
fn parse_rendered(line: &str) -> Option<MetaTag> {
    let rest = line.strip_prefix("<meta ")?;
    let (_, rest) = rest.split_once("=\"")?;
    let (name, rest) = rest.split_once("\" content=\"")?;
    let content = rest.strip_suffix("\">")?;
    Some(MetaTag::new(
        crate::utils::unescape_html(name),
        crate::utils::unescape_html(content),
    ))
}

impl fmt::Display for MetaTagGroups {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}",
            self.apple, self.primary, self.og, self.ms, self.twitter
        )
    }
}

/// Generates all meta tag groups from the given metadata.
///
/// # Arguments
///
/// * `metadata` - A map of metadata keys to values.
///
/// # Returns
///
/// A [`MetaTagGroups`] with one rendered group per platform.
///
/// # Example
///
/// ```
/// use metadata_gen::metatags::generate_metatags;
/// use metadata_gen::MetadataMap;
///
/// let mut metadata = MetadataMap::new();
/// metadata.insert("description".to_string(), "A sample page".to_string());
/// metadata.insert("twitter:card".to_string(), "summary".to_string());
///
/// let tags = generate_metatags(&metadata);
/// assert_eq!(tags.primary, r#"<meta name="description" content="A sample page">"#);
/// assert_eq!(tags.twitter, r#"<meta name="twitter:card" content="summary">"#);
/// ```
pub fn generate_metatags(metadata: &MetadataMap) -> MetaTagGroups {
    let mut meta_tag_groups = MetaTagGroups::default();
    meta_tag_groups.generate_apple_meta_tags(metadata);
    meta_tag_groups.generate_primary_meta_tags(metadata);
    meta_tag_groups.generate_og_meta_tags(metadata);
    meta_tag_groups.generate_ms_meta_tags(metadata);
    meta_tag_groups.generate_twitter_meta_tags(metadata);
    meta_tag_groups
}

#[cfg(feature = "html")]
mod html;
#[cfg(feature = "html")]
#[cfg_attr(docsrs, doc(cfg(feature = "html")))]
pub use html::{extract_meta_tags, extract_meta_tags_lenient};

/// Converts a list of tags into a map from identifier to content.
///
/// A repeated identifier keeps the last value.
///
/// # Example
///
/// ```
/// use metadata_gen::metatags::{meta_tags_to_hashmap, MetaTag};
///
/// let map = meta_tags_to_hashmap(vec![MetaTag::new("a", "1"), MetaTag::new("a", "2")]);
/// assert_eq!(map["a"], "2");
/// ```
pub fn meta_tags_to_hashmap(meta_tags: Vec<MetaTag>) -> MetadataMap {
    meta_tags
        .into_iter()
        .map(|tag| (tag.name, tag.content))
        .collect()
}

#[cfg(all(test, feature = "html"))]
mod tests;
