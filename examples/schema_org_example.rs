// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Schema.org JSON-LD from front matter (issue #10).
//!
//! Run with `cargo run --example schema_org_example`.

use metadata_gen::schema_org::{
    breadcrumb_list, json_ld, json_ld_script, SchemaType,
};
use metadata_gen::{extract_metadata, MetadataError};

fn main() -> Result<(), MetadataError> {
    let post = "---\ntitle: Hello\ndescription: First post\n\
                author: Ada\ndate: 2026-01-02\n\
                permalink: https://example.com/hello/\n---\nBody";
    let metadata = extract_metadata(post)?.into_inner();
    println!(
        "{}",
        json_ld_script(&json_ld(SchemaType::Article, &metadata))
    );
    println!(
        "{}",
        json_ld_script(&breadcrumb_list(&[
            ("Home", "https://example.com/"),
            ("Hello", "https://example.com/hello/"),
        ]))
    );
    Ok(())
}
