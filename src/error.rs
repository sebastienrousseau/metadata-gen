//! Error types for the metadata-gen library.
//!
//! This module defines custom error types used throughout the library,
//! providing detailed information about various failure scenarios.

use noyalib::Error as SerdeYmlError;
use serde::de::Error as SerdeError;
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

/// Displays the context error as `"context: source"`.
impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

/// Provides access to the underlying source error.
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
            Self::Utf8Error(error) => Self::Utf8Error(error),
            Self::Other(error) => Self::Other(Box::new(ContextError {
                context: ctx.to_string(),
                source: error,
            })),
            other => other.prefix_message(&ctx),
        }
    }

    /// Prefixes `ctx` onto the variants whose payload is a plain message.
    /// Variants that wrap a foreign error are handled by [`Self::context`].
    fn prefix_message<C: Display>(self, ctx: &C) -> Self {
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
            other => other,
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod context_tests;
