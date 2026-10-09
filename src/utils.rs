//! Utility functions for metadata processing and HTML manipulation.
//!
//! This module provides various utility functions for tasks such as HTML escaping,
//! asynchronous file reading, and metadata extraction from files.

use alloc::borrow::Cow;
use alloc::string::String;

/// Escapes special HTML characters in a string.
///
/// This function replaces the following characters with their HTML entity equivalents:
/// - `&` becomes `&amp;`
/// - `<` becomes `&lt;`
/// - `>` becomes `&gt;`
/// - `"` becomes `&quot;`
/// - `'` becomes `&#x27;`
///
/// # Arguments
///
/// * `value` - The string to escape.
///
/// # Returns
///
/// A new string with special HTML characters escaped.
///
/// # Examples
///
/// ```
/// use metadata_gen::utils::escape_html;
///
/// let input = "Hello, <world>!";
/// let expected = "Hello, &lt;world&gt;!";
///
/// assert_eq!(escape_html(input), expected);
/// ```
///
/// # Security
///
/// This function is designed to prevent XSS (Cross-Site Scripting) attacks by escaping
/// potentially dangerous characters. However, it should not be relied upon as the sole
/// method of sanitizing user input for use in HTML contexts.
pub fn escape_html(value: &str) -> String {
    // One pass, one allocation. The five-`replace` chain this replaces
    // walked the string five times and allocated up to five
    // intermediates; the output is byte-identical.
    let mut out = String::with_capacity(value.len() + value.len() / 8);
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            other => out.push(other),
        }
    }
    out
}

/// Escapes HTML without allocating when nothing needs escaping.
///
/// Returns `Cow::Borrowed(value)` when `value` contains none of `&`, `<`,
/// `>`, `"` or `'`, and otherwise the same string [`escape_html`] produces.
///
/// # Example
///
/// ```
/// use metadata_gen::utils::escape_html_cow;
/// use std::borrow::Cow;
///
/// assert!(matches!(escape_html_cow("plain"), Cow::Borrowed("plain")));
/// assert_eq!(escape_html_cow("a < b"), "a &lt; b");
/// ```
pub fn escape_html_cow(value: &str) -> Cow<'_, str> {
    if value
        .bytes()
        .any(|b| matches!(b, b'&' | b'<' | b'>' | b'"' | b'\''))
    {
        Cow::Owned(escape_html(value))
    } else {
        Cow::Borrowed(value)
    }
}

/// Escapes a value for use inside a double-quoted HTML attribute.
///
/// `&`, `<`, `>` and `"` are replaced; an apostrophe is left alone because
/// it cannot terminate a double-quoted attribute, which keeps rendered
/// meta tags readable.
///
/// # Example
///
/// ```
/// use metadata_gen::escape_attribute;
///
/// assert_eq!(
///     escape_attribute(r#"Fish & "Chips" <it's>"#),
///     "Fish &amp; &quot;Chips&quot; &lt;it's&gt;"
/// );
/// ```
pub fn escape_attribute(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + value.len() / 8);
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

