# 0005. Formats and integrations are Cargo features; the core is `no_std + alloc`

- **Status:** accepted
- **Date:** 2026-10-09

## Context

Up to 0.0.7 every consumer compiled all three front-matter parsers, the
`quick-xml` scanner, `regex`, `dtt` and a trimmed Tokio, whatever it
parsed. A static-site generator that only reads YAML paid for TOML,
JSON and an async runtime it never started, and the crate could not be
used in `no_std` targets or a WASM component at all, because it named
`std` unconditionally.

Two alternatives were considered: keep one monolithic build and accept
the cost, or split the parsers into separate crates. The first leaves
the problem; the second multiplies release work across a family that
already ships in lockstep.

## Decision

Each capability is a Cargo feature: `std`, `yaml`, `toml`, `json`,
`html` (`<meta>` extraction, needs `std`) and `tokio` (the async file
helpers, needs `std`). The default set enables all of them, so a
default build offers what 0.0.7 offered.

With `default-features = false` the crate is `no_std + alloc`. Every
dependency is declared `default-features = false` and the crate's own
`std` feature turns their `std` support back on. `MetadataMap` is a
`HashMap` with `std` and a `BTreeMap` without it; both have the
`get`, `insert`, `iter` and `contains_key` surface the crate uses. This
amends [ADR-0002](0002-flat-string-metadata.md) for `no_std` builds
only.

A fence whose format is compiled out is reported as
`MetadataError::UnsupportedFormatError`, never as "no front matter", so
a missing feature cannot look like an empty document.

The regex fence matcher gives way to a `memchr` scanner and `dtt` to
the `time` crate, because neither `regex` nor `dtt` builds without
`std`.

## Consequences

- A consumer that parses one format compiles one parser.
- `cargo hack check --feature-powerset --all-targets` is the gate that
  every combination still builds; the test suites run on the default
  set, and `metadata::scan` carries tests for a YAML-only build.
- `cargo build --no-default-features --features yaml --target
  thumbv7em-none-eabihf` is how the `no_std` claim is checked.
- Code that uses `std` must sit behind `feature = "std"`; reaching for
  `std::` in shared code breaks the `no_std` build, not the default one.
