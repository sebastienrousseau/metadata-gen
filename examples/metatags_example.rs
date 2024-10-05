// examples/metatags_example.rs
#![allow(missing_docs)]

use metadata_gen::{
    metatags::{
        extract_meta_tags, generate_metatags, meta_tags_to_hashmap,
        MetaTag, MetaTagGroups,
    },
    MetadataError,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 metadata-gen Meta Tags Generation and Extraction Examples\n");

    generate_metatags_example()?;
    extract_meta_tags_example()?;
    custom_meta_tags_example()?;
    meta_tags_to_hashmap_example()?;

    println!("\n🎉 All meta tags examples completed successfully!");

    Ok(())
}

fn generate_metatags_example() -> Result<(), MetadataError> {
    println!("🦀 Generate Meta Tags Example");
    println!("---------------------------------------------");

    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), "My Awesome Page".to_string());
    metadata.insert(
        "description".to_string(),
        "This is an awesome page about Rust".to_string(),
    );
    metadata.insert(
        "keywords".to_string(),
        "rust, programming, web development".to_string(),
    );
    metadata.insert(
        "og:title".to_string(),
        "Awesome Rust Page".to_string(),
    );
    metadata.insert(
        "og:description".to_string(),
        "Learn about Rust programming".to_string(),
    );
    metadata.insert(
        "twitter:card".to_string(),
        "summary_large_image".to_string(),
    );
    metadata.insert(
        "apple-mobile-web-app-title".to_string(),
        "Rust App".to_string(),
    );

    let meta_tags = generate_metatags(&metadata);

    println!("    ✅ Successfully generated meta tags");
    println!("    🏷️  Meta Tags:");
    print_meta_tag_groups(&meta_tags);

    Ok(())
}

fn extract_meta_tags_example() -> Result<(), MetadataError> {
    println!("\n🦀 Extract Meta Tags Example");
    println!("---------------------------------------------");

    let html_content = r#"
    <html>
      <head>
        <meta name="description" content="A sample page for meta tag extraction">
        <meta property="og:title" content="Sample Meta Tags">
        <meta name="keywords" content="meta tags, extraction, example">
        <meta name="twitter:card" content="summary">
        <meta http-equiv="content-type" content="text/html; charset=UTF-8">
      </head>
      <body>
        <p>Some content</p>
      </body>
    </html>
    "#;

    match extract_meta_tags(html_content) {
        Ok(meta_tags) => {
            println!("    ✅ Successfully extracted meta tags");
            println!("    🏷️  Extracted Meta Tags:");
            for tag in &meta_tags {
                println!("       {} = {}", tag.name, tag.content);
            }
        }
        Err(e) => println!("    ❌ Failed to extract meta tags: {}", e),
    }

    Ok(())
}

fn custom_meta_tags_example() -> Result<(), MetadataError> {
    println!("\n🦀 Custom Meta Tags Example");
    println!("---------------------------------------------");

    let mut meta_tag_groups = MetaTagGroups::default();

    meta_tag_groups.add_custom_tag("author", "John Doe");
    meta_tag_groups.add_custom_tag(
        "viewport",
        "width=device-width, initial-scale=1",
    );
    meta_tag_groups.add_custom_tag("og:type", "website");
    meta_tag_groups.add_custom_tag("twitter:site", "@rustlang");
    meta_tag_groups
        .add_custom_tag("apple-mobile-web-app-capable", "yes");

    println!("    ✅ Successfully added custom meta tags");
    println!("    🏷️  Custom Meta Tags:");
    print_meta_tag_groups(&meta_tag_groups);

    Ok(())
}

fn meta_tags_to_hashmap_example() -> Result<(), MetadataError> {
    println!("\n🦀 Meta Tags to HashMap Example");
    println!("---------------------------------------------");

    let meta_tags = vec![
        MetaTag {
            name: "description".to_string(),
            content: "A sample page".to_string(),
        },
        MetaTag {
            name: "og:title".to_string(),
            content: "Sample Title".to_string(),
        },
        MetaTag {
            name: "keywords".to_string(),
            content: "sample, meta tags, rust".to_string(),
        },
    ];

    let hashmap = meta_tags_to_hashmap(meta_tags);

    println!("    ✅ Successfully converted meta tags to HashMap");
    println!("    🗺️  Meta Tags HashMap:");
    for (key, value) in &hashmap {
        println!("       {} = {}", key, value);
    }

    Ok(())
}

fn print_meta_tag_groups(groups: &MetaTagGroups) {
    if !groups.primary.is_empty() {
        println!("       Primary Tags:");
        println!("       {}", groups.primary);
    }
    if !groups.og.is_empty() {
        println!("       Open Graph Tags:");
        println!("       {}", groups.og);
    }
    if !groups.twitter.is_empty() {
        println!("       Twitter Tags:");
        println!("       {}", groups.twitter);
    }
    if !groups.apple.is_empty() {
        println!("       Apple Tags:");
        println!("       {}", groups.apple);
    }
    if !groups.ms.is_empty() {
        println!("       Microsoft Tags:");
        println!("       {}", groups.ms);
    }
}
