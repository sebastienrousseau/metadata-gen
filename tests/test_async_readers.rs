// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The runtime-agnostic readers (issues #59 and #60): every reader gives
//! the same result as the synchronous string API.

#![cfg(all(
    feature = "yaml",
    feature = "futures-io",
    feature = "embedded-io-async"
))]

use core::future::Future;
use core::pin::{pin, Pin};
use core::task::{Context, Poll};
use metadata_gen::{extract_and_prepare_metadata, MetadataError};

/// A minimal executor: polls with a no-op waker until ready.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(std::task::Waker::noop());
    loop {
        if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
            return v;
        }
    }
}

/// A futures-io reader that hands out `step` bytes per poll and
/// returns `Pending` every other poll, as a real socket would.
struct Trickle<'a> {
    data: &'a [u8],
    step: usize,
    pending: bool,
    fail: bool,
}

impl futures_io::AsyncRead for Trickle<'_> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.fail {
            return Poll::Ready(Err(std::io::Error::other("boom")));
        }
        self.pending = !self.pending;
        if self.pending {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let n = buf.len().min(self.step).min(self.data.len());
        buf[..n].copy_from_slice(&self.data[..n]);
        self.data = &self.data[n..];
        Poll::Ready(Ok(n))
    }
}

const DOC: &str = "---\ntitle: Same\nkeywords: a, b\n---\nbody";

fn trickle(data: &[u8]) -> Trickle<'_> {
    Trickle {
        data,
        step: 3,
        pending: false,
        fail: false,
    }
}

#[test]
fn futures_io_matches_the_string_api() {
    let got = block_on(metadata_gen::futures_io::extract_from_reader(
        trickle(DOC.as_bytes()),
    ))
    .unwrap();
    assert_eq!(got, extract_and_prepare_metadata(DOC).unwrap());
}

#[test]
fn futures_io_reads_past_one_chunk() {
    let doc = format!("---\ntitle: {}\n---\n", "x".repeat(20_000));
    let mut reader = trickle(doc.as_bytes());
    reader.step = 9000;
    let got =
        block_on(metadata_gen::futures_io::extract_from_reader(reader))
            .unwrap();
    assert_eq!(got.0["title"].len(), 20_000);
}

#[test]
fn futures_io_errors() {
    let mut failing = trickle(b"");
    failing.fail = true;
    let err = block_on(metadata_gen::futures_io::extract_from_reader(
        failing,
    ))
    .unwrap_err();
    assert!(matches!(err, MetadataError::IoError(_)));
    let err = block_on(metadata_gen::futures_io::extract_from_reader(
        trickle(&[0xff, 0xfe]),
    ))
    .unwrap_err();
    assert!(matches!(err, MetadataError::IoError(_)));
    let (map, _, _) = block_on(
        metadata_gen::futures_io::extract_from_reader(trickle(b" \n")),
    )
    .unwrap();
    assert!(map.is_empty());
}

/// An embedded-io reader that fails on demand.
struct Device<'a> {
    data: &'a [u8],
    fail: bool,
}

impl embedded_io_async::ErrorType for Device<'_> {
    type Error = embedded_io_async::ErrorKind;
}

impl embedded_io_async::Read for Device<'_> {
    async fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        if self.fail {
            return Err(embedded_io_async::ErrorKind::TimedOut);
        }
        let n = buf.len().min(2).min(self.data.len());
        buf[..n].copy_from_slice(&self.data[..n]);
        self.data = &self.data[n..];
        Ok(n)
    }
}

#[test]
fn embedded_reader_matches_the_string_api() {
    let mut buffer = [0_u8; 128];
    let device = Device {
        data: DOC.as_bytes(),
        fail: false,
    };
    let got = block_on(metadata_gen::embedded_io::extract_from_reader(
        device,
        &mut buffer,
    ))
    .unwrap();
    assert_eq!(got, extract_and_prepare_metadata(DOC).unwrap());
}

#[test]
fn embedded_reader_errors() {
    let run = |data: &[u8], fail: bool, size: usize| {
        let mut buffer = vec![0_u8; size];
        block_on(metadata_gen::embedded_io::extract_from_reader(
            Device { data, fail },
            &mut buffer,
        ))
    };
    let small = run(DOC.as_bytes(), false, 8).unwrap_err();
    assert!(small.to_string().contains("8-byte buffer"), "{small}");
    let failed = run(DOC.as_bytes(), true, 64).unwrap_err();
    assert!(failed.to_string().contains("TimedOut"), "{failed}");
    let bad = run(&[0xff], false, 64).unwrap_err();
    assert!(bad.to_string().contains("not UTF-8"), "{bad}");
    assert!(run(b"", false, 4).unwrap().0.is_empty());
}
