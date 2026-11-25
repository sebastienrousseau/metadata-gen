// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Reading front matter from an `embedded_io_async::Read` into a fixed
//! buffer (feature `embedded-io-async`), as an Embassy task would.
//!
//! On a device the reader is a UART, a flash file or a socket and the
//! executor is Embassy's. This example runs on the host instead: a
//! byte slice implements `embedded_io_async::Read`, and `block_on` is
//! a ten-line executor. The `no_std` build itself is checked by the
//! `embedded` CI job (`thumbv7em-none-eabihf`, features
//! `yaml,embedded-io-async`). Run with
//! `cargo run --example embedded_io_example --features embedded-io-async`.

use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use metadata_gen::MetadataError;

/// Polls `future` until it is ready. Reading a byte slice never waits,
/// so a no-op waker is enough here.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
    }
}

fn main() -> Result<(), MetadataError> {
    // The input lands in this buffer; nothing is allocated for it.
    let mut buffer = [0_u8; 256];
    let uart: &[u8] =
        b"---\ntitle: Sensor 7\nkeywords: device, telemetry\n---\n";
    let (metadata, keywords, _) =
        block_on(metadata_gen::embedded_io::extract_from_reader(
            uart,
            &mut buffer,
        ))?;
    println!("title:    {}", metadata["title"]);
    println!("keywords: {}", keywords.join(", "));

    // Input larger than the buffer is an error, not a truncation.
    let mut small = [0_u8; 16];
    let err = block_on(metadata_gen::embedded_io::extract_from_reader(
        uart, &mut small,
    ))
    .unwrap_err();
    println!("16-byte buffer: {err}");
    Ok(())
}
