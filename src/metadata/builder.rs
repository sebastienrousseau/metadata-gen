// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A typestate builder for [`Metadata`].
//!
//! `title` and `date` are required: [`MetadataBuilder::build`] only
//! exists once both are set, so a missing field is a compile error rather
//! than a [`crate::MetadataError::MissingFieldError`] at run time. The
//! optional setters can be called in any order, before or after the
//! required ones.
//!
//! ```
//! use metadata_gen::Metadata;
//!
//! let meta = Metadata::builder()
//!     .description("A post")
//!     .title("Hello")
//!     .keywords(["rust", "yaml"])
//!     .date("2026-01-01")
//!     .build();
//! assert_eq!(meta.get("title").unwrap(), "Hello");
//! assert_eq!(meta.get("keywords").unwrap(), "[rust, yaml]");
//! assert_eq!(meta.keywords(), ["rust", "yaml"]);
//! ```
//!
//! Leaving out `date` does not compile; the error reads "missing
//! required field: date":
//!
//! ```compile_fail
//! use metadata_gen::Metadata;
//!
//! let meta = Metadata::builder().title("Hello").build();
//! ```

use super::Metadata;
use crate::MetadataMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Typestate marker: the field has not been set.
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::builder::{MetadataBuilder, Missing, Present};
/// use metadata_gen::Metadata;
///
/// // Before `title` is set the builder's first parameter is `Missing`.
/// let start: MetadataBuilder<Missing, Missing> = Metadata::builder();
/// let titled: MetadataBuilder<Present, Missing> = start.title("T");
/// let _ = titled;
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Missing;

/// Typestate marker: the field has been set.
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::builder::{MetadataBuilder, Present};
/// use metadata_gen::Metadata;
///
/// let ready: MetadataBuilder<Present, Present> =
///     Metadata::builder().title("T").date("2026-01-01");
/// assert_eq!(ready.build().get("title").unwrap(), "T");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Present;

/// Implemented only for [`Present`]: `title` has been set.
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::builder::{DateSet, MetadataBuilder, TitleSet};
/// use metadata_gen::Metadata;
///
/// // A helper that accepts only builders whose required fields are set.
/// fn finish<T: TitleSet, D: DateSet>(b: MetadataBuilder<T, D>) -> Metadata {
///     b.build()
/// }
/// let meta = finish(Metadata::builder().title("T").date("2026-01-01"));
/// assert!(meta.contains_key("date"));
/// ```
#[diagnostic::on_unimplemented(
    message = "missing required field: title",
    label = "call `.title(..)` before `.build()`"
)]
pub trait TitleSet: sealed::Sealed {}

/// Implemented only for [`Present`]: `date` has been set.
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::builder::{DateSet, MetadataBuilder, Present};
/// use metadata_gen::Metadata;
///
/// fn has_date<D: DateSet>(_: &MetadataBuilder<Present, D>) {}
/// has_date(&Metadata::builder().title("T").date("2026-01-01"));
/// ```
#[diagnostic::on_unimplemented(
    message = "missing required field: date",
    label = "call `.date(..)` before `.build()`"
)]
pub trait DateSet: sealed::Sealed {}

impl TitleSet for Present {}
impl DateSet for Present {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Present {}
    impl Sealed for super::Missing {}
}

/// Builds a [`Metadata`]; start it with [`Metadata::builder`].
///
/// `T` and `D` record whether `title` and `date` are set.
///
/// # Example
///
/// ```
/// use metadata_gen::Metadata;
///
/// let meta = Metadata::builder()
///     .title("Hello")
///     .date("2026-01-01")
///     .field("author", "Ada")
///     .build();
/// assert_eq!(meta.get("author").unwrap(), "Ada");
/// ```
#[derive(Debug, Clone)]
#[must_use]
pub struct MetadataBuilder<T, D> {
    map: MetadataMap,
    state: PhantomData<(T, D)>,
}

impl MetadataBuilder<Missing, Missing> {
    pub(super) fn new() -> Self {
        Self {
            map: MetadataMap::new(),
            state: PhantomData,
        }
    }
}

