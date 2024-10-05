// examples/metadata_example.rs
#![allow(missing_docs)]

use metadata_gen::{
    metadata::{extract_metadata, process_metadata, Metadata},
    MetadataError,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 metadata-gen Metadata Extraction and Processing Examples\n");

    yaml_metadata_example()?;
    toml_metadata_example()?;
    json_metadata_example()?;
    complex_metadata_example()?;
    metadata_processing_example()?;

    println!("\n🎉 All metadata examples completed successfully!");

    Ok(())
}

fn yaml_metadata_example() -> Result<(), MetadataError> {
    println!("🦀 YAML Metadata Extraction Example");
    println!("---------------------------------------------");

    let yaml_content = r#"---
title: YAML Test Page
description: A test page for YAML metadata extraction
date: 2023-05-20
tags:
  - yaml
  - metadata
  - test
---
# Content starts here
"#;

    match extract_metadata(yaml_content) {
        Ok(metadata) => {
            println!("    ✅ Successfully extracted YAML metadata");
            print_metadata(&metadata);
        }
        Err(e) => {
            println!("    ❌ Failed to extract YAML metadata: {}", e)
        }
    }

    Ok(())
}

fn toml_metadata_example() -> Result<(), MetadataError> {
    println!("\n🦀 TOML Metadata Extraction Example");
    println!("---------------------------------------------");

    let toml_content = r#"+++
title = "TOML Test Page"
description = "A test page for TOML metadata extraction"
date = 2023-05-21
tags = ["toml", "metadata", "test"]
+++
# Content starts here
"#;

    match extract_metadata(toml_content) {
        Ok(metadata) => {
            println!("    ✅ Successfully extracted TOML metadata");
            print_metadata(&metadata);
        }
        Err(e) => {
            println!("    ❌ Failed to extract TOML metadata: {}", e)
        }
    }

    Ok(())
}

fn json_metadata_example() -> Result<(), MetadataError> {
    println!("\n🦀 JSON Metadata Extraction Example");
    println!("---------------------------------------------");

    let json_content = r#"{
"title": "JSON Test Page",
"description": "A test page for JSON metadata extraction",
"date": "2023-05-22",
"tags": ["json", "metadata", "test"]
}
# Content starts here
"#;

    match extract_metadata(json_content) {
        Ok(metadata) => {
            println!("    ✅ Successfully extracted JSON metadata");
            print_metadata(&metadata);
        }
        Err(e) => {
            println!("    ❌ Failed to extract JSON metadata: {}", e)
        }
    }

    Ok(())
}

fn complex_metadata_example() -> Result<(), MetadataError> {
    println!("\n🦀 Complex Metadata Extraction Example");
    println!("---------------------------------------------");

    let complex_content = r#"---
title: Complex Metadata Test
date: 2023-05-23
author:
  name: John Doe
  email: john@example.com
nested:
  level1:
    level2: deep value
tags:
  - complex
  - nested
  - metadata
---
# Content starts here
"#;

    match extract_metadata(complex_content) {
        Ok(metadata) => {
            println!("    ✅ Successfully extracted complex metadata");
            print_metadata(&metadata);
        }
        Err(e) => {
            println!("    ❌ Failed to extract complex metadata: {}", e)
        }
    }

    Ok(())
}

fn metadata_processing_example() -> Result<(), MetadataError> {
    println!("\n🦀 Metadata Processing Example");
    println!("---------------------------------------------");

    let mut raw_metadata = HashMap::new();
    raw_metadata
        .insert("title".to_string(), "Processing Test".to_string());
    raw_metadata
        .insert("date".to_string(), "2023-05-24T12:00:00Z".to_string());
    raw_metadata.insert(
        "description".to_string(),
        "Testing metadata processing".to_string(),
    );

    let metadata = Metadata::new(raw_metadata);

    match process_metadata(&metadata) {
        Ok(processed) => {
            println!("    ✅ Successfully processed metadata");
            print_metadata(&processed);
            if let Some(slug) = processed.get("slug") {
                println!("    📎 Generated slug: {}", slug);
            }
        }
        Err(e) => println!("    ❌ Failed to process metadata: {}", e),
    }

    Ok(())
}

fn print_metadata(metadata: &Metadata) {
    println!("    📊 Extracted Metadata:");
    for (key, value) in metadata.clone().into_inner().iter() {
        println!("       {}: {}", key, value);
    }
}
