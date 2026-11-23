// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The YAML value-path replay, driven directly: noyalib only asks for it
//! on a few shapes, so the tests build the value tree themselves.

use super::*;
use alloc::vec;

fn replay(yaml: &str, skip: usize) -> Vec<Event<'static>> {
    let value: noyalib::Value = noyalib::from_str(yaml).unwrap();
    let mut out = Vec::new();
    let mut sink = Sink::new(|e| {
        out.push(e);
        ControlFlow::Continue(())
    });
    sink.skip = skip;
    emit_value::<_, noyalib::Error>(&value, &mut sink).unwrap();
    out
}

#[test]
fn replays_every_shape() {
    let events = replay("a: [1, 2.5, true, ~, s]\nb: {c: !t d}\n", 0);
    assert_eq!(
        events,
        vec![
            Event::MapStart,
            Event::Key(Cow::Owned("a".into())),
            Event::SeqStart,
            Event::Scalar(Scalar::Int(1)),
            Event::Scalar(Scalar::Float(2.5)),
            Event::Scalar(Scalar::Bool(true)),
            Event::Scalar(Scalar::Null),
            Event::Scalar(Scalar::Str(Cow::Owned("s".into()))),
            Event::SeqEnd,
            Event::Key(Cow::Owned("b".into())),
            Event::MapStart,
            Event::Key(Cow::Owned("c".into())),
            Event::Scalar(Scalar::Str(Cow::Owned("d".into()))),
            Event::MapEnd,
            Event::MapEnd,
        ]
    );
}

#[test]
fn replay_skips_what_streaming_delivered() {
    assert_eq!(
        replay("a: 1\n", 2),
        vec![Event::Scalar(Scalar::Int(1)), Event::MapEnd,]
    );
}

#[test]
fn replay_stops_when_asked() {
    let value: noyalib::Value = noyalib::from_str("[1, 2]").unwrap();
    let mut seen = 0;
    let mut sink = Sink::new(|_| {
        seen += 1;
        ControlFlow::Break(())
    });
    assert!(emit_value::<_, noyalib::Error>(&value, &mut sink).is_err());
    assert!(sink.stopped);
    assert_eq!(seen, 1);
}

#[test]
fn scalars_display_as_text() {
    assert_eq!(Scalar::Str(Cow::Borrowed("x")).to_string(), "x");
    assert_eq!(Scalar::Float(1.5).to_string(), "1.5");
    assert_eq!(Scalar::Bool(false).to_text(), "false");
    assert_eq!(Scalar::Int(-3).to_text(), "-3");
}

#[test]
fn yaml_syntax_errors_are_parse_errors() {
    let err = events("---\na: b: c\n---\n").unwrap_err();
    assert!(
        matches!(
            err,
            MetadataError::Parse {
                format: FrontMatterFormat::Yaml,
                ..
            }
        ),
        "{err:?}"
    );
}
