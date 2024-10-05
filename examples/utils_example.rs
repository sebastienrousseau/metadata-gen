// examples/utils_example.rs
#![allow(missing_docs)]

use metadata_gen::{
    utils::{
        async_extract_metadata_from_file, escape_html, unescape_html,
    },
    MetadataError,
};
use tempfile::tempdir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 metadata-gen Utility Functions Examples\n");

    html_escaping_example()?;
    html_unescaping_example()?;
    escape_unescape_roundtrip_example()?;
    async_file_metadata_extraction_example().await?;

    println!(
        "\n🎉 All utility function examples completed successfully!"
    );

    Ok(())
}

fn html_escaping_example() -> Result<(), MetadataError> {
    println!("🦀 HTML Escaping Example");
    println!("---------------------------------------------");

    let unescaped =
        r#"<script>alert("XSS");</script> & "quotes" & 'apostrophes'"#;
    let escaped = escape_html(unescaped);

    println!("    ✅ Successfully escaped HTML");
    println!("    🔓 Original: {}", unescaped);
    println!("    🔒 Escaped:  {}", escaped);

    Ok(())
}

fn html_unescaping_example() -> Result<(), MetadataError> {
    println!("\n🦀 HTML Unescaping Example");
    println!("---------------------------------------------");

    let escaped = r#"&lt;div&gt;Hello, &amp; welcome!&lt;/div&gt;"#;
    let unescaped = unescape_html(escaped);

    println!("    ✅ Successfully unescaped HTML");
    println!("    🔒 Original: {}", escaped);
    println!("    🔓 Unescaped: {}", unescaped);

    Ok(())
}

fn escape_unescape_roundtrip_example() -> Result<(), MetadataError> {
    println!("\n🦀 HTML Escape/Unescape Roundtrip Example");
    println!("---------------------------------------------");

    let original =
        r#"<p>Test & "verify" 'roundtrip' functionality</p>"#;
    let escaped = escape_html(original);
    let roundtrip = unescape_html(&escaped);

    println!("    ✅ Successfully performed escape/unescape roundtrip");
    println!("    🔓 Original:  {}", original);
    println!("    🔒 Escaped:   {}", escaped);
    println!("    🔓 Roundtrip: {}", roundtrip);

    assert_eq!(
        original, roundtrip,
        "Roundtrip failed: original and final strings don't match"
    );
    println!("    ✅ Roundtrip assertion passed");

    Ok(())
}

async fn async_file_metadata_extraction_example(
) -> Result<(), MetadataError> {
    println!("\n🦀 Async File Metadata Extraction Example");
    println!("---------------------------------------------");

    // Create a temporary directory and file
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_metadata.md");

    // Write test content to the file
    let content = r#"---
title: Test Metadata Page
description: A test page for async metadata extraction
keywords: async, metadata, extraction, example
---
# Test Content
This is a test file for asynchronous metadata extraction."#;

    let mut file = File::create(&file_path).await.unwrap();
    file.write_all(content.as_bytes()).await.unwrap();

    // Extract metadata from the file
    match async_extract_metadata_from_file(file_path.to_str().unwrap())
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
