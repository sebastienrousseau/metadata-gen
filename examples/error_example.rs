// src/examples/error_example.rs
#![allow(missing_docs)]

use metadata_gen::{
    error::MetadataError, extract_and_prepare_metadata,
    utils::async_extract_metadata_from_file,
};

/// Entry point for the metadata-gen error handling examples.
///
/// This function runs various examples demonstrating error creation, conversion,
/// and handling for different scenarios in the metadata-gen library.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 metadata-gen Error Handling Examples\n");

    extraction_error_example()?;
    processing_error_example()?;
    missing_field_error_example()?;
    date_parse_error_example()?;
    yaml_error_example()?;
    json_error_example()?;
    toml_error_example()?;
    unsupported_format_error_example()?;
    validation_error_example()?;
    io_error_example().await?;

    println!(
        "\n🎉 All error handling examples completed successfully!"
    );

    Ok(())
}

/// Demonstrates handling of extraction errors.
fn extraction_error_example() -> Result<(), MetadataError> {
    println!("🦀 Extraction Error Example");
    println!("---------------------------------------------");

    let invalid_content = "This content has no metadata";
    match extract_and_prepare_metadata(invalid_content) {
        Ok(_) => {
            println!("    ❌ Unexpected success in extracting metadata")
        }
        Err(e) => {
            println!(
                "    ✅ Successfully caught Extraction Error: {}",
                e
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of processing errors.
fn processing_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 Processing Error Example");
    println!("---------------------------------------------");

    let error = MetadataError::new_processing_error(
        "Failed to process metadata",
    );
    println!("    ✅ Created Processing Error: {}", error);

    Ok(())
}

/// Demonstrates handling of missing field errors.
fn missing_field_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 Missing Field Error Example");
    println!("---------------------------------------------");

    let error = MetadataError::MissingFieldError("title".to_string());
    println!("    ✅ Created Missing Field Error: {}", error);

    Ok(())
}

/// Demonstrates handling of date parse errors.
fn date_parse_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 Date Parse Error Example");
    println!("---------------------------------------------");

    let error = MetadataError::DateParseError(
        "Invalid date format".to_string(),
    );
    println!("    ✅ Created Date Parse Error: {}", error);

    Ok(())
}

/// Demonstrates handling of YAML parsing errors.
fn yaml_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 YAML Error Example");
    println!("---------------------------------------------");

    let invalid_yaml = "invalid: yaml: data";
    let result: Result<serde_yml::Value, _> =
        serde_yml::from_str(invalid_yaml);

    match result {
        Ok(_) => println!(
            "    ❌ Unexpected success in parsing invalid YAML"
        ),
        Err(e) => {
            let error = MetadataError::YamlError(e);
            println!(
                "    ✅ Successfully caught YAML Error: {}",
                error
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of JSON parsing errors.
fn json_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 JSON Error Example");
    println!("---------------------------------------------");

    let invalid_json = "{ invalid json }";
    match serde_json::from_str::<serde_json::Value>(invalid_json) {
        Ok(_) => println!(
            "    ❌ Unexpected success in parsing invalid JSON"
        ),
        Err(e) => {
            let error = MetadataError::JsonError(e);
            println!(
                "    ✅ Successfully caught JSON Error: {}",
                error
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of TOML parsing errors.
fn toml_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 TOML Error Example");
    println!("---------------------------------------------");

    let invalid_toml = "invalid = toml data";
    match toml::from_str::<toml::Value>(invalid_toml) {
        Ok(_) => println!(
            "    ❌ Unexpected success in parsing invalid TOML"
        ),
        Err(e) => {
            let error = MetadataError::TomlError(e);
            println!("    ✅ Successfully caught TOML Error:");
            println!("    {}", error);

            // Print additional details about the error
            if let MetadataError::TomlError(ref toml_error) = error {
                println!("\n    📍 Error details:");
                // Split the error message into lines for better formatting
                for line in toml_error.to_string().lines() {
                    println!("       {}", line);
                }

                // Add a suggestion
                println!("\n    💡 Suggestion:");
                println!("       Try enclosing the value in quotes:");
                println!("       invalid = \"toml data\"");
            }
        }
    }

    Ok(())
}

/// Demonstrates handling of unsupported format errors.
fn unsupported_format_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 Unsupported Format Error Example");
    println!("---------------------------------------------");

    let error =
        MetadataError::UnsupportedFormatError("XML".to_string());
    println!("    ✅ Created Unsupported Format Error: {}", error);

    Ok(())
}

/// Demonstrates handling of validation errors.
fn validation_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 Validation Error Example");
    println!("---------------------------------------------");

    let error = MetadataError::new_validation_error(
        "title",
        "Title must not be empty",
    );
    println!("    ✅ Created Validation Error: {}", error);

    Ok(())
}

/// Demonstrates handling of I/O errors.
async fn io_error_example() -> Result<(), MetadataError> {
    println!("\n🦀 I/O Error Example");
    println!("---------------------------------------------");

    match async_extract_metadata_from_file("nonexistent_file.md").await
    {
        Ok(_) => println!(
            "    ❌ Unexpected success in reading nonexistent file"
        ),
        Err(e) => {
            println!("    ✅ Successfully caught I/O Error: {}", e);
        }
    }

    Ok(())
}
