//! The high-level flow: `extract_and_prepare_metadata` and the meta-tag groups it returns.
//!
//! Run with `cargo run --example lib_example`.

// examples/lib_example.rs
#![allow(missing_docs)]

use metadata_gen::{
    extract_and_prepare_metadata, generate_metatags,
    utils::{async_extract_metadata_from_file, escape_html},
    MetadataError,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 metadata-gen Library Usage Examples\n");

    extract_and_prepare_example()?;
    generate_metatags_example()?;
    escape_html_example()?;
    async_file_extraction_example().await?;

    println!("\n🎉 All library usage examples completed successfully!");

    Ok(())
}

fn extract_and_prepare_example() -> Result<(), MetadataError> {
    println!("🦀 Extract and Prepare Metadata Example");
    println!("---------------------------------------------");

    let content = r#"---
title: Sample Page
description: This is a sample page for metadata extraction
keywords: metadata, example, rust
---
# Sample Content
This is the main content of the page."#;

    match extract_and_prepare_metadata(content) {
        Ok((metadata, keywords, meta_tags)) => {
            println!(
                "    ✅ Successfully extracted and prepared metadata"
            );
            println!("    📊 Metadata:");
            for (key, value) in metadata.iter() {
                println!("       {}: {}", key, value);
            }
            println!("    🏷️  Keywords: {}", keywords.join(", "));
            println!("    🏷️  Meta Tags:");
            println!("       {}", meta_tags);
        }
        Err(e) => println!(
            "    ❌ Failed to extract and prepare metadata: {}",
            e
        ),
    }

    Ok(())
}

fn generate_metatags_example() -> Result<(), MetadataError> {
    println!("\n🦀 Generate Meta Tags Example");
    println!("---------------------------------------------");

    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), "My Awesome Page".to_string());
    metadata.insert(
        "description".to_string(),
        "This is an awesome page about Rust".to_string(),
    );
    metadata.insert(
        "og:image".to_string(),
        "https://example.com/image.jpg".to_string(),
    );

    let meta_tags = generate_metatags(&metadata);
    println!("    ✅ Successfully generated meta tags");
    println!("    🏷️  Meta Tags:");
    println!("       {}", meta_tags);

    Ok(())
}

fn escape_html_example() -> Result<(), MetadataError> {
    println!("\n🦀 Escape HTML Example");
    println!("---------------------------------------------");

    let unescaped = r#"<script>alert("XSS");</script>"#;
    let escaped = escape_html(unescaped);

    println!("    ✅ Successfully escaped HTML");
    println!("    🔒 Original: {}", unescaped);
    println!("    🔐 Escaped:  {}", escaped);

    Ok(())
}

async fn async_file_extraction_example() -> Result<(), MetadataError> {
    println!("\n🦀 Async File Metadata Extraction Example");
    println!("---------------------------------------------");

    // Note: This example assumes a file named "example.md" exists in the current directory
    // Create this file or change the path to an existing file for the example to work
    match async_extract_metadata_from_file("./examples/example.md")
        .await
    {
        Ok((metadata, keywords, meta_tags)) => {
            println!(
                "    ✅ Successfully extracted metadata from file"
            );
            println!("    📊 Metadata:");
            for (key, value) in metadata.iter() {
                println!("       {}: {}", key, value);
            }
            println!("    🏷️  Keywords: {}", keywords.join(", "));
            println!("    🏷️  Meta Tags:");
            println!("       {}", meta_tags);
        }
        Err(e) => println!(
            "    ❌ Failed to extract metadata from file: {}",
            e
        ),
    }

    Ok(())
}
