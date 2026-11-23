// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The serde visitor that turns any self-describing deserialiser into
//! [`Event`]s.

use super::{Event, Scalar, Sink};
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use core::fmt;
use core::ops::ControlFlow;

/// A `DeserializeSeed` for one value: emits its events into the sink.
pub(super) struct Node<'s, F>(pub(super) &'s mut Sink<F>);

/// TOML's serde representation of a datetime: a one-entry map under this
/// key whose value is the datetime's text.
const TOML_DATETIME: &str = "$__toml_private_datetime";

impl<'de, F> serde::de::DeserializeSeed<'de> for Node<'_, F>
where
    F: FnMut(Event<'de>) -> ControlFlow<()>,
{
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, F> serde::de::Visitor<'de> for Node<'_, F>
where
    F: FnMut(Event<'de>) -> ControlFlow<()>,
{
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any front-matter value")
    }

    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<(), E> {
        self.0.emit(Event::Scalar(Scalar::Bool(v)))
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<(), E> {
        self.0.emit(Event::Scalar(Scalar::Int(v)))
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<(), E> {
        let scalar =
            i64::try_from(v).map_or(Scalar::UInt(v), Scalar::Int);
        self.0.emit(Event::Scalar(scalar))
    }

    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<(), E> {
        self.0.emit(Event::Scalar(Scalar::Float(v)))
    }

    fn visit_borrowed_str<E: serde::de::Error>(
        self,
        v: &'de str,
    ) -> Result<(), E> {
        self.0.emit(Event::Scalar(Scalar::Str(Cow::Borrowed(v))))
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<(), E> {
        self.visit_string(v.to_string())
    }

    fn visit_string<E: serde::de::Error>(
        self,
        v: String,
    ) -> Result<(), E> {
        self.0.emit(Event::Scalar(Scalar::Str(Cow::Owned(v))))
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        self.0.emit(Event::Scalar(Scalar::Null))
    }

    fn visit_none<E: serde::de::Error>(self) -> Result<(), E> {
        self.visit_unit()
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> Result<(), A::Error> {
        self.0.emit(Event::SeqStart)?;
        while seq.next_element_seed(Node(&mut *self.0))?.is_some() {}
        self.0.emit(Event::SeqEnd)
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> Result<(), A::Error> {
        let first = next_key(&mut map)?;
        if first.as_deref() == Some(TOML_DATETIME) {
            let text: String = map.next_value()?;
            return self
                .0
                .emit(Event::Scalar(Scalar::Str(Cow::Owned(text))));
        }
        self.0.emit(Event::MapStart)?;
        let mut key = first;
        while let Some(k) = key {
            self.0.emit(Event::Key(k))?;
            map.next_value_seed(Node(&mut *self.0))?;
            key = next_key(&mut map)?;
        }
        self.0.emit(Event::MapEnd)
    }
}

/// The next mapping key, borrowed when the deserialiser can lend it.
fn next_key<'de, A: serde::de::MapAccess<'de>>(
    map: &mut A,
) -> Result<Option<Cow<'de, str>>, A::Error> {
    Ok(map.next_key::<CowStr<'de>>()?.map(|k| k.0))
}

/// A string that stays borrowed when the deserialiser can lend it.
/// serde's own `Cow<str>` impl always allocates.
struct CowStr<'de>(Cow<'de, str>);

impl<'de> serde::Deserialize<'de> for CowStr<'de> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_str(CowStrVisitor)
    }
}

struct CowStrVisitor;

impl<'de> serde::de::Visitor<'de> for CowStrVisitor {
    type Value = CowStr<'de>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string key")
    }

    fn visit_borrowed_str<E: serde::de::Error>(
        self,
        v: &'de str,
    ) -> Result<Self::Value, E> {
        Ok(CowStr(Cow::Borrowed(v)))
    }

    fn visit_str<E: serde::de::Error>(
        self,
        v: &str,
    ) -> Result<Self::Value, E> {
        Ok(CowStr(Cow::Owned(v.to_string())))
    }

    fn visit_string<E: serde::de::Error>(
        self,
        v: String,
    ) -> Result<Self::Value, E> {
        Ok(CowStr(Cow::Owned(v)))
    }
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use super::*;
    use serde::de::Expected;

    #[test]
    fn visitors_describe_what_they_expect() {
        let mut sink = Sink::new(|_| ControlFlow::Continue(()));
        let node = Node(&mut sink);
        assert_eq!(
            alloc::format!("{}", &node as &dyn Expected),
            "any front-matter value"
        );
        assert_eq!(
            alloc::format!("{}", &CowStrVisitor as &dyn Expected),
            "a string key"
        );
    }

    #[test]
    fn escaped_keys_are_owned() {
        let mut keys = alloc::vec::Vec::new();
        let mut sink = Sink::new(|e| {
            if let Event::Key(k) = e {
                keys.push(k);
            }
            ControlFlow::Continue(())
        });
        let mut de = serde_json::Deserializer::from_str(
            "{\"a\\nb\": 1, \"c\": 2}",
        );
        serde::de::DeserializeSeed::deserialize(
            Node(&mut sink),
            &mut de,
        )
        .unwrap();
        assert!(matches!(&keys[0], Cow::Owned(k) if k == "a\nb"));
        assert!(matches!(&keys[1], Cow::Borrowed("c")));
    }
}
