// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Synchronous readers and files.
//!
//! The parser works on `&str`; these helpers do the reading. Any async
//! runtime can use them from a blocking task, and the Tokio-native
//! versions live in `metadata_gen::tokio` behind the `tokio` feature.

use crate::error::MetadataError;
use crate::{
    extract_and_prepare_metadata, Keywords, MetaTagGroups, MetadataMap,
};
use std::io::Read;
use std::path::Path;

/// Reads everything from `reader` and runs [`extract_and_prepare_metadata`].
///
/// An empty (or whitespace-only) input yields empty metadata rather than
/// an error, matching `metadata_gen::tokio::extract_from_file` (feature `tokio`).
///
/// # Errors
///
/// [`MetadataError::IoError`] when the read fails, otherwise whatever
/// [`extract_and_prepare_metadata`] returns.
///
/// # Example
///
/// ```
/// use metadata_gen::io::extract_from_reader;
///
/// let doc = b"---\ntitle: Reader\n---\nbody" as &[u8];
/// let (metadata, _, _) = extract_from_reader(doc).unwrap();
/// assert_eq!(metadata["title"], "Reader");
/// ```
pub fn extract_from_reader<R: Read>(
    mut reader: R,
) -> Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError> {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    extract_from_content(&content)
}

/// Reads the file at `path` and runs [`extract_and_prepare_metadata`].
///
/// # Errors
///
/// As [`extract_from_reader`].
///
/// # Security
///
/// `path` is opened as given. Validate a path that comes from untrusted
/// input before passing it, to rule out path traversal.
///
/// # Example
///
/// ```no_run
/// use metadata_gen::io::extract_from_file;
///
/// let (metadata, keywords, tags) = extract_from_file("content/post.md").unwrap();
/// # let _ = (metadata, keywords, tags);
/// ```
pub fn extract_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError> {
    extract_from_reader(std::fs::File::open(path)?)
}

/// The shared tail of every file helper: empty input is empty output.
pub(crate) fn extract_from_content(
    content: &str,
) -> Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError> {
    if content.trim().is_empty() {
        return Ok((
            MetadataMap::new(),
            Vec::new(),
            MetaTagGroups::default(),
        ));
    }
    extract_and_prepare_metadata(content)
}

#[cfg(all(test, feature = "yaml"))]
mod tests {
    use super::*;

    #[test]
    fn reader_round_trip_matches_string_api() {
        let doc = "---\ntitle: T\ndate: 2024-01-02\n---\nbody";
        let via_reader = extract_from_reader(doc.as_bytes()).unwrap();
        let direct = extract_and_prepare_metadata(doc).unwrap();
        assert_eq!(via_reader.0, direct.0);
        assert_eq!(via_reader.1, direct.1);
        assert_eq!(via_reader.2, direct.2);
    }

    #[test]
    fn empty_reader_is_empty_metadata() {
        let (map, keywords, tags) =
            extract_from_reader(b"  \n" as &[u8]).unwrap();
        assert!(map.is_empty());
        assert!(keywords.is_empty());
        assert_eq!(tags, MetaTagGroups::default());
    }

    #[test]
    fn invalid_utf8_is_an_io_error() {
        let err = extract_from_reader(&[0xff, 0xfe][..]).unwrap_err();
        assert!(matches!(err, MetadataError::IoError(_)));
    }

    #[cfg_attr(
        miri,
        ignore = "touches the filesystem; Miri isolation forbids it"
    )]
    #[test]
    fn missing_file_is_an_io_error() {
        let err =
            extract_from_file("/definitely/not/here.md").unwrap_err();
        assert!(matches!(err, MetadataError::IoError(_)));
    }
}
