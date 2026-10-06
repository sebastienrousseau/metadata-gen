<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/metadata-gen/v1/logos/metadata-gen.svg" alt="metadata-gen logo" width="128" />
</p>

<h1 align="center">metadata-gen</h1>

<p align="center">
  Front matter in, metadata and SEO meta tags out. YAML, TOML and JSON, with zero <code>unsafe</code> code.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/metadata-gen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/metadata-gen/ci.yml?branch=main&style=for-the-badge&label=build" alt="Build" /></a>
  <a href="https://crates.io/crates/metadata-gen"><img src="https://img.shields.io/crates/v/metadata-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Registry" /></a>
  <a href="https://docs.rs/metadata-gen"><img src="https://img.shields.io/badge/docs.rs-metadata--gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/metadata-gen"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/metadata-gen?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
  <a href="https://www.bestpractices.dev/projects/14536"><img src="https://img.shields.io/cii/level/14536?style=for-the-badge&label=OpenSSF%20Best%20Practices&logo=openssf" alt="OpenSSF Best Practices" /></a>
  <a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License: Apache-2.0 OR MIT" /></a>
  <a href="docs/POLICIES.md"><img src="https://img.shields.io/badge/MSRV-1.88.0-93450a.svg?style=for-the-badge&logo=rust" alt="MSRV 1.88.0" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install): Cargo, source
