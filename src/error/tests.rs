// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for `error`; a child module so private items stay reachable.

use super::*;
use std::error::Error;
use std::fmt;
use std::io;

#[test]
fn test_extraction_error() {
    let error = MetadataError::new_extraction_error(
        "No valid front matter found.",
    );
    assert_eq!(
        error.to_string(),
        "Failed to extract metadata: No valid front matter found."
    );
}

#[test]
fn test_processing_error() {
    let error = MetadataError::new_processing_error("Unknown field");
    assert_eq!(
        error.to_string(),
        "Failed to process metadata: Unknown field"
    );
}

#[test]
fn test_missing_field_error() {
    let error = MetadataError::MissingFieldError("author".to_string());
    assert_eq!(
        error.to_string(),
        "Missing required metadata field: author"
    );
}

#[test]
fn test_date_parse_error() {
    let error = MetadataError::DateParseError(
        "Invalid date format".to_string(),
    );
    assert_eq!(
        error.to_string(),
        "Failed to parse date: Invalid date format"
    );
}

#[test]
fn test_io_error() {
    let io_error =
        io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error: MetadataError = io_error.into();
    assert_eq!(error.to_string(), "I/O error: File not found");
}

#[test]
fn test_yaml_error() {
    let yaml_error = noyalib::Error::custom("YAML structure error");
    let error: MetadataError = yaml_error.into();
    assert!(error.to_string().contains("YAML parsing error"));
}

#[test]
fn test_json_error() {
    let json_error = serde_json::Error::custom("Invalid JSON format");
    let error: MetadataError = json_error.into();
    assert_eq!(
        error.to_string(),
        "JSON parsing error: Invalid JSON format"
    );
}

#[test]
fn test_toml_error() {
    let toml_error = toml::de::Error::custom("Invalid TOML structure");
    let error: MetadataError = toml_error.into();
    assert!(error.to_string().contains("TOML parsing error"));
}

#[test]
fn test_unsupported_format_error() {
    let error =
        MetadataError::UnsupportedFormatError("XML".to_string());
    assert_eq!(error.to_string(), "Unsupported metadata format: XML");
}

#[test]
fn test_validation_error() {
    let error = MetadataError::new_validation_error(
        "title",
        "Title must not be empty",
    );
    match error {
        MetadataError::ValidationError { field, message } => {
            assert_eq!(field, "title");
            assert_eq!(message, "Title must not be empty");
        }
        _ => panic!("Unexpected error variant"),
    }
}

#[test]
#[allow(invalid_from_utf8)]
fn test_utf8_error() {
    let invalid_bytes: &[u8] = &[0xFF, 0xFF];
    let utf8_error = std::str::from_utf8(invalid_bytes).unwrap_err();
    let error: MetadataError = utf8_error.into();
    assert!(matches!(error, MetadataError::Utf8Error(..)));
    assert!(error.to_string().starts_with("UTF-8 decoding error:"));
}

#[test]
fn test_other_error() {
    use std::error::Error;

    #[derive(Debug)]
    struct CustomError;

    impl std::fmt::Display for CustomError {
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            write!(f, "Custom error occurred")
        }
    }

    impl Error for CustomError {}

    let custom_error = CustomError;
    let error = MetadataError::Other(Box::new(custom_error));

    assert!(matches!(error, MetadataError::Other(..)));
    assert_eq!(
        error.to_string(),
        "Unexpected error: Custom error occurred"
    );
}

#[test]
fn test_extraction_error_with_empty_message() {
    let error = MetadataError::new_extraction_error("");
    assert_eq!(error.to_string(), "Failed to extract metadata: ");
}

#[test]
fn test_processing_error_with_empty_message() {
    let error = MetadataError::new_processing_error("");
    assert_eq!(error.to_string(), "Failed to process metadata: ");
}

#[test]
fn test_validation_error_with_empty_field_and_message() {
    let error = MetadataError::new_validation_error("", "");
    match error {
        MetadataError::ValidationError { field, message } => {
            assert_eq!(field, "");
            assert_eq!(message, "");
        }
        _ => panic!("Unexpected error variant"),
    }
}

#[test]
fn test_unsupported_format_error_with_empty_format() {
    let error = MetadataError::UnsupportedFormatError("".to_string());
    assert_eq!(error.to_string(), "Unsupported metadata format: ");
}

