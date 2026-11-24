// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Front matter as a stream of events.
//!
//! [`for_each_event`] walks the block through the format's own `serde`
//! deserialiser with a visitor that hands each key, scalar and container
//! boundary to a callback as it is read; no map or tree of values is
//! built. The YAML and JSON deserialisers are streaming, so memory stays
//! bounded by nesting depth, not by input size. The TOML parser reads
//! the whole block into its document model first; the events are still
//! delivered one at a time. Returning [`ControlFlow::Break`] from the
//! callback stops the walk and no further input is parsed.
//!
//! String values that the input holds verbatim arrive as
//! [`Cow::Borrowed`] slices of the document; values that needed
//! unescaping arrive owned.

use super::{FrontMatterFormat, ParseLimits};
use visitor::Node;

mod visitor;
use crate::error::MetadataError;
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::ops::ControlFlow;

/// One step of a front-matter walk.
///
/// # Example
///
/// ```
/// use metadata_gen::{for_each_event, Event};
/// use std::ops::ControlFlow;
///
/// let mut keys = Vec::new();
/// for_each_event("---\ntitle: Hi\ntags: [a]\n---\n", |event| {
///     if let Event::Key(k) = event {
///         keys.push(k.into_owned());
///     }
///     ControlFlow::Continue(())
/// })
/// .unwrap();
/// assert_eq!(keys, ["title", "tags"]);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Event<'a> {
    /// A mapping opens. The top-level block is a mapping too.
    MapStart,
    /// A mapping key; its value follows.
    Key(Cow<'a, str>),
    /// A scalar value.
    Scalar(Scalar<'a>),
    /// A sequence opens.
    SeqStart,
    /// The innermost open sequence closes.
    SeqEnd,
    /// The innermost open mapping closes.
    MapEnd,
}

/// A scalar value as the parser typed it.
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::events::Scalar;
///
/// assert_eq!(Scalar::Int(3).to_text(), "3");
/// assert_eq!(Scalar::Bool(true).to_text(), "true");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Scalar<'a> {
    /// A string, borrowed from the document when it needed no unescaping.
    Str(Cow<'a, str>),
    /// A boolean.
    Bool(bool),
    /// A signed integer.
    Int(i64),
    /// An unsigned integer too large for `i64`.
    UInt(u64),
    /// A floating-point number.
    Float(f64),
    /// `null` (YAML `~`, JSON `null`).
    Null,
}

impl Scalar<'_> {
    /// The scalar as text, the way `extract_metadata` renders leaves:
    /// strings as-is, `null` as the word `null`.
    ///
    /// # Example
    ///
    /// ```
    /// use metadata_gen::metadata::events::Scalar;
    ///
    /// assert_eq!(Scalar::Str("hi".into()).to_text(), "hi");
    /// assert_eq!(Scalar::Null.to_text(), "null");
    /// ```
    pub fn to_text(&self) -> String {
        match self {
            Scalar::Str(s) => s.to_string(),
            other => other.to_string(),
        }
    }
}

impl fmt::Display for Scalar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scalar::Str(s) => f.write_str(s),
            Scalar::Bool(b) => write!(f, "{b}"),
            Scalar::Int(i) => write!(f, "{i}"),
            Scalar::UInt(u) => write!(f, "{u}"),
            Scalar::Float(x) => write!(f, "{x}"),
            Scalar::Null => f.write_str("null"),
        }
    }
}