/// Unescapes HTML entities in a string.
///
/// This function replaces HTML entities with their corresponding characters:
/// - `&amp;` becomes `&`
/// - `&lt;` becomes `<`
/// - `&gt;` becomes `>`
/// - `&quot;` becomes `"`
/// - `&#x27;` and `&#39;` become `'`
/// - `&#x2F;` and `&#x2f;` become `/`
///
/// # Arguments
///
/// * `value` - The string to unescape.
///
/// # Returns
///
/// A new string with HTML entities unescaped.
///
/// # Examples
///
/// ```
/// use metadata_gen::utils::unescape_html;
///
/// let input = "Hello, &lt;world&gt;!";
/// let expected = "Hello, <world>!";
///
/// assert_eq!(unescape_html(input), expected);
/// ```
///
/// # Security
///
/// This function should be used with caution, especially on user-supplied input,
/// as it can potentially introduce security vulnerabilities if the unescaped content
/// is then rendered as HTML.
pub fn unescape_html(value: &str) -> String {
    // One left-to-right pass. Each entity is decoded exactly once and the
    // decoded text is never rescanned, so `&amp;lt;` yields `&lt;`, not
    // `<`. A chain of `replace` calls did rescan, and the fuzz target
    // caught it on its seed corpus: escape/unescape was not an identity.
    const ENTITIES: [(&str, &str); 8] = [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#x27;", "'"),
        ("&#39;", "'"),
        ("&#x2F;", "/"),
        ("&#x2f;", "/"),
    ];
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        match ENTITIES.iter().find(|(name, _)| tail.starts_with(name)) {
            Some((name, decoded)) => {
                out.push_str(decoded);
                rest = &tail[name.len()..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Asynchronously extracts metadata from a file (feature `tokio`).
///
/// Kept for callers of earlier releases; new code should use
/// [`crate::tokio::extract_from_file`], which accepts any path type.
///
/// # Errors
///
/// As [`crate::tokio::extract_from_file`].
///
/// # Security
///
/// This function reads files from the file system. Ensure that the `file_path`
/// is properly sanitized and validated to prevent potential security issues like
/// path traversal attacks.
///
/// # Example
///
/// ```no_run
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// use metadata_gen::utils::async_extract_metadata_from_file;
///
/// let result = async_extract_metadata_from_file("content/post.md").await;
/// # let _ = result;
/// # });
/// ```
#[cfg(all(feature = "tokio", not(loom)))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub async fn async_extract_metadata_from_file(
    file_path: &str,
) -> Result<
    (crate::MetadataMap, crate::Keywords, crate::MetaTagGroups),
    crate::error::MetadataError,
> {
    crate::tokio::extract_from_file(file_path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(all(feature = "tokio", not(loom)))]
    use crate::error::MetadataError;
    #[cfg(all(feature = "tokio", not(loom)))]
    use tempfile::tempdir;
    #[cfg(all(feature = "tokio", not(loom)))]
    use tokio::fs::File;
    #[cfg(all(feature = "tokio", not(loom)))]
    use tokio::io::AsyncWriteExt;

    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[test]
    fn test_escape_html() {
        let input = "Hello, <world> & \"friends\"!";
        let expected =
            "Hello, &lt;world&gt; &amp; &quot;friends&quot;!";
        assert_eq!(escape_html(input), expected);
    }

    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[test]
    fn test_escape_html_special_characters() {
        let input = "It's <b>bold</b> & it's <i>italic</i>";
        let expected = "It&#x27;s &lt;b&gt;bold&lt;/b&gt; &amp; it&#x27;s &lt;i&gt;italic&lt;/i&gt;";
        assert_eq!(escape_html(input), expected);
    }

    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[test]
    fn test_unescape_html() {
        let input = "Hello, &lt;world&gt; &amp; &quot;friends&quot;!";
        let expected = "Hello, <world> & \"friends\"!";
        assert_eq!(unescape_html(input), expected);
    }

    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[test]
    fn test_unescape_html_edge_cases() {
        let input = "&lt;&amp;&gt;&quot;&#x27;&#39;&#x2F;";
        let expected = "<&>\"''/";
        assert_eq!(unescape_html(input), expected);
    }

    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[test]
    fn test_escape_unescape_roundtrip() {
        let original = "Test <script>alert('XSS');</script> & other \"special\" chars";
        let escaped = escape_html(original);
        let unescaped = unescape_html(&escaped);
        assert_eq!(original, unescaped);
    }

    #[cfg(all(feature = "tokio", not(loom)))]
    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[tokio::test]
    async fn test_async_extract_metadata_from_file() {
        // Create a temporary directory and file
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.md");

        // Write test content to the file
        let content = r#"---
title: Test Page
description: A test page for metadata extraction
keywords: test, metadata, extraction
---
# Test Content
This is a test file for metadata extraction."#;

        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        // Test the async_extract_metadata_from_file function
        let result = async_extract_metadata_from_file(
            file_path.to_str().unwrap(),
        )
        .await;
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

    #[cfg(all(feature = "tokio", not(loom)))]
    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[tokio::test]
    async fn test_async_extract_metadata_from_empty_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.md");

        // Create an empty file
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"").await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        let result = async_extract_metadata_from_file(
            file_path.to_str().unwrap(),
        )
        .await;

        // Ensure the result is empty metadata, keywords, and meta tags
        assert!(result.is_ok());
        let (metadata, keywords, meta_tags) = result.unwrap();
        assert!(metadata.is_empty());
        assert!(keywords.is_empty());
        assert!(meta_tags.primary.is_empty());
    }

    #[cfg(all(feature = "tokio", not(loom)))]
    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[tokio::test]
    async fn test_async_extract_metadata_from_nonexistent_file() {
        let result =
            async_extract_metadata_from_file("nonexistent_file.md")
                .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MetadataError::IoError(_)
        ));
    }
}