#[test]
fn test_yaml_error_with_custom_message() {
    // Custom YAML error message
    let yaml_error =
        noyalib::Error::custom("Custom YAML error occurred");
    let error: MetadataError = yaml_error.into();
    assert!(error
        .to_string()
        .contains("YAML parsing error: Custom YAML error occurred"));
}

#[test]
fn test_json_error_with_custom_message() {
    // Custom JSON error message
    let json_error = serde_json::Error::custom("Custom JSON error");
    let error: MetadataError = json_error.into();
    assert_eq!(
        error.to_string(),
        "JSON parsing error: Custom JSON error"
    );
}

#[test]
fn test_toml_error_with_custom_message() {
    // Custom TOML error message
    let toml_error = toml::de::Error::custom("Custom TOML error");
    let error: MetadataError = toml_error.into();
    assert!(error
        .to_string()
        .contains("TOML parsing error: Custom TOML error"));
}

#[test]
#[allow(invalid_from_utf8)]
fn test_utf8_error_with_specific_invalid_bytes() {
    let invalid_bytes: &[u8] = &[0xC0, 0x80]; // Overlong encoding, invalid UTF-8
    let utf8_error = std::str::from_utf8(invalid_bytes).unwrap_err();
    let error: MetadataError = utf8_error.into();
    assert!(matches!(error, MetadataError::Utf8Error(..)));
    assert!(error.to_string().starts_with("UTF-8 decoding error:"));
}

#[test]
fn test_io_error_with_custom_message() {
    let io_error = std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "Permission denied",
    );
    let error: MetadataError = io_error.into();
    assert_eq!(error.to_string(), "I/O error: Permission denied");
}

#[test]
fn test_extraction_error_to_debug() {
    let error = MetadataError::new_extraction_error(
        "Failed to extract metadata",
    );
    assert_eq!(
        format!("{:?}", error),
        r#"ExtractionError { message: "Failed to extract metadata" }"#
    );
}

#[test]
fn test_processing_error_to_debug() {
    let error =
        MetadataError::new_processing_error("Processing failed");
    assert_eq!(
        format!("{:?}", error),
        r#"ProcessingError { message: "Processing failed" }"#
    );
}

#[test]
fn test_validation_error_to_debug() {
    let error = MetadataError::new_validation_error(
        "title",
        "Title cannot be empty",
    );
    assert_eq!(
        format!("{:?}", error),
        r#"ValidationError { field: "title", message: "Title cannot be empty" }"#
    );
}

#[test]
fn test_other_error_to_debug() {
    #[derive(Debug)]
    struct CustomError;

    impl std::fmt::Display for CustomError {
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            write!(f, "A custom error occurred")
        }
    }

    impl std::error::Error for CustomError {}

    let custom_error = CustomError;
    let error = MetadataError::Other(Box::new(custom_error));

    // Ensure the debug output is correctly formatted
    assert!(format!("{:?}", error).contains("Other("));
}

#[test]
fn test_context_error() {
    let error =
        MetadataError::new_extraction_error("Failed to parse YAML")
            .context("Processing file 'example.md'");
    assert_eq!(
        error.to_string(),
        "Failed to extract metadata: Processing file 'example.md': Failed to parse YAML"
    );
}

#[test]
fn test_nested_context_error() {
    let error =
        MetadataError::new_extraction_error("Failed to parse YAML")
            .context("Processing file 'example.md'")
            .context("Metadata extraction process");
    assert_eq!(
        error.to_string(),
        "Failed to extract metadata: Metadata extraction process: Processing file 'example.md': Failed to parse YAML"
    );
}

#[test]
fn test_extraction_error_empty_message() {
    let error = MetadataError::ExtractionError {
        message: "".to_string(),
    };
    assert_eq!(error.to_string(), "Failed to extract metadata: ");
}

#[test]
fn test_processing_error_empty_message() {
    let error = MetadataError::ProcessingError {
        message: "".to_string(),
    };
    assert_eq!(error.to_string(), "Failed to process metadata: ");
}

#[test]
fn test_missing_field_error_empty_message() {
    let error = MetadataError::MissingFieldError("".to_string());
    assert_eq!(error.to_string(), "Missing required metadata field: ");
}

#[test]
fn test_date_parse_error_empty_message() {
    let error = MetadataError::DateParseError("".to_string());
    assert_eq!(error.to_string(), "Failed to parse date: ");
}

