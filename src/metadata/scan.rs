// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Front-matter delimiter scanning.
//!
//! Finds the fenced block (`---` for YAML, `+++` for TOML) or the leading
//! JSON object without a regex: two `memchr::memmem` searches per fence,
//! so a 10 MiB document costs one pass over the opening line and one
//! SIMD scan for the closing fence.
//!
//! Rules, kept identical to the regex they replace where it matters for
//! output stability:
//!
//! - A UTF-8 BOM and leading whitespace before the opening fence are
//!   allowed.
//! - The opening fence is followed by optional whitespace and a newline.
//! - The closing fence sits at the start of a line (after optional
//!   whitespace), and that line is not the opening line.
//! - YAML: the body starts after the closing fence and any whitespace
//!   that follows it. TOML: the body starts right after the closing fence.
//! - The raw block is trimmed.

use super::FrontMatterFormat;
use memchr::memmem;

/// What the scanner found at the top of a document.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Scan<'a> {
    /// A complete block whose format is compiled in.
    Found {
        /// The block's format.
        format: FrontMatterFormat,
        /// The trimmed text between the fences.
        raw: &'a str,
        /// Byte offset where the body starts.
        body_offset: usize,
    },
    /// An opening fence with no closing fence; `offset` is the fence's byte offset.
    Unterminated {
        /// The format whose fence opened the block.
        format: FrontMatterFormat,
        /// Byte offset of the opening fence.
        offset: usize,
    },
    /// A block in a format that is not compiled in.
    Unsupported(FrontMatterFormat),
    /// No front matter.
    None,
}

/// Scans `content` for YAML, then TOML, then JSON front matter.
pub(super) fn scan(content: &str) -> Scan<'_> {
    if let Some(found) = fenced(content, "---", FrontMatterFormat::Yaml)
    {
        return gate(found, cfg!(feature = "yaml"));
    }
    if let Some(found) = fenced(content, "+++", FrontMatterFormat::Toml)
    {
        return gate(found, cfg!(feature = "toml"));
    }
    json(content)
}

/// Turns a found block into `Unsupported` when its format is compiled out.
fn gate(found: Scan<'_>, enabled: bool) -> Scan<'_> {
    match found {
        Scan::Found { format, .. } if !enabled => {
            Scan::Unsupported(format)
        }
        other => other,
    }
}

/// Finds a block fenced by `fence`, or `None` when `content` does not open with it.
fn fenced<'a>(
    content: &'a str,
    fence: &str,
    format: FrontMatterFormat,
) -> Option<Scan<'a>> {
    let (open_at, raw_start) = opening_fence(content, fence)?;
    let Some(close_at) = closing_fence(content, raw_start, fence)
    else {
        return Some(Scan::Unterminated {
            format,
            offset: open_at,
        });
    };
    Some(Scan::Found {
        format,
        raw: content[raw_start..close_at].trim(),
        body_offset: body_start(
            content,
            close_at + fence.len(),
            format,
        ),
    })
}

/// The opening fence's byte offset and the offset just past its line, or
/// `None` when `content` does not open with `fence` on a line of its own.
fn opening_fence(content: &str, fence: &str) -> Option<(usize, usize)> {
    let without_bom =
        content.strip_prefix('\u{feff}').unwrap_or(content);
    let trimmed = without_bom.trim_start();
    let open_at = content.len() - trimmed.len();
    let after_fence = trimmed.strip_prefix(fence)?;
    // `\s*\n`: only whitespace may follow the opening fence on its line.
    let newline = after_fence.find('\n')?;
    if !after_fence[..newline].trim().is_empty() {
        return None;
    }
    Some((open_at, open_at + fence.len() + newline + 1))
}

/// Where the body starts once the closing fence ends at `after_close`:
/// YAML skips the whitespace that follows the fence, TOML does not.
fn body_start(
    content: &str,
    after_close: usize,
    format: FrontMatterFormat,
) -> usize {
    if format != FrontMatterFormat::Yaml {
        return after_close;
    }
    let rest = &content[after_close..];
    after_close + (rest.len() - rest.trim_start().len())
}

/// The byte offset of the first `fence` at or after `from` that starts a
/// line (only whitespace between the preceding newline and it).
fn closing_fence(
    content: &str,
    from: usize,
    fence: &str,
) -> Option<usize> {
    let finder = memmem::Finder::new(fence.as_bytes());
    let bytes = content.as_bytes();
    let mut search_from = from;
    while let Some(rel) = finder.find(&bytes[search_from..]) {
        let at = search_from + rel;
        if starts_line(bytes, from, at) {
            return Some(at);
        }
        search_from = at + 1;
    }
    None
}

