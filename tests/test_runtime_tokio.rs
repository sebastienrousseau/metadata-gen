// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Issue #59: a Tokio file reaches the runtime-neutral
//! `metadata_gen::futures_io::extract_from_reader` through
//! `tokio_util::compat`, and gives what the synchronous API gives for
//! the same file.

#![cfg(all(
    feature = "yaml",
    feature = "futures-io",
    feature = "tokio",
    not(loom)
))]

use std::io::Write as _;
use tokio_util::compat::TokioAsyncReadCompatExt as _;

const DOC: &str = "---\ntitle: Same\nkeywords: a, b\n---\nbody";

#[test]
#[cfg_attr(
    miri,
    ignore = "writes and reads a temp file; Miri isolation forbids it"
)]
fn tokio_file_through_compat_matches_the_sync_api() {
    let mut fixture = tempfile::NamedTempFile::new().unwrap();
    fixture.write_all(DOC.as_bytes()).unwrap();
    let path = fixture.path().to_path_buf();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let got = runtime.block_on(async {
        let file = tokio::fs::File::open(&path).await.unwrap();
        metadata_gen::futures_io::extract_from_reader(file.compat())
            .await
            .unwrap()
    });

    let sync = metadata_gen::io::extract_from_file(&path).unwrap();
    assert_eq!(got, sync);
    assert_eq!(got.0["title"], "Same");
}
