<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Frequently asked questions

## Why three formats?

Static-site generators disagree: Jekyll and most Markdown tools use
YAML (`---`), Hugo and Zola also use TOML (`+++`), and some tools put a
JSON object at the top. One crate that reads all three gives a pipeline
one code path. Each format is a Cargo feature, so a build that reads
only YAML compiles only the YAML parser
([ADR-0005](adr/0005-feature-flags-and-no-std.md)).

## How is the format chosen?

By the first non-blank characters: `---` is YAML, `+++` is TOML, `{`
is JSON. The block is never re-tried as another format
([ADR-0007](adr/0007-front-matter-detection-order.md)).

## How does this compare with `gray_matter`?

Both parse YAML, TOML and JSON front matter into a serde type and
return the body. `metadata-gen` also offers a flat dotted-key map,
`<meta>` tag generation and extraction, parse limits and a `no_std`
build; `gray_matter` offers excerpts and custom delimiters. The full
table, with the source each cell comes from, is in
[`COMPARISON.md`](COMPARISON.md).

## Is it async-only?

No. Every extraction function is synchronous and needs no runtime.
Async file reading is optional: `tokio` (default), `futures-io` (smol,
async-std) and `embedded-io-async` (Embassy) each add a reader behind
their own feature.

## Can I use it in `no_std` or WebAssembly?

Yes. With `default-features = false` and the formats you need (for
example `features = ["yaml"]`) the crate is `no_std + alloc`; CI builds
it for `thumbv7em-none-eabihf` and RISC-V targets. For WASI hosts,
`wasi/` builds a WASI 0.2 component with a WIT interface, and
[`deploy/`](deploy/README.md) shows it on Cloudflare Workers.

## What is the MSRV policy?

The floor is Rust 1.89.0, the `rust-version` in `Cargo.toml`, checked
by a CI job on that exact toolchain. It is the floor of the
first-party `noyalib` and `oxml` crates, and the `time` dependency
alone needs 1.88.0. Raising it is a documented change in the
changelog; see [`POLICIES.md`](POLICIES.md#1-msrv-minimum-supported-rust-version).

## How are dates parsed?

`process_metadata` normalises `date` to `YYYY-MM-DD`. ISO 8601 and RFC
3339 dates are read as written; a slash date such as `01/02/2024` is
day-first by default and month-first with
`ProcessOptions::default().date_order(DateOrder::MonthFirst)`.
`extract_metadata` alone leaves the value as written.

## How do I add custom required fields?

`ProcessOptions::new().require("author")` adds one (the defaults are
`title` and `date`), `required_fields([...])` replaces the list, and
`process_metadata_with` or `options.process(&metadata)` reports the
first missing one as `MetadataError::MissingFieldError`. For a
compile-time check, `Metadata::builder()` only builds once `title` and
`date` are set.

## How is HTML escaping handled?

`generate_metatags` and `MetaTag::render` pass every attribute value
through `escape_attribute`, so a title such as `"><script>` cannot
break out of the `content` attribute. `escape_html` escapes `&`, `<`,
`>`, `"` and `'` in one pass. `extract_meta_tags` decodes entity
references in the values it reads.

## How do I extract typed structs?

Derive `serde::Deserialize` and call `extract_typed::<T>(content)`;
`extract_typed_borrowed` lets `&str` fields point into the document
instead of allocating. The flat map (`extract_metadata`) loses types by
design ([ADR-0002](adr/0002-flat-string-metadata.md)).

## Can I sign front matter?

With the `pqc-sign` feature, `Metadata::sign` and `Metadata::verify`
use ML-DSA-65 (FIPS 204) over a documented canonical encoding
([`spec/canonical-encoding.md`](spec/canonical-encoding.md)). The
signature covers the extracted metadata, not the document body.

## Can an AI agent call it?

`examples/mcp_server.rs` is a Model Context Protocol server over stdio
with `extract_metadata`, `extract_meta_tags` and `escape_html` tools;
[`MCP.md`](MCP.md) shows how to register it.

## Where do I report a vulnerability?

Privately, as [`SECURITY.md`](../SECURITY.md) describes, never in a
public issue.