- [Requirements](#requirements): toolchain floor, platforms
- [Quick Start](#quick-start): front matter to meta tags in ten lines

**The metadata-gen ecosystem**

- [The metadata-gen ecosystem](#the-metadata-gen-ecosystem): front-matter and content generation companion crates

**Library reference**

- [Capabilities at a glance](#capabilities-at-a-glance): the current surface by theme
- [Ecosystem comparison](#ecosystem-comparison): how the crate compares with neighbouring crates
- [Benchmarks](#benchmarks): measured numbers with the host stated
- [Features](#features): module-level capability list
- [Configuration](#configuration): core options
- [Examples](#examples): runnable example index

**Operational**

- [When not to use metadata-gen](#when-not-to-use-metadata-gen): limitations
- [Development](#development): make targets, fuzzing, CI
- [Security](#security): guarantees and compliance
- [Documentation](#documentation): all reference docs
- [Stability guarantees](#stability-guarantees): SemVer axis, output stability, minimum toolchain discipline
- [License](#license)

---

## Install

### As a Rust library

```toml
[dependencies]
metadata-gen = "0.0.8"
```

Or from the command line:

```bash
cargo add metadata-gen
```

There is no CLI binary. `metadata-gen` is a library crate and ships no `[[bin]]`.

### Build from source

```bash
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen
make          # check + clippy + test
```

### Cargo features

None. Every capability is active by default, and the manifest declares no optional features.

---

## Requirements

- **Rust 1.88.0 or newer.** `rust-version` in `Cargo.toml` is the floor and Cargo enforces it; CI builds on stable across Linux, macOS and Windows.
- **A `std` platform.** The crate uses `std` unconditionally today. A `no_std + alloc` core is roadmap work.
- **No async runtime is required.** Every synchronous entry point works without one. `async_extract_metadata_from_file` is a convenience for callers who run Tokio; the dependency is trimmed to `fs`, `io-util`, `rt` and `macros`.

---

## Quick Start

```rust
use metadata_gen::extract_and_prepare_metadata;

let content = "---\n\
title: Hello, world!\n\
description: A short greeting\n\
keywords: rust, frontmatter, seo\n\
---\n\
# Body starts here";

let (metadata, keywords, tags) =
    extract_and_prepare_metadata(content).expect("valid front matter");

assert_eq!(metadata.get("title"), Some(&"Hello, world!".to_string()));
assert_eq!(keywords, vec!["rust", "frontmatter", "seo"]);
assert!(tags.primary.contains("description"));
```

`extract_and_prepare_metadata` detects the format, flattens the metadata into a map, extracts keywords, and generates formatted meta tags in one call.

---

## The metadata-gen ecosystem

The family releases along the `0.0.x` line, with each repository owning a focused role in static site and content pipelines.

| Component | Purpose | Use case |
| :--- | :--- | :--- |
| [`metadata-gen`](https://github.com/sebastienrousseau/metadata-gen) | Extract and process front matter in YAML, TOML, JSON; generate meta tags | Content pipelines, documentation, static site generators |
| [`frontmatter-gen`](https://github.com/sebastienrousseau/frontmatter-gen) | Front-matter validation and schema generation | Build-time front-matter validation |
| [`mdx-gen`](https://github.com/sebastienrousseau/mdx-gen) | MDX compiler and component renderer | Embedded interactive components in Markdown |
| [`sitemap-gen`](https://github.com/sebastienrousseau/sitemap-gen) | XML sitemap generator conforming to sitemaps.org | Search engine index generation |
| [`rssgen`](https://github.com/sebastienrousseau/rssgen) | RSS, Atom, and JSON feed generator | Content syndication feeds |
| [`staticdatagen`](https://github.com/sebastienrousseau/staticdatagen) | Static data engine for template processing | Template data hydration |

---

## Capabilities at a glance

| Area | Capability | Status |
| :--- | :--- | :--- |
| Front-matter extraction | YAML, TOML, and JSON delimited headers | Stable |
| Document separation | Split front-matter block and document body in one call | Stable |
| Typed extraction | Deserialise front matter into user-defined structs | Stable |
| Flat metadata | Flatten nested structures to dot-separated string maps | Stable |
| Meta tag generation | Open Graph, Twitter, Apple, Microsoft, and primary tags | Stable |
| Meta tag extraction | Single streaming pass over HTML documents with `quick-xml` | Stable |
| Utilities | Single-pass HTML entity escaping and async file loading | Stable |

---

## Ecosystem comparison

This summary identifies API shape, not a universal winner. Workload-specific trade-offs and the evidence behind each cell are documented separately.

| Project | YAML | TOML | JSON | Typed | Body returned | Meta tags |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`metadata-gen`** | Yes | Yes | Yes | Yes | Yes | Yes |
| `gray_matter` | Yes | Yes | Yes | Yes | Yes | No |
| `yaml-front-matter` | Yes | No | No | Yes | Yes | No |
| `matter` | Yes | No | No | No | Yes | No |

The matrix reflects each crate's documented surface at the time of this release.

---

## Benchmarks

Measured with Criterion on an Apple A18 Pro, rustc 1.98.0, single thread. Middle estimate of the confidence interval; run `cargo bench` to reproduce on your own hardware.

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| `extract_metadata` (YAML, 1 KB) | 631 µs (1.5 MiB/s) | Apple A18 Pro, rustc 1.98.0 |
| `extract_metadata` (YAML, 10 KB) | 1.81 ms (5.4 MiB/s) | Apple A18 Pro, rustc 1.98.0 |
| `extract_metadata` (YAML, 1 MB) | 181 ms (5.5 MiB/s) | Apple A18 Pro, rustc 1.98.0 |
| `extract_meta_tags` (1 KB) | 50 µs (18.7 MiB/s) | Apple A18 Pro, rustc 1.98.0 |
| `extract_meta_tags` (1 MB) | 34.9 ms (28.7 MiB/s) | Apple A18 Pro, rustc 1.98.0 |
| `escape_html` (10 KB) | 31.9 µs (295 MiB/s) | Apple A18 Pro, rustc 1.98.0 |
| `extract_and_prepare_metadata` (~250 B) | 23 µs | Apple A18 Pro, rustc 1.98.0 |

Reproduce with `cargo bench --all-features`; the harnesses live in [`benches/`](benches/).

---

## Features

- **Three front-matter shapes**: YAML (`---` ... `---`), TOML (`+++` ... `+++`), and JSON (`{ ... }` at top of file).
- **Dual extraction APIs**:
  - `extract_metadata`: returns flat `Metadata` (`HashMap<String, String>`) with dot-separated keys (`author.name`), ideal for templates.
  - `extract_typed::<T>`: hands the raw block to serde, returning your strongly typed struct.
- **Document body access**: `extract_metadata_with_body` returns `(Metadata, &str)` with the body following the closing delimiter. `detect_front_matter` exposes format, raw block, and byte offset without parsing.
- **Processing and normalization**: `process_metadata` and `process_metadata_with` normalize dates to `YYYY-MM-DD`, verify required fields, and derive URL slugs from titles.
- **Meta tag synthesis**: `generate_metatags` creates grouped tags (`primary`, `og`, `twitter`, `apple`, `ms`) with attribute values passed through `escape_html`.
- **Streaming HTML tag extraction**: `extract_meta_tags` extracts `<meta>` tags in a single streaming pass using `quick-xml` without full DOM allocation.
- **HTML escaping and file utilities**: single-pass, single-allocation `escape_html` and `unescape_html`, plus Tokio-backed `async_extract_metadata_from_file`.

---

## Configuration

`process_metadata` uses a default policy: `title` and `date` are required, and `slug` is derived from `title`. `process_metadata_with` accepts caller-configured `ProcessOptions`:

```rust
use metadata_gen::{process_metadata_with, Metadata, ProcessOptions};
use std::collections::HashMap;

let options = ProcessOptions::default()
    .required_fields(["title", "author"])
    .derive_slug(false);

let mut map = HashMap::new();
map.insert("title".to_string(), "Hello".to_string());
map.insert("author".to_string(), "Ada".to_string());

let processed = process_metadata_with(&Metadata::new(map), &options).unwrap();
assert!(!processed.contains_key("slug"));
```

| Option | Default | Effect |
| :--- | :--- | :--- |
| `required_fields` | `["title", "date"]` | Missing field returns `MissingFieldError` naming it |
| `derive_slug` | `true` | Derives `slug` from `title` when absent |

`ProcessOptions` is `#[non_exhaustive]`, so options can be added without breaking releases.

---

## Examples

Run any example with `cargo run --example <name>`:

| Example | Shows |
| :--- | :--- |
| `lib_example` | The high-level `extract_and_prepare_metadata` flow |
| `metadata_example` | Per-format extraction, nested tables, typed extraction, the body |
| `metatags_example` | Generating `<meta>` groups and reading them back |
| `utils_example` | HTML escape/unescape and the async file helper |
| `error_example` | Every `MetadataError` variant and how to recover |

`make examples` runs all examples; CI executes the same suite on every push.

---

## When not to use metadata-gen

- **You need `no_std` or a WASM component today.** The crate uses `std` unconditionally and pulls `tokio` for the async file helper. A `no_std + alloc` core is roadmap work.
- **You need element-level access to arrays of objects from the flat map.** `[a, b]` is rendered as a string in the flat map by design. Use `extract_typed::<T>` instead, which preserves structure.
- **You need to round-trip front matter byte-for-byte.** The crate parses; it does not preserve comments, key order or quoting style, and there is no serialiser back to a fenced block.
- **You need every `<meta>` element from arbitrary broken HTML.** Extraction stops at the first unrecoverable reader error and returns what was found so far. A full HTML5 parser (`html5ever`, `scraper`) is the right tool if you need error recovery over broken whole pages.

---

## Development

```bash
make              # check + clippy + test
make test         # all tests, all features
make clippy       # lints, warnings denied
make fmt          # formatting check
make lint         # markdownlint + codespell + REUSE
make doc          # rustdoc with warnings denied
make coverage     # line coverage gate (98%)
make miri         # lib tests under Miri
make proptest     # property-based tests (involution + round-trip)
make loom         # concurrency testing via Loom model checker
make kani         # formal verification proofs via Kani
make mutants      # mutation testing kill rate (cargo-mutants)
make fuzz         # build every target, replay corpus and regressions
make examples     # run every example
make bench-smoke  # compile and run each bench once
make versions     # every version-bearing file agrees
make complexity   # per-function complexity ceilings
make links        # every Markdown link resolves
make msrv         # builds on the declared minimum Rust
make semver       # public API against the last release
make hack         # every feature combination compiles
make distcheck    # package and verify the archive
make deny / vet / audit   # supply chain
```

[`DEVELOPMENT.md`](DEVELOPMENT.md) maps each CI job to its local equivalent and explains reproduction steps. [`docs/MIGRATION.md`](docs/MIGRATION.md) lists what changes for consumers between releases.

---

## Security

**Reporting:** never open a public issue for a vulnerability. See [`SECURITY.md`](SECURITY.md) for disclosure instructions.

- `#![forbid(unsafe_code)]` proves the absence of unsafe code ([ADR-0001](docs/adr/0001-zero-unsafe-policy.md)).
- No C dependencies, no FFI, no network I/O, no environment reads.
- Meta-tag attribute values are escaped on generation, preventing markup injection.
- Supply chain audited via `cargo-audit`, `cargo-deny`, and `cargo-vet` with a locked exemption baseline.
- First-party dependencies `noyalib` and `dtt` are pinned exactly ([ADR-0004](docs/adr/0004-first-party-exact-pins.md)).
- Property-based testing via Proptest for parser round-trips and HTML escape involution.
- Concurrency testing scaffold via Loom (`tests/loom_smoke.rs`) for thread schedule exploration.
- Formal verification via Kani (`tests/kani/`) proving HTML escape totality and ASCII round-trip.
- Mutation testing via `cargo-mutants` configured with `.cargo/mutants.toml`.
- Three fuzz targets replaying seed and regression corpora per push.

---

## Documentation

The canonical entry points across the repository family:

- **[API reference](https://docs.rs/metadata-gen)**: rustdoc on docs.rs
- **[Developer docs](DEVELOPMENT.md)**: toolchain, task map, reproducing CI gates
- **[Architecture](docs/ARCHITECTURE.md)**: module map, pipeline, design decisions
- **[Engineering policies](docs/POLICIES.md)**: MSRV, SemVer, security, concurrency
- **[Decision records](docs/adr/README.md)**: irreversible architectural decisions

| Document | Covers |
| :--- | :--- |
| [`CHANGELOG.md`](CHANGELOG.md) | Per-release notes, Keep a Changelog format |
| [`SECURITY.md`](SECURITY.md) | Disclosure policy, supported versions, security design |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Branch and commit conventions, PR expectations |
| [`GOVERNANCE.md`](GOVERNANCE.md) | Project stewardship, how changes land |
| [`SUPPORT.md`](SUPPORT.md) | Support channels and expectations |
| [`AGENTS.md`](AGENTS.md) | Invariants for AI-assisted contributions |

---

## Stability guarantees

### Versioning

[SemVer 2.0.0](https://semver.org) is followed, with the pre-1.0 posture that releases increment strictly by `+0.0.1` along the `0.0.x` line. Every breaking change is documented in [`CHANGELOG.md`](CHANGELOG.md).

### Output stability

What the crate produces is part of the API: a change to how a document flattens, which key a value lands under, or what a meta-tag group renders is treated as a breaking change even when no Rust signature moves.

### Deprecations

Deprecations live for at least two releases with a `#[deprecated]` attribute naming the replacement before removal.

### Minimum-toolchain discipline

The floor is **Rust 1.88.0**, declared as `rust-version` in `Cargo.toml`. Raising it is a breaking change and occurs only on releases with rationale documented in the changelog. See [`docs/POLICIES.md`](docs/POLICIES.md) for the complete policy.

### Version-bearing files

Checked against the manifest by `scripts/verify-release-versions.sh` before a tag exists, ensuring install snippets and metadata stay synchronised.

---

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you shall be dual-licensed as above, without any additional terms or conditions.

<p align="right"><a href="#metadata-gen">Back to top</a></p>
