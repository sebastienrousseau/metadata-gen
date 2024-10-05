//! Error types for the metadata-gen library.
//!
//! This module defines custom error types used throughout the library,
//! providing detailed information about various failure scenarios.

use serde::de::Error as SerdeError;
use serde_yml::Error as SerdeYmlError;
use std::fmt::Display;
use thiserror::Error;

/// A custom error type to add context to the `Other` variant of `MetadataError`.
///
/// This struct wraps another error and provides additional context information.
#[derive(Debug)]
pub struct ContextError {
    /// The context message providing additional information about the error.
    context: String,
    /// The source error that this `ContextError` is wrapping.
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.source)
    }
}

/// Custom error types for the metadata-gen library.
///
/// This enum encompasses all possible errors that can occur during
/// metadata extraction, processing, and related operations.
#[derive(Error, Debug)]
pub enum MetadataError {
    /// Error occurred while extracting metadata.
    #[error("Failed to extract metadata: {message}")]
    ExtractionError {
        /// A descriptive message about the extraction error.
        message: String,
    },

    /// Error occurred while processing metadata.
    #[error("Failed to process metadata: {message}")]
    ProcessingError {
        /// A descriptive message about the processing error.
        message: String,
    },

    /// Error occurred due to missing required field.
    #[error("Missing required metadata field: {0}")]
    MissingFieldError(String),

