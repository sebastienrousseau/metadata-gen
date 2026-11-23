// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Zero-copy flat metadata: the dotted-key map of `extract_metadata`,
//! with keys and values borrowed from the document where it allows.

use super::events::{for_each_event, Event, Scalar};
use super::Metadata;
use crate::error::MetadataError;
use crate::MetadataMap;
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::ControlFlow;

/// Flat front matter whose keys and values borrow from the document.
///
/// Built by [`extract_metadata_cow`]. A top-level key and a string value
/// written without escapes are [`Cow::Borrowed`] slices of the input;
/// nested keys (`author.name`), sequences (`[a, b]`), numbers, booleans
/// and unescaped strings are owned. The map cannot outlive the document;
/// [`CowMetadata::into_owned`] gives a `'static` [`Metadata`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CowMetadata<'a> {
    inner: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> CowMetadata<'a> {
    /// The value under `key`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|v| &**v)
    }

    /// The value under `key`, keeping whether it is borrowed.
    pub fn get_cow(&self, key: &str) -> Option<&Cow<'a, str>> {
        self.inner.get(key)
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// True when there are no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Entries in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.inner.iter().map(|(k, v)| (&**k, &**v))
    }

    /// How many values are borrowed from the document rather than
    /// allocated.
    pub fn borrowed_values(&self) -> usize {
        self.inner
            .values()
            .filter(|v| matches!(v, Cow::Borrowed(_)))
            .count()
    }

    /// The underlying map.
    pub fn into_inner(self) -> BTreeMap<Cow<'a, str>, Cow<'a, str>> {
        self.inner
    }

    /// Copies every entry into an owned [`Metadata`], for callers that
    /// must keep the metadata after the document is gone.
    pub fn into_owned(self) -> Metadata {
        let map: MetadataMap = self
            .inner
            .into_iter()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        Metadata::new(map)
    }
}

/// Extracts flat metadata without copying the strings the document
/// already holds.
///
/// Keys and nesting follow [`crate::extract_metadata`]: nested mappings
/// give dotted keys and a sequence of scalars gives `[a, b]`. Numbers are
/// rendered with Rust's `Display` (`1.0` reads as `1`), YAML is parsed as
/// written (the multi-line quoted-scalar repair of `extract_metadata`
/// would need a copy to borrow from), and a mapping inside a sequence is
/// rendered as `{k: v}`.
///
/// # Errors
///
/// As [`crate::extract_metadata`].
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::extract_metadata_cow;
/// use std::borrow::Cow;
///
/// let doc = "---\ntitle: Hello\nauthor:\n  name: Ada\ntags: [a, b]\n---\nbody";
/// let meta = extract_metadata_cow(doc).unwrap();
/// assert_eq!(meta.get("title"), Some("Hello"));
/// assert_eq!(meta.get("author.name"), Some("Ada"));
/// assert_eq!(meta.get("tags"), Some("[a, b]"));
/// assert!(matches!(meta.get_cow("title"), Some(Cow::Borrowed(_))));
/// let owned = meta.into_owned();
/// assert_eq!(owned.get("title").unwrap(), "Hello");
/// ```
pub fn extract_metadata_cow(
    content: &str,
) -> Result<CowMetadata<'_>, MetadataError> {
    let mut flat = Flattener::default();
    let _ = for_each_event(content, |event| {
        flat.push(event);
        ControlFlow::Continue(())
    })?;
    Ok(CowMetadata { inner: flat.map })
}

/// Turns the event stream into dotted-key entries.
#[derive(Default)]
struct Flattener<'a> {
    map: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
    /// One entry per open mapping: the key it sits under, if any.
    maps: Vec<Option<Cow<'a, str>>>,
    pending: Option<Cow<'a, str>>,
    /// Sequence nesting; while non-zero, events render into `text`.
    seq_depth: usize,
    text: String,
    separate: bool,
}

impl<'a> Flattener<'a> {
    fn push(&mut self, event: Event<'a>) {
        if self.seq_depth > 0 {
            self.render(event);
            return;
        }
        match event {
            Event::MapStart => self.maps.push(self.pending.take()),
            Event::MapEnd => {
                let _ = self.maps.pop();
            }
            Event::Key(k) => self.pending = Some(k),
            Event::Scalar(scalar) => self.leaf(scalar),
            Event::SeqStart => self.open_seq(),
            Event::SeqEnd => {}
        }
    }