#[cfg(test)]
mod unescape_single_pass_tests {
    //! Found by `fuzz_html_escape` on its seed corpus: unescaping was a
    //! chain of `replace` calls, so `&amp;lt;` decoded to `<` — the
    //! output of one replacement was re-read by the next. A decoder
    //! that can turn an escaped `&lt;` back into a raw `<` undoes the
    //! escaping that `escape_html` exists to provide.

    use super::*;

    #[test]
    fn unescape_does_not_rescan_its_own_output() {
        assert_eq!(unescape_html("&amp;lt;"), "&lt;");
        assert_eq!(unescape_html("&amp;amp;"), "&amp;");
        assert_eq!(unescape_html("&amp;#39;"), "&#39;");
    }

    #[test]
    fn escape_then_unescape_is_the_identity() {
        for s in [
            "&amp;&lt;&#39;&#x27;",
            "a < b && c > \"d\" 'e'",
            "&",
            "&&amp;",
            "plain",
        ] {
            assert_eq!(unescape_html(&escape_html(s)), s, "{s:?}");
        }
    }

    #[test]
    fn unknown_and_unterminated_entities_pass_through() {
        assert_eq!(unescape_html("&unknown;"), "&unknown;");
        assert_eq!(unescape_html("&amp"), "&amp");
        assert_eq!(unescape_html("& x"), "& x");
        assert_eq!(unescape_html("&#x2F;&#x2f;&#39;"), "//'");
    }
}

#[cfg(test)]
mod escape_single_pass_tests {
    use super::*;

    #[test]
    fn escape_html_cow_borrows_clean_input() {
        assert!(matches!(
            escape_html_cow("plain text"),
            Cow::Borrowed(_)
        ));
        for dirty in ["a&b", "a<b", "a>b", "a\"b", "a'b"] {
            let out = escape_html_cow(dirty);
            assert!(matches!(out, Cow::Owned(_)), "{dirty}");
            assert_eq!(out, escape_html(dirty));
        }
    }

    #[test]
    fn escape_attribute_keeps_apostrophes() {
        assert_eq!(
            escape_attribute("<a href=\"x\">it's & that</a>"),
            "&lt;a href=&quot;x&quot;&gt;it's &amp; that&lt;/a&gt;"
        );
    }

    #[test]
    fn escape_matches_the_replace_chain_it_replaced() {
        let reference = |v: &str| {
            v.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#x27;")
        };
        for s in [
            "",
            "plain",
            "a<b>c&d\"e'f",
            "&&&",
            "<<>>",
            "ünïcödé <tag> & 'q'",
            "&amp;",
        ] {
            assert_eq!(escape_html(s), reference(s), "{s:?}");
        }
    }
}
