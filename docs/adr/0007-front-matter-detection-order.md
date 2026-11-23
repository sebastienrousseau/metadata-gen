<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 0007. Front matter is detected by its opening delimiter: YAML, then TOML, then JSON

- **Status:** accepted
- **Date:** 2026-11-23 (records behaviour in place since 0.0.5)

## Context

A document may open with `---` (YAML), `+++` (TOML) or `{` (a JSON
object). The caller does not say which, so the crate has to decide,
and the decision is visible output: a different choice turns the same
document into different metadata or into an error. Content-sniffing
(try each parser until one succeeds) was the alternative; it makes the
result depend on which parsers are compiled in and on parser error
recovery, and it parses the block up to three times.

## Decision

`metadata::scan` looks only at the first non-blank characters after an
optional UTF-8 byte-order mark, in a fixed order:

1. `---` alone on its line opens a YAML block that a later line
   starting with `---` closes.
2. `+++` alone on its line opens a TOML block, closed the same way.
3. `{` opens a JSON object, which ends where `serde_json`'s streaming
   deserializer finishes the first value.

The delimiter decides the format; the block is never re-tried as
another format. An opening fence without a closing one is reported
with its byte offset. A block whose format is compiled out is reported
as `UnsupportedFormatError` ([ADR-0005](0005-feature-flags-and-no-std.md)),
never as "no front matter". A JSON block with a syntax error still
counts as JSON, so the caller sees the parser's message and position.

The fence search uses `memchr::memmem`, not a regular expression, so
the scan is linear in the document and builds without `std`.

## Consequences

- One parse per document, and the same answer in every feature set.
- A document that starts with `{` for another reason (a templating
  directive, say) is treated as JSON front matter and fails as JSON.
  Such callers should not pass the document to `extract_metadata`.
- The rules are listed in the `metadata::scan` module docs and pinned
  by its tests; changing them is a breaking change under the README's
  output-stability guarantee.