/// True when every byte between `at` and the previous newline is
/// whitespace, and that newline lies at or after `from - 1` (the one
/// that ends the opening line), so `---\n---` is an empty block.
fn starts_line(bytes: &[u8], from: usize, at: usize) -> bool {
    let mut i = at;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b'\n' => return i + 1 >= from,
            b' ' | b'\t' | b'\r' | 0x0b | 0x0c => {}
            _ => return false,
        }
    }
    false
}

/// A JSON object at the top of the document (feature `json`).
#[cfg(feature = "json")]
fn json(content: &str) -> Scan<'_> {
    let trimmed = content.trim_start_matches('\u{feff}').trim_start();
    if !trimmed.starts_with('{') {
        return Scan::None;
    }
    let lead = content.len() - trimmed.len();
    let mut stream = serde_json::Deserializer::from_str(trimmed)
        .into_iter::<serde_json::Map<
        alloc::string::String,
        serde_json::Value,
    >>();
    let Some(first) = stream.next() else {
        return Scan::None;
    };
    // A syntax error still identifies the shape. The block is then the
    // rest of the document, so the caller's parse reports the parser's
    // own message and position instead of "no front matter" or an
    // empty-input error.
    let end = match first {
        Ok(_) => lead + stream.byte_offset(),
        Err(_) => content.len(),
    };
    Scan::Found {
        format: FrontMatterFormat::Json,
        raw: &content[lead..end],
        body_offset: end,
    }
}

/// Without the `json` feature a leading `{` is reported as unsupported.
#[cfg(not(feature = "json"))]
fn json(content: &str) -> Scan<'_> {
    let trimmed = content.trim_start_matches('\u{feff}').trim_start();
    if trimmed.starts_with('{') {
        Scan::Unsupported(FrontMatterFormat::Json)
    } else {
        Scan::None
    }
}

#[cfg(all(test, feature = "yaml", feature = "toml"))]
mod tests {
    use super::*;

    #[test]
    fn yaml_block_body_after_whitespace() {
        let s = scan("---\ntitle: a\n---\n\nbody");
        assert_eq!(
            s,
            Scan::Found {
                format: FrontMatterFormat::Yaml,
                raw: "title: a",
                body_offset: 18
            }
        );
    }

    #[test]
    fn toml_body_right_after_fence() {
        let s = scan("+++\na = 1\n+++\nbody");
        assert_eq!(
            s,
            Scan::Found {
                format: FrontMatterFormat::Toml,
                raw: "a = 1",
                body_offset: 13
            }
        );
    }

    #[test]
    fn bom_and_indented_closing_fence() {
        let s = scan("\u{feff}  ---\nk: v\n  ---  \nbody");
        assert!(matches!(s, Scan::Found { raw: "k: v", .. }));
    }

    #[test]
    fn closing_fence_must_start_a_line() {
        assert!(matches!(
            scan("---\nk: a---b\n---\n"),
            Scan::Found {
                raw: "k: a---b",
                ..
            }
        ));
        assert_eq!(
            scan("---\nk: v---\n"),
            Scan::Unterminated {
                format: FrontMatterFormat::Yaml,
                offset: 0
            }
        );
    }

    #[test]
    fn opening_line_is_not_a_closing_fence() {
        assert_eq!(
            scan("---\n"),
            Scan::Unterminated {
                format: FrontMatterFormat::Yaml,
                offset: 0
            }
        );
        assert_eq!(
            scan("---\n---\nbody"),
            Scan::Found {
                format: FrontMatterFormat::Yaml,
                raw: "",
                body_offset: 8
            }
        );
        assert!(matches!(
            scan("---\n\n---\n"),
            Scan::Found { raw: "", .. }
        ));
    }

    #[test]
    fn text_after_opening_fence_is_not_front_matter() {
        assert_eq!(scan("--- title\n---\n"), Scan::None);
        assert_eq!(scan("plain text"), Scan::None);
    }

    #[test]
    fn unterminated_reports_fence_offset() {
        assert_eq!(
            scan("  +++\na = 1\n"),
            Scan::Unterminated {
                format: FrontMatterFormat::Toml,
                offset: 2
            }
        );
    }
}

/// A format that is not compiled in is reported, not mistaken for "no
/// front matter". Run with `--no-default-features --features yaml`.
#[cfg(all(
    test,
    feature = "yaml",
    not(feature = "toml"),
    not(feature = "json")
))]
mod compiled_out_tests {
    use super::*;

    #[test]
    fn toml_fence_without_toml_feature_is_unsupported() {
        assert_eq!(
            scan("+++\na = 1\n+++\n"),
            Scan::Unsupported(FrontMatterFormat::Toml)
        );
    }

    #[test]
    fn json_object_without_json_feature_is_unsupported() {
        assert_eq!(
            scan("{\"a\": 1}\nbody"),
            Scan::Unsupported(FrontMatterFormat::Json)
        );
    }

    #[test]
    fn yaml_still_parses() {
        assert!(matches!(
            scan("---\nk: v\n---\n"),
            Scan::Found { raw: "k: v", .. }
        ));
    }
}
