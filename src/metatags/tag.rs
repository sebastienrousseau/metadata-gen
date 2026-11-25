// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! One `<meta>` element, [`MetaTag`], and the attribute that carries
//! its identifier, [`MetaAttribute`].

use crate::utils::escape_attribute;
use alloc::format;
use alloc::string::String;
use core::fmt;

/// Which attribute carries a meta tag's identifier; returned by
/// [`MetaTag::attribute`], whose example shows each case.
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
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(metadata_gen::metatags::MetaAttribute::HttpEquiv.as_str(), "http-equiv");
    /// ```
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
pub(super) const PROPERTY_PREFIXES: [&str; 7] = [
    "og:", "article:", "fb:", "profile:", "book:", "music:", "video:",
];

impl MetaTag {
    /// Creates a tag from its identifier and content.
    ///
    /// # Example
    ///
    /// ```
    /// let tag = metadata_gen::MetaTag::new("description", "A page");
    /// assert_eq!((tag.name.as_str(), tag.content.as_str()), ("description", "A page"));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// let tag = metadata_gen::MetaTag::new("description", "A < B");
    /// assert_eq!(tag.render(), r#"<meta name="description" content="A &lt; B">"#);
    /// ```
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
