// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Reading front matter from any `futures_io::AsyncRead` (feature
//! `futures-io`), with no runtime named.
//!
//! `metadata_gen::futures_io::extract_from_reader` takes the reader
//! trait smol, async-std and the `futures` crate share, so a
//! `smol::fs::File` or an `async_net::TcpStream` goes in unchanged.
//! This example needs no runtime at all: the reader is a byte slice
//! that hands out a few bytes per poll and is sometimes not ready, as
//! a socket would be, and `block_on` is a ten-line executor. In an
//! application, use your runtime's (`smol::block_on`). Run with
//! `cargo run --example futures_io_example --features futures-io`.

use core::future::Future;
use core::pin::{pin, Pin};
use core::task::{Context, Poll, Waker};
use metadata_gen::MetadataError;

const DOC: &str = "---\n\
title: Hello from any runtime\n\
keywords: rust, smol, futures-io\n\
---\n\
# Body";

/// Polls `future` until it is ready. A real executor parks the thread
/// until the waker fires; a no-op waker and a busy loop are enough for
/// a reader that always wakes itself.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
    }
}

/// A reader that returns at most 7 bytes per read and is not ready on
/// every other poll.
struct Trickle<'a> {
    data: &'a [u8],
    ready: bool,
}

impl futures_io::AsyncRead for Trickle<'_> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        self.ready = !self.ready;
        if !self.ready {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let n = buf.len().min(7).min(self.data.len());
        buf[..n].copy_from_slice(&self.data[..n]);
        self.data = &self.data[n..];
        Poll::Ready(Ok(n))
    }
}

fn main() -> Result<(), MetadataError> {
    let reader = Trickle {
        data: DOC.as_bytes(),
        ready: false,
    };
    let (metadata, keywords, _) = block_on(
        metadata_gen::futures_io::extract_from_reader(reader),
    )?;
    println!("title:    {}", metadata["title"]);
    println!("keywords: {}", keywords.join(", "));

    let (sync, _, _) = metadata_gen::extract_and_prepare_metadata(DOC)?;
    assert_eq!(sync, metadata);
    println!("The async reader and the string API agree.");
    Ok(())
}