    fn leaf(&mut self, scalar: Scalar<'a>) {
        let at_root = self.pending.is_none() && self.maps.is_empty();
        if at_root && scalar == Scalar::Null {
            return; // An empty block: no entries, as extract_metadata.
        }
        let value = match scalar {
            Scalar::Str(s) => s,
            other => Cow::Owned(other.to_text()),
        };
        let key = self.full_key();
        let _ = self.map.insert(key, value);
    }

    fn open_seq(&mut self) {
        self.seq_depth = 1;
        self.text.clear();
        self.text.push('[');
        self.separate = false;
    }

    /// Renders an event inside a sequence in flow style.
    fn render(&mut self, event: Event<'a>) {
        let closing = matches!(event, Event::SeqEnd | Event::MapEnd);
        if self.separate && !closing {
            self.text.push_str(", ");
        }
        self.separate = !matches!(
            event,
            Event::SeqStart | Event::MapStart | Event::Key(_)
        );
        match event {
            Event::SeqStart => self.nest('['),
            Event::MapStart => self.nest('{'),
            Event::MapEnd => self.unnest('}'),
            Event::SeqEnd => self.unnest(']'),
            Event::Key(k) => {
                self.text.push_str(&k);
                self.text.push_str(": ");
            }
            Event::Scalar(s) => self.text.push_str(&s.to_text()),
        }
    }

    fn nest(&mut self, open: char) {
        self.seq_depth += 1;
        self.text.push(open);
    }

    fn unnest(&mut self, close: char) {
        self.seq_depth -= 1;
        self.text.push(close);
        if self.seq_depth == 0 {
            let value = Cow::Owned(core::mem::take(&mut self.text));
            let key = self.full_key();
            let _ = self.map.insert(key, value);
        }
    }

    /// The dotted key for the pending key under the open mappings.
    fn full_key(&mut self) -> Cow<'a, str> {
        let last = self.pending.take().unwrap_or_default();
        let parents: Vec<&str> =
            self.maps.iter().flatten().map(|k| &**k).collect();
        if parents.is_empty() {
            return last;
        }
        let mut key = parents.join(".");
        key.push('.');
        key.push_str(&last);
        Cow::Owned(key)
    }
}

#[cfg(all(test, feature = "std", feature = "yaml", feature = "json"))]
mod tests {
    use super::*;

    #[test]
    fn matches_extract_metadata_on_common_shapes() {
        let doc = "---\ntitle: T\nn: 3\nok: true\nnone: ~\n\
                   a:\n  b:\n    c: deep\nlist: [x, y]\n---\n";
        let cow = extract_metadata_cow(doc).unwrap();
        let flat = crate::extract_metadata(doc).unwrap().into_inner();
        assert_eq!(cow.clone().into_owned().into_inner(), flat);
        assert_eq!(cow.get("a.b.c"), Some("deep"));
    }

    #[test]
    fn escaped_values_are_owned() {
        let doc = "{\"plain\": \"abc\", \"esc\": \"a\\nb\"}";
        let cow = extract_metadata_cow(doc).unwrap();
        assert!(matches!(cow.get_cow("plain"), Some(Cow::Borrowed(_))));
        assert!(matches!(cow.get_cow("esc"), Some(Cow::Owned(_))));
        assert_eq!(cow.get("esc"), Some("a\nb"));
        assert_eq!(cow.borrowed_values(), 1);
    }

    #[test]
    fn containers_inside_sequences_render_in_flow_style() {
        let doc =
            "{\"s\": [{\"k\": 1, \"j\": [2, 3]}, [], 4], \"e\": {}}";
        let cow = extract_metadata_cow(doc).unwrap();
        assert_eq!(cow.get("s"), Some("[{k: 1, j: [2, 3]}, [], 4]"));
        assert_eq!(cow.len(), 1);
    }

    #[test]
    fn empty_block_is_empty() {
        let cow = extract_metadata_cow("---\n---\n").unwrap();
        assert!(cow.is_empty());
        assert_eq!(cow.iter().count(), 0);
    }

    #[test]
    fn root_scalar_sits_under_the_empty_key() {
        let cow = extract_metadata_cow("---\nhello\n---\n").unwrap();
        assert_eq!(cow.get(""), Some("hello"));
        assert_eq!(cow.into_inner().len(), 1);
    }
}