/// Walks the front matter of `content`, calling `on_event` for each
/// event in document order.
///
/// Returns `ControlFlow::Continue(())` when the whole block was walked
/// and `ControlFlow::Break(())` when `on_event` stopped it.
///
/// # Errors
///
/// The errors of [`crate::extract_metadata`]: no front matter, an
/// unclosed fence, a block over the default [`ParseLimits`], a parse
/// failure, or a format that is not compiled in. Events already
/// delivered before a parse failure stay delivered.
///
/// # Example
///
/// ```
/// use core::ops::ControlFlow;
/// use metadata_gen::metadata::events::{for_each_event, Event};
///
/// let doc = "---\ntitle: Hello\ntags: [a, b]\n---\nbody";
/// let mut keys = Vec::new();
/// for_each_event(doc, |event| {
///     if let Event::Key(k) = event {
///         keys.push(k.into_owned());
///     }
///     ControlFlow::Continue(())
/// })
/// .unwrap();
/// assert_eq!(keys, ["title", "tags"]);
///
/// // Stop at the first key: nothing after it is parsed.
/// let flow = for_each_event(doc, |event| match event {
///     Event::Key(_) => ControlFlow::Break(()),
///     _ => ControlFlow::Continue(()),
/// })
/// .unwrap();
/// assert!(flow.is_break());
/// ```
pub fn for_each_event<'a, F>(
    content: &'a str,
    on_event: F,
) -> Result<ControlFlow<()>, MetadataError>
where
    F: FnMut(Event<'a>) -> ControlFlow<()>,
{
    for_each_event_with_limits(
        content,
        &ParseLimits::default(),
        on_event,
    )
}

/// [`for_each_event`] under explicit [`ParseLimits`], for manifests
/// larger than the default 1 MiB block.
///
/// # Errors
///
/// As [`for_each_event`].
///
/// # Example
///
/// ```
/// use core::ops::ControlFlow;
/// use metadata_gen::metadata::events::for_each_event_with_limits;
/// use metadata_gen::ParseLimits;
///
/// let limits = ParseLimits::default().max_block_bytes(16);
/// let err = for_each_event_with_limits(
///     "---\ntitle: far too long for the limit\n---\n",
///     &limits,
///     |_| ControlFlow::Continue(()),
/// )
/// .unwrap_err();
/// assert!(err.to_string().contains("exceeds"));
/// ```
pub fn for_each_event_with_limits<'a, F>(
    content: &'a str,
    limits: &ParseLimits,
    on_event: F,
) -> Result<ControlFlow<()>, MetadataError>
where
    F: FnMut(Event<'a>) -> ControlFlow<()>,
{
    let (format, raw) = locate(content, limits)?;
    let mut sink = Sink::new(on_event);
    let outcome = walk(format, raw, limits, &mut sink);
    match outcome {
        _ if sink.stopped => Ok(ControlFlow::Break(())),
        Ok(()) => Ok(ControlFlow::Continue(())),
        Err(e) => Err(e),
    }
}

/// Appends every event of the front matter to `buffer`.
///
/// The caller owns the buffer, so a pipeline that reads many documents
/// can `clear()` and reuse one allocation.
///
/// # Errors
///
/// As [`for_each_event`].
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::events::{events_into, Event, Scalar};
///
/// let mut buffer = Vec::new();
/// for doc in ["---\na: 1\n---\n", "---\nb: true\n---\n"] {
///     buffer.clear();
///     events_into(doc, &mut buffer).unwrap();
///     assert_eq!(buffer.len(), 4); // MapStart, Key, Scalar, MapEnd
/// }
/// assert_eq!(buffer[2], Event::Scalar(Scalar::Bool(true)));
/// ```
pub fn events_into<'a>(
    content: &'a str,
    buffer: &mut Vec<Event<'a>>,
) -> Result<(), MetadataError> {
    for_each_event(content, |event| {
        buffer.push(event);
        ControlFlow::Continue(())
    })
    .map(|_| ())
}

/// Collects every event of the front matter.
///
/// # Errors
///
/// As [`for_each_event`].
///
/// # Example
///
/// ```
/// use metadata_gen::metadata::events::{events, Event};
///
/// let all = events("{\"a\": {\"b\": 1}}\nbody").unwrap();
/// assert_eq!(all.first(), Some(&Event::MapStart));
/// assert_eq!(all.len(), 7);
/// ```
pub fn events(content: &str) -> Result<Vec<Event<'_>>, MetadataError> {
    let mut buffer = Vec::new();
    events_into(content, &mut buffer)?;
    Ok(buffer)
}

/// Finds the block and checks it against `limits`.
fn locate<'a>(
    content: &'a str,
    limits: &ParseLimits,
) -> Result<(FrontMatterFormat, &'a str), MetadataError> {
    let (format, raw, _) = super::find_block(content)?;
    super::front_matter::check_size(raw, limits)?;
    Ok((format, raw))
}

/// Runs the walking visitor over `raw` with the format's deserialiser.
fn walk<'a, F>(
    format: FrontMatterFormat,
    raw: &'a str,
    limits: &ParseLimits,
    sink: &mut Sink<F>,
) -> Result<(), MetadataError>
where
    F: FnMut(Event<'a>) -> ControlFlow<()>,
{
    match format {
        #[cfg(feature = "yaml")]
        FrontMatterFormat::Yaml => walk_yaml(raw, limits, sink),
        #[cfg(feature = "toml")]
        FrontMatterFormat::Toml => {
            super::front_matter::toml_fm::walk(raw, Node(sink))
        }
        #[cfg(feature = "json")]
        FrontMatterFormat::Json => {
            super::front_matter::json::walk(raw, Node(sink))
        }
        #[cfg(not(all(
            feature = "yaml",
            feature = "toml",
            feature = "json"
        )))]
        other => {
            // Unreachable in practice: the scanner reports a compiled-out
            // format before a walk starts. The bindings stay used in
            // every feature combination.
            let _ = (raw, limits, &sink);
            Err(MetadataError::UnsupportedFormatError(
                other.to_string(),
            ))
        }
    }
}