    /// Error occurred while parsing date.
    #[error("Failed to parse date: {0}")]
    DateParseError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// YAML parsing error.
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] SerdeYmlError),

    /// JSON parsing error.
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// TOML parsing error.
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// Unsupported metadata format error.
    #[error("Unsupported metadata format: {0}")]
    UnsupportedFormatError(String),

    /// Validation error for metadata fields.
    #[error("Metadata validation error: {field} - {message}")]
    ValidationError {
        /// The field that failed validation.
        field: String,
        /// A descriptive message about the validation error.
        message: String,
    },

    /// UTF-8 decoding error.
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Catch-all for unexpected errors.
    #[error("Unexpected error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl MetadataError {
    /// Creates a new `ExtractionError` with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - A descriptive message about the extraction error.
    ///
    /// # Returns
    ///
    /// A new `MetadataError::ExtractionError` variant.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::error::MetadataError;
    ///
    /// let error = MetadataError::new_extraction_error("Failed to extract title");
    /// assert!(matches!(error, MetadataError::ExtractionError { .. }));
    /// ```
    pub fn new_extraction_error(message: impl Into<String>) -> Self {
        Self::ExtractionError {
            message: message.into(),
        }
    }

    /// Creates a new `ProcessingError` with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - A descriptive message about the processing error.
    ///
    /// # Returns
    ///
    /// A new `MetadataError::ProcessingError` variant.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::error::MetadataError;
    ///
    /// let error = MetadataError::new_processing_error("Failed to process metadata");
    /// assert!(matches!(error, MetadataError::ProcessingError { .. }));
    /// ```
    pub fn new_processing_error(message: impl Into<String>) -> Self {
        Self::ProcessingError {
            message: message.into(),
        }
    }

    /// Creates a new `ValidationError` with the given field and message.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field that failed validation.
    /// * `message` - A descriptive message about the validation error.
    ///
    /// # Returns
    ///
    /// A new `MetadataError::ValidationError` variant.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::error::MetadataError;
    ///
    /// let error = MetadataError::new_validation_error("title", "Title must not be empty");
    /// assert!(matches!(error, MetadataError::ValidationError { .. }));
    /// ```
    pub fn new_validation_error(
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Adds context to an existing error.
    ///
    /// This method wraps the current error with additional context information.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context to add to the error.
    ///
    /// # Returns
    ///
    /// A new `MetadataError` with the added context.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::error::MetadataError;
    ///
    /// let error = MetadataError::new_extraction_error("Failed to parse YAML")
    ///     .context("Processing file 'example.md'");
    /// assert_eq!(error.to_string(), "Failed to extract metadata: Processing file 'example.md': Failed to parse YAML");
    /// ```
    pub fn context<C>(self, ctx: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        match self {
            Self::ExtractionError { message } => {
                Self::ExtractionError {
                    message: format!("{}: {}", ctx, message),
                }
            }
            Self::ProcessingError { message } => {
                Self::ProcessingError {
                    message: format!("{}: {}", ctx, message),
                }
            }
            Self::MissingFieldError(field) => {
                Self::MissingFieldError(format!("{}: {}", ctx, field))
            }
            Self::DateParseError(error) => {
                Self::DateParseError(format!("{}: {}", ctx, error))
            }
            Self::IoError(error) => Self::IoError(std::io::Error::new(
                error.kind(),
                format!("{}: {}", ctx, error),
            )),
            Self::YamlError(error) => Self::YamlError(
                SerdeYmlError::custom(format!("{}: {}", ctx, error)),
            ),
            Self::JsonError(error) => {
                Self::JsonError(serde_json::Error::custom(format!(
                    "{}: {}",
                    ctx, error
                )))
            }
            Self::TomlError(error) => Self::TomlError(
                toml::de::Error::custom(format!("{}: {}", ctx, error)),
            ),
            Self::UnsupportedFormatError(format) => {
                Self::UnsupportedFormatError(format!(
                    "{}: {}",
                    ctx, format
                ))
            }
            Self::ValidationError { field, message } => {
                Self::ValidationError {
                    field,
                    message: format!("{}: {}", ctx, message),
                }
            }
            Self::Utf8Error(error) => Self::Utf8Error(error),
            Self::Other(error) => Self::Other(Box::new(ContextError {
                context: ctx.to_string(),
                source: error,
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let error =
            MetadataError::new_processing_error("Unknown field");
        assert_eq!(
            error.to_string(),
            "Failed to process metadata: Unknown field"
        );
    }

    #[test]
    fn test_missing_field_error() {
        let error =
            MetadataError::MissingFieldError("author".to_string());
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
        let yaml_error =
            serde_yml::Error::custom("YAML structure error");
        let error: MetadataError = yaml_error.into();
        assert!(error.to_string().contains("YAML parsing error"));
    }

    #[test]
    fn test_json_error() {
        let json_error =
            serde_json::Error::custom("Invalid JSON format");
        let error: MetadataError = json_error.into();
        assert_eq!(
            error.to_string(),
            "JSON parsing error: Invalid JSON format"
        );
    }

    #[test]
    fn test_toml_error() {
        let toml_error =
            toml::de::Error::custom("Invalid TOML structure");
        let error: MetadataError = toml_error.into();
        assert!(error.to_string().contains("TOML parsing error"));
    }

    #[test]
    fn test_unsupported_format_error() {
        let error =
            MetadataError::UnsupportedFormatError("XML".to_string());
        assert_eq!(
            error.to_string(),
            "Unsupported metadata format: XML"
        );
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
        let utf8_error =
            std::str::from_utf8(invalid_bytes).unwrap_err();
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
        let error =
            MetadataError::UnsupportedFormatError("".to_string());
        assert_eq!(error.to_string(), "Unsupported metadata format: ");
    }

    #[test]
    fn test_yaml_error_with_custom_message() {
        // Custom YAML error message
        let yaml_error =
            serde_yml::Error::custom("Custom YAML error occurred");
        let error: MetadataError = yaml_error.into();
        assert!(error.to_string().contains(
            "YAML parsing error: Custom YAML error occurred"
        ));
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
        let utf8_error =
            std::str::from_utf8(invalid_bytes).unwrap_err();
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
        let error = MetadataError::ExtractionError { message: "".to_string() };
        assert_eq!(error.to_string(), "Failed to extract metadata: ");
    }

    #[test]
    fn test_processing_error_empty_message() {
        let error = MetadataError::ProcessingError { message: "".to_string()};
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
    let error = MetadataError::ExtractionError { message: "Error extracting metadata".to_string() };
    // The correct Debug output for the struct variant should include the field name
    assert_eq!(format!("{:?}", error), r#"ExtractionError { message: "Error extracting metadata" }"#);
}

#[test]
fn test_processing_error_debug() {
    let error = MetadataError::ProcessingError { message: "Error processing metadata".to_string() };
    // The correct Debug output for the struct variant should include the field name
    assert_eq!(format!("{:?}", error), r#"ProcessingError { message: "Error processing metadata" }"#);
}


    #[test]
    fn test_io_error_propagation() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error: MetadataError = io_error.into();
        assert_eq!(error.to_string(), "I/O error: file not found");
        assert!(matches!(error, MetadataError::IoError(_)));
    }

    #[test]
    fn test_yaml_error_propagation() {
        let yaml_error = serde_yml::Error::custom("Custom YAML error");
        let error: MetadataError = yaml_error.into();
        assert_eq!(error.to_string(), "YAML parsing error: Custom YAML error");
        assert!(matches!(error, MetadataError::YamlError(_)));
    }

    #[test]
    fn test_json_error_propagation() {
        let json_error = serde_json::Error::custom("Custom JSON error");
        let error: MetadataError = json_error.into();
        assert_eq!(error.to_string(), "JSON parsing error: Custom JSON error");
        assert!(matches!(error, MetadataError::JsonError(_)));
    }

    #[test]
    fn test_toml_error_propagation() {
        let toml_error = toml::de::Error::custom("Custom TOML error");
        let error: MetadataError = toml_error.into();
        assert_eq!(error.to_string(), "TOML parsing error: Custom TOML error\n");
        assert!(matches!(error, MetadataError::TomlError(_)));
    }

    #[test]
    fn test_missing_field_error_debug() {
        let error = MetadataError::MissingFieldError("title".to_string());
        assert_eq!(format!("{:?}", error), r#"MissingFieldError("title")"#);
    }

    #[test]
    fn test_date_parse_error_debug() {
        let error = MetadataError::DateParseError("Invalid date format".to_string());
        assert_eq!(format!("{:?}", error), r#"DateParseError("Invalid date format")"#);
    }

    #[test]
    fn test_empty_yaml_error_message() {
        let yaml_error = serde_yml::Error::custom("");
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
}