#[test]
fn test_extraction_error_debug() {
    let error = MetadataError::ExtractionError {
        message: "Error extracting metadata".to_string(),
    };
    // The correct Debug output for the struct variant should include the field name
    assert_eq!(
        format!("{:?}", error),
        r#"ExtractionError { message: "Error extracting metadata" }"#
    );
}

#[test]
fn test_processing_error_debug() {
    let error = MetadataError::ProcessingError {
        message: "Error processing metadata".to_string(),
    };
    // The correct Debug output for the struct variant should include the field name
    assert_eq!(
        format!("{:?}", error),
        r#"ProcessingError { message: "Error processing metadata" }"#
    );
}

#[test]
fn test_io_error_propagation() {
    let io_error =
        io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error: MetadataError = io_error.into();
    assert_eq!(error.to_string(), "I/O error: file not found");
    assert!(matches!(error, MetadataError::IoError(_)));
}

#[test]
fn test_yaml_error_propagation() {
    let yaml_error = noyalib::Error::custom("Custom YAML error");
    let error: MetadataError = yaml_error.into();
    assert_eq!(
        error.to_string(),
        "YAML parsing error: Custom YAML error"
    );
    assert!(matches!(error, MetadataError::YamlError(_)));
}

#[test]
fn test_json_error_propagation() {
    let json_error = serde_json::Error::custom("Custom JSON error");
    let error: MetadataError = json_error.into();
    assert_eq!(
        error.to_string(),
        "JSON parsing error: Custom JSON error"
    );
    assert!(matches!(error, MetadataError::JsonError(_)));
}

#[test]
fn test_toml_error_propagation() {
    let toml_error = toml::de::Error::custom("Custom TOML error");
    let error: MetadataError = toml_error.into();
    assert_eq!(
        error.to_string(),
        "TOML parsing error: Custom TOML error\n"
    );
    assert!(matches!(error, MetadataError::TomlError(_)));
}

#[test]
fn test_missing_field_error_debug() {
    let error = MetadataError::MissingFieldError("title".to_string());
    assert_eq!(format!("{:?}", error), r#"MissingFieldError("title")"#);
}

#[test]
fn test_date_parse_error_debug() {
    let error = MetadataError::DateParseError(
        "Invalid date format".to_string(),
    );
    assert_eq!(
        format!("{:?}", error),
        r#"DateParseError("Invalid date format")"#
    );
}

#[test]
fn test_empty_yaml_error_message() {
    let yaml_error = noyalib::Error::custom("");
    let error: MetadataError = yaml_error.into();
    assert_eq!(error.to_string(), "YAML parsing error: ");
}

#[test]
fn test_empty_json_error_message() {
    let json_error = serde_json::Error::custom("");
    let error: MetadataError = json_error.into();
    assert_eq!(error.to_string(), "JSON parsing error: ");
}

#[test]
fn test_empty_toml_error_message() {
    let toml_error = toml::de::Error::custom("");
    let error: MetadataError = toml_error.into();
    assert_eq!(error.to_string(), "TOML parsing error: \n");
}

// A custom error for testing purposes
#[derive(Debug)]
struct CustomError;

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Custom error occurred")
    }
}

impl Error for CustomError {}

#[test]
fn test_context_error_fmt() {
    let custom_error = CustomError;
    let context_error = ContextError {
        context: "An error occurred while processing".to_string(),
        source: Box::new(custom_error),
    };

    let formatted = format!("{}", context_error);
    assert_eq!(
        formatted,
        "An error occurred while processing: Custom error occurred"
    );
}

#[test]
fn test_context_error_source() {
    let custom_error = CustomError;
    let context_error = ContextError {
        context: "Error with context".to_string(),
        source: Box::new(custom_error),
    };

    // The source method should return a reference to the original error (custom_error in this case)
    let source = context_error.source().unwrap();
    assert_eq!(source.to_string(), "Custom error occurred");
}

#[test]
fn test_context_error_debug() {
    let custom_error = CustomError;
    let context_error = ContextError {
        context: "Error during processing".to_string(),
        source: Box::new(custom_error),
    };

    let debug_output = format!("{:?}", context_error);

    // Ensure the debug output includes the "ContextError" struct and its fields
    assert!(debug_output.contains("ContextError"));
    assert!(debug_output.contains("Error during processing"));
    assert!(debug_output.contains("CustomError"));
}
