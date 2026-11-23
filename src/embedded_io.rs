// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Async reading for embedded executors (feature `embedded-io-async`).
//!
//! Embassy and other `no_std` executors read through
//! [`embedded_io_async::Read`].
//! [`extract_from_reader`](crate::embedded_io::extract_from_reader)
//! reads into a buffer the caller provides, so it allocates nothing
//! for the input and works without `std`.

use crate::error::MetadataError;
use crate::{Keywords, MetaTagGroups, MetadataMap};
use alloc::format;
use embedded_io_async::{Error as _, Read};

/// Reads `reader` to its end into `buffer`, then extracts metadata from
/// the text as [`crate::extract_and_prepare_metadata`] does. Empty input
/// is empty metadata.
///
/// # Errors
///
/// [`MetadataError::ExtractionError`] when a read fails, the input does
/// not fit in `buffer`, or it is not UTF-8; otherwise whatever
/// [`crate::extract_and_prepare_metadata`] returns.
///
/// # Example
///
/// ```
/// use metadata_gen::embedded_io::extract_from_reader;
/// # use core::task::{Context, Poll};
/// # fn block_on<F: core::future::Future>(f: F) -> F::Output {
/// #     let mut f = core::pin::pin!(f);
/// #     let mut cx = Context::from_waker(std::task::Waker::noop());
/// #     loop { if let Poll::Ready(v) = f.as_mut().poll(&mut cx) { return v; } }
/// # }
/// // `&[u8]` implements `embedded_io_async::Read`; on a device this is a
/// // UART, a flash file or a socket, and `block_on` is the executor.
/// let mut buffer = [0_u8; 256];
/// let doc: &[u8] = b"---\ntitle: Device\n---\n";
/// let (metadata, _, _) = block_on(extract_from_reader(doc, &mut buffer)).unwrap();
/// assert_eq!(metadata.get("title").map(String::as_str), Some("Device"));
/// ```
pub async fn extract_from_reader<R: Read>(
    mut reader: R,
    buffer: &mut [u8],
) -> Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError> {
    let mut filled = 0;
    loop {
        if filled == buffer.len() {
            return Err(MetadataError::ExtractionError {
                message: format!(
                    "Input does not fit in the {}-byte buffer.",
                    buffer.len()
                ),
            });
        }
        let n =
            reader.read(&mut buffer[filled..]).await.map_err(|e| {
                MetadataError::ExtractionError {
                    message: format!("Read failed: {:?}.", e.kind()),
                }
            })?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    let content =
        core::str::from_utf8(&buffer[..filled]).map_err(|e| {
            MetadataError::ExtractionError {
                message: format!("Input is not UTF-8: {e}."),
            }
        })?;
    crate::extract_from_text(content)
}
