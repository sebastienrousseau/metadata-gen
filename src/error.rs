//! Error types for the metadata-gen library.
//!
//! This module defines custom error types used throughout the library,
//! providing detailed information about various failure scenarios.

use serde_yml::Error as SerdeYmlError;
use thiserror::Error;

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
    pub fn new_extraction_error(message: impl Into<String>) -> Self {
        MetadataError::ExtractionError {
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
    pub fn new_processing_error(message: impl Into<String>) -> Self {
        MetadataError::ProcessingError {
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
    pub fn new_validation_error(
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        MetadataError::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_error() {
        let error = MetadataError::ExtractionError {
            message: "No valid front matter found.".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to extract metadata: No valid front matter found."
        );
    }

    #[test]
    fn test_processing_error() {
        let error = MetadataError::ProcessingError {
            message: "Unknown field".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to process metadata: Unknown field"
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
    fn test_error_display() {
        let error =
            MetadataError::MissingFieldError("title".to_string());
        assert_eq!(
            error.to_string(),
            "Missing required metadata field: title"
        );
    }
}
