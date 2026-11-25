// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Issue #59: smol's file type is a `futures_io::AsyncRead`, so it goes
//! straight into `metadata_gen::futures_io::extract_from_reader` with
//! no adapter, and gives what the synchronous API gives for the same
//! file. `smol::fs::File` is `async_fs::File` and `smol::block_on` is
//! `futures_lite::future::block_on`; the test names those crates so it
//! does not depend on the rest of smol.

#![cfg(all(feature = "yaml", feature = "futures-io"))]

use std::io::Write as _;

const DOC: &str = "---\ntitle: Same\nkeywords: a, b\n---\nbody";

#[test]
fn smol_file_matches_the_sync_api() {
    let mut fixture = tempfile::NamedTempFile::new().unwrap();
    fixture.write_all(DOC.as_bytes()).unwrap();
    let path = fixture.path().to_path_buf();

    let got = futures_lite::future::block_on(async {
        let file = async_fs::File::open(&path).await.unwrap();
        metadata_gen::futures_io::extract_from_reader(file)
            .await
            .unwrap()
    });

    let sync = metadata_gen::io::extract_from_file(&path).unwrap();
    assert_eq!(got, sync);
    assert_eq!(got.0["title"], "Same");
}
