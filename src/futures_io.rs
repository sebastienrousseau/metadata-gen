// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runtime-agnostic async reading (feature `futures-io`).
//!
//! [`extract_from_reader`](crate::futures_io::extract_from_reader) takes
//! any [`AsyncRead`](::futures_io::AsyncRead), the trait
//! smol, async-std and the `futures` crate share: a `smol::fs::File`, an
//! `async_net::TcpStream` or an in-memory `futures::io::Cursor` all work
//! without an adapter, and no executor is named. Tokio's readers
//! implement Tokio's own trait instead; use [`crate::tokio`] (feature
//! `tokio`) or `tokio_util::compat` for those.

use crate::error::MetadataError;
use crate::io::extract_from_content;
use crate::{Keywords, MetaTagGroups, MetadataMap};
use alloc::vec::Vec;
use core::future::poll_fn;
use core::pin::Pin;
use futures_io::AsyncRead;

/// Reads everything from `reader` and runs
/// [`crate::extract_and_prepare_metadata`]; empty input is empty
/// metadata, as in [`crate::io::extract_from_reader`].
///
/// # Errors
///
/// [`MetadataError::IoError`] when a read fails or the input is not
/// UTF-8, otherwise whatever [`crate::extract_and_prepare_metadata`]
/// returns.
///
/// # Example
///
/// ```
/// use metadata_gen::futures_io::extract_from_reader;
/// # use core::pin::Pin;
/// # use core::task::{Context, Poll};
/// # struct Bytes<'a>(&'a [u8]);
/// # impl futures_io::AsyncRead for Bytes<'_> {
/// #     fn poll_read(mut self: Pin<&mut Self>, _: &mut Context<'_>, buf: &mut [u8])
/// #         -> Poll<std::io::Result<usize>> {
/// #         let n = buf.len().min(self.0.len());
/// #         buf[..n].copy_from_slice(&self.0[..n]);
/// #         self.0 = &self.0[n..];
/// #         Poll::Ready(Ok(n))
/// #     }
/// # }
/// # fn block_on<F: core::future::Future>(f: F) -> F::Output {
/// #     let mut f = core::pin::pin!(f);
/// #     let mut cx = Context::from_waker(std::task::Waker::noop());
/// #     loop { if let Poll::Ready(v) = f.as_mut().poll(&mut cx) { return v; } }
/// # }
/// // `Bytes` is any `futures_io::AsyncRead`; `block_on` is your executor's.
/// let reader = Bytes(b"---\ntitle: Async\n---\nbody");
/// let (metadata, _, _) = block_on(extract_from_reader(reader)).unwrap();
/// assert_eq!(metadata["title"], "Async");
/// ```
pub async fn extract_from_reader<R: AsyncRead + Unpin>(
    mut reader: R,
) -> Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let n = poll_fn(|cx| {
            Pin::new(&mut reader).poll_read(cx, &mut chunk)
        })
        .await?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    let content =
        alloc::string::String::from_utf8(bytes).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;
    extract_from_content(&content)
}
