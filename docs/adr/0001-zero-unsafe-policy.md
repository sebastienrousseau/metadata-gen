# 0001. `#![forbid(unsafe_code)]`, pure Rust, no FFI

- **Status:** accepted
- **Date:** 2024-10-03 (recorded 2026-09-06)

## Context

The crate parses untrusted text (front matter, HTML) on behalf of static
site generators. A memory-safety bug here is a bug in every site built
with it. Nothing in the problem needs raw pointers, and every parser it
relies on (`noyalib`, `toml`, `serde_json`, `quick-xml`) is itself pure
Rust.

## Decision

`#![forbid(unsafe_code)]` at the crate root; no C dependencies, no FFI.
A dependency that requires `unsafe` in this crate's own code, or that
pulls in a C build, is a reason to pick a different dependency.

## Consequences

- The compiler proves the absence of unsafe blocks; there is nothing to
  audit by hand.
- Performance work is confined to safe abstractions. If a hot path ever
  needs `unsafe`, that is a new ADR superseding this one, not a local
  `#[allow]`.