/// Streams YAML; when noyalib signals that a shape needs its value
/// path, parses the tree and replays it, skipping the events the
/// streaming pass already delivered.
#[cfg(feature = "yaml")]
fn walk_yaml<'a, F>(
    raw: &'a str,
    limits: &ParseLimits,
    sink: &mut Sink<F>,
) -> Result<(), MetadataError>
where
    F: FnMut(Event<'a>) -> ControlFlow<()>,
{
    use super::front_matter::yaml;
    match yaml::stream(raw, limits, Node(&mut *sink)) {
        Ok(()) => Ok(()),
        Err(e) if !sink.stopped && yaml::is_fallback(&e) => {
            let value = yaml::value(raw, limits)?;
            sink.skip = sink.delivered;
            emit_value::<F, noyalib::Error>(&value, sink)
                .map_err(|e| yaml::error(raw, e))
        }
        Err(e) => Err(yaml::error(raw, e)),
    }
}

/// Emits the events of a parsed YAML value; every string is owned.
#[cfg(feature = "yaml")]
fn emit_value<'a, F, E>(
    value: &noyalib::Value,
    sink: &mut Sink<F>,
) -> Result<(), E>
where
    F: FnMut(Event<'a>) -> ControlFlow<()>,
    E: serde::de::Error,
{
    use noyalib::Value;
    match value {
        Value::Sequence(items) => emit_seq(items, sink),
        Value::Mapping(entries) => emit_map(entries, sink),
        Value::Tagged(tagged) => emit_value(tagged.value(), sink),
        other => sink.emit(Event::Scalar(yaml_scalar(other))),
    }
}

#[cfg(feature = "yaml")]
fn emit_seq<'a, F, E>(
    items: &[noyalib::Value],
    sink: &mut Sink<F>,
) -> Result<(), E>
where
    F: FnMut(Event<'a>) -> ControlFlow<()>,
    E: serde::de::Error,
{
    sink.emit::<E>(Event::SeqStart)?;
    for item in items {
        emit_value::<F, E>(item, sink)?;
    }
    sink.emit(Event::SeqEnd)
}

#[cfg(feature = "yaml")]
fn emit_map<'a, F, E>(
    entries: &noyalib::Mapping,
    sink: &mut Sink<F>,
) -> Result<(), E>
where
    F: FnMut(Event<'a>) -> ControlFlow<()>,
    E: serde::de::Error,
{
    sink.emit::<E>(Event::MapStart)?;
    for (k, v) in entries {
        sink.emit::<E>(Event::Key(Cow::Owned(k.clone())))?;
        emit_value::<F, E>(v, sink)?;
    }
    sink.emit(Event::MapEnd)
}

/// A YAML leaf as a [`Scalar`]; containers never reach here.
#[cfg(feature = "yaml")]
fn yaml_scalar(value: &noyalib::Value) -> Scalar<'static> {
    use noyalib::Value;
    match value {
        Value::Bool(b) => Scalar::Bool(*b),
        Value::Number(n) => {
            n.as_i64().map_or(Scalar::Float(n.as_f64()), Scalar::Int)
        }
        Value::String(s) => Scalar::Str(Cow::Owned(s.clone())),
        _ => Scalar::Null,
    }
}

/// The callback, whether it asked to stop, and the replay bookkeeping
/// for the YAML fallback path.
pub(in crate::metadata) struct Sink<F> {
    on_event: F,
    stopped: bool,
    delivered: usize,
    skip: usize,
}

impl<'a, F: FnMut(Event<'a>) -> ControlFlow<()>> Sink<F> {
    pub(super) fn new(on_event: F) -> Self {
        Self {
            on_event,
            stopped: false,
            delivered: 0,
            skip: 0,
        }
    }

    pub(super) fn emit<E: serde::de::Error>(
        &mut self,
        event: Event<'a>,
    ) -> Result<(), E> {
        if self.skip > 0 {
            self.skip -= 1;
            return Ok(());
        }
        self.delivered += 1;
        if (self.on_event)(event).is_break() {
            self.stopped = true;
            return Err(E::custom("event walk stopped by the caller"));
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "yaml"))]
mod tests;