impl<T, D> MetadataBuilder<T, D> {
    fn with(mut self, key: &str, value: String) -> Self {
        let _ = self.map.insert(key.to_string(), value);
        self
    }

    fn into_state<T2, D2>(self) -> MetadataBuilder<T2, D2> {
        MetadataBuilder {
            map: self.map,
            state: PhantomData,
        }
    }

    /// Sets `title`.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::Metadata;
    ///
    /// let meta = Metadata::builder().title("Hello").date("2026-01-01").build();
    /// assert_eq!(meta.get("title").unwrap(), "Hello");
    /// ```
    pub fn title(
        self,
        title: impl Into<String>,
    ) -> MetadataBuilder<Present, D> {
        self.with("title", title.into()).into_state()
    }

    /// Sets `date`, stored as given (run [`crate::process_metadata`] to
    /// normalise it).
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::Metadata;
    ///
    /// let meta = Metadata::builder().date("01/02/2026").title("T").build();
    /// assert_eq!(meta.get("date").unwrap(), "01/02/2026");
    /// ```
    pub fn date(
        self,
        date: impl Into<String>,
    ) -> MetadataBuilder<T, Present> {
        self.with("date", date.into()).into_state()
    }

    /// Sets `description`.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::Metadata;
    ///
    /// let meta = Metadata::builder()
    ///     .title("T")
    ///     .date("2026-01-01")
    ///     .description("A post")
    ///     .build();
    /// assert_eq!(meta.get("description").unwrap(), "A post");
    /// ```
    pub fn description(self, description: impl Into<String>) -> Self {
        self.with("description", description.into())
    }

    /// Sets `keywords`, stored as `[a, b]`, the shape a front-matter
    /// sequence flattens to. [`Metadata::keywords`] gives the list back.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::Metadata;
    ///
    /// let meta = Metadata::builder()
    ///     .title("T")
    ///     .date("2026-01-01")
    ///     .keywords(["rust", "yaml"])
    ///     .build();
    /// assert_eq!(meta.get("keywords").unwrap(), "[rust, yaml]");
    /// ```
    pub fn keywords<I, S>(self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let list: Vec<String> =
            keywords.into_iter().map(Into::into).collect();
        self.with("keywords", format!("[{}]", list.join(", ")))
    }

    /// Sets any other field.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::Metadata;
    ///
    /// let meta = Metadata::builder()
    ///     .title("T")
    ///     .date("2026-01-01")
    ///     .field("og:image", "/cover.png")
    ///     .build();
    /// assert_eq!(meta.get("og:image").unwrap(), "/cover.png");
    /// ```
    pub fn field(
        self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        let key = key.into();
        self.with(&key, value.into())
    }
}

impl<T, D> MetadataBuilder<T, D> {
    /// Finishes the builder. Only compiles once `title` and `date` are
    /// set; otherwise the error names the missing field.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::Metadata;
    ///
    /// let meta = Metadata::builder().title("T").date("2026-01-01").build();
    /// assert_eq!(meta.keys().count(), 2);
    /// ```
    #[must_use]
    pub fn build(self) -> Metadata
    where
        T: TitleSet,
        D: DateSet,
    {
        Metadata::new(self.map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setters_in_any_order() {
        let meta = Metadata::builder()
            .date("2026-01-01")
            .field("author", "Ada")
            .title("T")
            .description("D")
            .build();
        assert_eq!(meta.get("date").unwrap(), "2026-01-01");
        assert_eq!(meta.get("author").unwrap(), "Ada");
        assert_eq!(meta.get("description").unwrap(), "D");
        assert!(meta.keywords().is_empty());
    }

    #[test]
    fn keywords_round_trip() {
        let meta = Metadata::builder()
            .title("T")
            .date("d")
            .keywords(alloc::vec!["a", "b"])
            .build();
        assert_eq!(meta.keywords(), ["a", "b"]);
    }

    #[test]
    fn comma_separated_keywords_are_split() {
        let meta = Metadata::builder()
            .title("T")
            .date("d")
            .field("keywords", "a, b,, c")
            .build();
        assert_eq!(meta.keywords(), ["a", "b", "c"]);
    }
}
