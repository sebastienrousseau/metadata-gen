// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Configuring which front-matter fields are required (issue #45).
//!
//! Run with `cargo run --example required_fields_example`.

use metadata_gen::{
    extract_metadata, MetadataError, MetadataProcessor,
};

fn main() -> Result<(), MetadataError> {
    // A blog post must carry an author on top of the default title and
    // date; `description` is known but optional.
    let processor = MetadataProcessor::new()
        .require("author")
        .optional("description");

    let good =
        "---\ntitle: Hello\ndate: 2026-01-02\nauthor: Ada\n---\nBody";
    let processed = processor.process(&extract_metadata(good)?)?;
    println!(
        "accepted: slug = {}",
        processed.get("slug").map_or("", |s| s)
    );

    let missing =
        "---\ntitle: Hello\ndate: 2026-01-02\ndraft: true\n---\n";
    let metadata = extract_metadata(missing)?;
    match processor.process(&metadata) {
        Err(MetadataError::MissingFieldError(field)) => {
            println!("rejected: missing required field `{field}`");
        }
        other => println!("unexpected: {other:?}"),
    }
    println!(
        "unknown fields: {:?}",
        processor.unknown_fields(&metadata)
    );
    Ok(())
}
