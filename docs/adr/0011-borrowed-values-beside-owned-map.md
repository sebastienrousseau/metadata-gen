<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 0011. Borrowed (`Cow`) values are a separate API beside the owned flat map

- **Status:** accepted
- **Date:** 2026-11-23 (records the 0.0.10 core-API change)

## Context

`extract_metadata` returns `Metadata`, a map of owned `String`s
([ADR-0002](0002-flat-string-metadata.md)), so every key and value is
copied out of the document even when the caller only reads a few of
them and drops the map before the document. Issue #46 asked for a
zero-copy shape. Two ways were open: change `Metadata` to hold
`Cow<'a, str>` (a lifetime on the type every consumer names, and a
breaking change for all of them), or add a second, borrowing API.

## Decision

`Metadata` stays owned. 0.0.10 adds `extract_metadata_cow`, which
returns `CowMetadata<'a>`: the same dotted keys, held in a
`BTreeMap<Cow<'a, str>, Cow<'a, str>>`. A top-level key and a string
value written without escapes are `Cow::Borrowed` slices of the input;
nested keys, sequences, numbers, booleans and unescaped strings are
owned. `CowMetadata::into_owned` converts to `Metadata` for callers that
must outlive the document.

The map is built from the streaming event walk (`for_each_event`), not
from the owned flattener, so a borrowed value is never copied first.

## Consequences

- Existing callers are untouched; the borrowing API is opt-in.
- `CowMetadata` iterates in key order (`BTreeMap`); `Metadata` with
  `std` does not.
- The two paths can disagree at the edges, and the function's docs list
  where: numbers are rendered with Rust's `Display` (`1.0` reads `1`),
  YAML is parsed as written (the multi-line quoted-scalar repair of
  `extract_metadata` would need a copy), and a mapping inside a
  sequence renders as `{k: v}`. A change to either path's output is a
  breaking change under the README's output-stability guarantee.
- `CowMetadata::borrowed_values` counts borrowed values, so the
  zero-copy claim is testable (`tests/test_events.rs`).
