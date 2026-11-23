// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The [`Metadata`] type.

use super::builder;
use crate::MetadataMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Represents metadata for a page or content item.
///
/// # Example
///
/// ```
/// use metadata_gen::Metadata;
/// use std::collections::HashMap;
///
/// let mut data = HashMap::new();
/// data.insert("title".to_string(), "My Page".to_string());
/// let metadata = Metadata::new(data);
/// assert_eq!(metadata.get("title"), Some(&"My Page".to_string()));
/// ```
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    /// The underlying key-value store for metadata fields.
    inner: MetadataMap,
}

impl Metadata {
    /// Creates a new `Metadata` instance with the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - A [`MetadataMap`] containing the metadata key-value pairs.
    ///
    /// # Returns
    ///
    /// A new `Metadata` instance.
    pub fn new(data: MetadataMap) -> Self {
        Metadata { inner: data }
    }

    /// Retrieves the value associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key to look up.
    ///
    /// # Returns
    ///
    /// An `Option<&String>` containing the value if the key exists, or `None` otherwise.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    /// Inserts a key-value pair into the metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert.
    /// * `value` - The value to associate with the key.
    ///
    /// # Returns
    ///
    /// The old value associated with the key, if it existed.
    pub fn insert(
        &mut self,
        key: String,
        value: String,
    ) -> Option<String> {
        self.inner.insert(key, value)
    }

    /// Checks if the metadata contains the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key to check for.
    ///
    /// # Returns
    ///
    /// `true` if the key exists, `false` otherwise.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// The keys present, in the map's iteration order.
    pub fn keys(&self) -> impl Iterator<Item = &str> + '_ {
        self.inner.keys().map(String::as_str)
    }

    /// The `keywords` value as a list: a flattened sequence (`[a, b]`)
    /// or a comma-separated string (`a, b`) both give `["a", "b"]`.
    /// Empty when there is no `keywords` key.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::extract_metadata;
    ///
    /// let meta = extract_metadata("---\nkeywords: [rust, yaml]\n---\n").unwrap();
    /// assert_eq!(meta.keywords(), ["rust", "yaml"]);
    /// ```
    pub fn keywords(&self) -> Vec<String> {
        let Some(raw) = self.inner.get("keywords") else {
            return Vec::new();
        };
        let list = raw
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
            .unwrap_or(raw);
        list.split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(ToString::to_string)
            .collect()
    }

    /// Starts a [`builder::MetadataBuilder`]. `title` and `date` must be
    /// set before `build` compiles; see the [`builder`] module.
    pub fn builder(
    ) -> builder::MetadataBuilder<builder::Missing, builder::Missing>
    {
        builder::MetadataBuilder::new()
    }

    /// Consumes the `Metadata` instance and returns the inner map.
    ///
    /// # Returns
    ///
    /// The inner [`MetadataMap`] containing all metadata key-value pairs.
    pub fn into_inner(self) -> MetadataMap {
        self.inner
    }
}
