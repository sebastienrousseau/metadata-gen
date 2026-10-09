// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Async file helpers on the Tokio runtime (feature `tokio`).
//!
//! Other runtimes do not need an adapter: read the file with your own
//! runtime and hand the text to [`crate::extract_and_prepare_metadata`],
//! or wrap [`crate::io::extract_from_file`] in a blocking task.

use crate::error::MetadataError;
use crate::io::extract_from_content;
use crate::{Keywords, MetaTagGroups, MetadataMap};
use ::tokio::io::{AsyncRead, AsyncReadExt};
use std::path::Path;

/// Reads everything from `reader` and runs [`crate::extract_and_prepare_metadata`].
///
/// # Errors
///
/// [`MetadataError::IoError`] when the read fails, otherwise whatever
/// [`crate::extract_and_prepare_metadata`] returns.
///
/// # Example
///
/// ```
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// use metadata_gen::tokio::extract_from_reader;
///
/// let doc = b"---\ntitle: Async\n---\n" as &[u8];
/// let (metadata, _, _) = extract_from_reader(doc).await.unwrap();
/// assert_eq!(metadata["title"], "Async");
/// # });
/// ```
pub async fn extract_from_reader<R: AsyncRead + Unpin>(
    mut reader: R,
) -> Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError> {
    let mut content = String::new();
    reader.read_to_string(&mut content).await?;
    extract_from_content(&content)
}

/// Reads the file at `path` asynchronously and runs
/// [`crate::extract_and_prepare_metadata`].
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
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// use metadata_gen::tokio::extract_from_file;
///
/// let (metadata, keywords, tags) = extract_from_file("content/post.md").await.unwrap();
/// # let _ = (metadata, keywords, tags);
/// # });
/// ```
pub async fn extract_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError> {
    let file = ::tokio::fs::File::open(path).await?;
    extract_from_reader(file).await
}
