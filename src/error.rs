//! Error types for the metadata-gen library.
//!
//! This module defines custom error types used throughout the library,
//! providing detailed information about various failure scenarios.

use crate::metadata::FrontMatterFormat;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::Display;
use core::ops::Range;
#[cfg(feature = "yaml")]
use noyalib::Error as SerdeYmlError;
#[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
use serde::de::Error as SerdeError;
use thiserror::Error;

/// A boxed error that can cross threads; the payload of
/// [`MetadataError::Other`] and the `source` of [`MetadataError::Parse`].
pub type BoxedError =
    Box<dyn core::error::Error + Send + Sync + 'static>;

/// A custom error type to add context to the `Other` variant of `MetadataError`.
///
/// This struct wraps another error and provides additional context information.
#[derive(Debug)]
pub struct ContextError {
    /// The context message providing additional information about the error.
    context: String,
    /// The source error that this `ContextError` is wrapping.
    source: BoxedError,
}

/// Displays the context error as `"context: source"`.
impl core::fmt::Display for ContextError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

/// Provides access to the underlying source error.
impl core::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&*self.source)
    }
}

/// Custom error types for the metadata-gen library.
///
/// This enum encompasses all possible errors that can occur during
/// metadata extraction, processing, and related operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MetadataError {
    /// A front-matter block was found but its parser rejected it.
    ///
    /// `span` is the byte range inside the raw block the parser pointed
    /// at, when it reported one (TOML and JSON always do; YAML reports
    /// a line and column that are mapped to an offset). `source` is the
    /// parser's own error, reachable through `Error::source`.
    #[error("{format} front matter failed to parse{}: {source}", span_suffix(.span))]
    Parse {
        /// Which parser rejected the block.
        format: FrontMatterFormat,
        /// Byte range inside the raw front-matter block, if the parser gave one.
        span: Option<Range<usize>>,
        /// The parser's error.
        #[source]
        source: BoxedError,
    },

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
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// YAML parsing error.
    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] SerdeYmlError),

    /// JSON parsing error.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// TOML parsing error.
    #[cfg(feature = "toml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "toml")))]
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
    ///
    /// No function in the crate returns it: every entry point takes
    /// `&str`, and the readers in `metadata_gen::io` report invalid UTF-8 as
    /// an I/O error. Kept for code that matches on it or converts a
    /// `core::str::Utf8Error` with `?`.
    #[deprecated(
        since = "0.0.8",
        note = "never produced by the crate; match on `IoError` for invalid UTF-8 from a reader"
    )]
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] core::str::Utf8Error),

    /// Catch-all for unexpected errors.
    ///
    /// There is deliberately no `From<BoxedError>`: a boxed error never
    /// becomes a `MetadataError` by accident through `?`. Build it with
    /// [`MetadataError::other`] where the coercion is intended.
    #[error("Unexpected error: {0}")]
    Other(BoxedError),
}

/// Renders ` at bytes a..b` for the `Parse` display, or nothing.
fn span_suffix(span: &Option<Range<usize>>) -> String {
    match span {
        Some(r) => format!(" at bytes {}..{}", r.start, r.end),
        None => String::new(),
    }
}

impl MetadataError {
    /// Wraps any error as [`MetadataError::Other`].
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::error::MetadataError;
    ///
    /// let err = MetadataError::other(core::fmt::Error);
    /// assert!(matches!(err, MetadataError::Other(_)));
    /// ```
    pub fn other<E>(error: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::Other(Box::new(error))
    }

    /// Builds a [`MetadataError::Parse`] from a parser's error.
    #[cfg(any(feature = "yaml", feature = "toml", feature = "json"))]
    pub(crate) fn parse<E>(
        format: FrontMatterFormat,
        span: Option<Range<usize>>,
        source: E,
    ) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::Parse {
            format,
            span,
            source: Box::new(source),
        }
    }

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
            #[cfg(feature = "std")]
            Self::IoError(error) => Self::IoError(std::io::Error::new(
                error.kind(),
                format!("{}: {}", ctx, error),
            )),
            #[cfg(feature = "yaml")]
            Self::YamlError(error) => Self::YamlError(
                SerdeYmlError::custom(format!("{}: {}", ctx, error)),
            ),
            #[cfg(feature = "json")]
            Self::JsonError(error) => {
                Self::JsonError(serde_json::Error::custom(format!(
                    "{}: {}",
                    ctx, error
                )))
            }
            #[cfg(feature = "toml")]
            Self::TomlError(error) => Self::TomlError(
                toml::de::Error::custom(format!("{}: {}", ctx, error)),
            ),
            Self::Parse {
                format,
                span,
                source,
            } => Self::Parse {
                format,
                span,
                source: Box::new(ContextError {
                    context: ctx.to_string(),
                    source,
                }),
            },
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

#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
mod tests;

#[cfg(all(
    test,
    feature = "std",
    feature = "yaml",
    feature = "toml",
    feature = "json"
))]
mod context_tests;
