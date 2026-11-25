// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Reading front matter from a file on the Tokio runtime (feature
//! `tokio`, on by default).
//!
//! `metadata_gen::tokio::extract_from_file` opens the file with
//! `tokio::fs` and reads it without blocking the runtime;
//! `extract_from_reader` takes any `tokio::io::AsyncRead`. Run with
//! `cargo run --example tokio_example`.

use metadata_gen::MetadataError;

const DOC: &str = "---\n\
title: Hello from Tokio\n\
description: Front matter read without blocking\n\
keywords: rust, tokio, async\n\
---\n\
# Body";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), MetadataError> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("post.md");
    tokio::fs::write(&path, DOC).await?;

    let (metadata, keywords, tags) =
        metadata_gen::tokio::extract_from_file(&path).await?;
    println!("title:    {}", metadata["title"]);
    println!("keywords: {}", keywords.join(", "));
    println!("primary:  {}", tags.primary.trim());

    // Any Tokio reader works too: here a byte slice.
    let (from_slice, _, _) =
        metadata_gen::tokio::extract_from_reader(DOC.as_bytes())
            .await?;
    assert_eq!(from_slice, metadata);
    println!("A Tokio file and a byte slice give the same metadata.");
    Ok(())
}
