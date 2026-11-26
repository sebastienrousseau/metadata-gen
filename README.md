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
  <a href="docs/POLICIES.md"><img src="https://img.shields.io/badge/MSRV-1.89.0-93450a.svg?style=for-the-badge&logo=rust" alt="MSRV 1.89.0" /></a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/sebastienrousseau/metadata-gen/840ff82875aa853f6e0cc9e852350c3f921c8891/.github/demo.gif" alt="A Markdown file's YAML front matter, then the flat metadata map metadata-gen extracts from it" width="100%" />
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
- [Async](#async): Tokio, any `futures-io` runtime, and embedded readers
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
metadata-gen = "0.0.10"
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

Every format and integration is a feature, and all of them are on by default, so a default build offers everything 0.0.7 did. Turn off what you do not parse with `default-features = false`.

| Feature | Default | Enables |
| :--- | :--- | :--- |
| `std` | yes | File helpers in `metadata_gen::io`, `std::io::Error` in `MetadataError`, `HashMap`-backed `MetadataMap` |
| `yaml` | yes | `---` front matter (`noyalib`) |
| `toml` | yes | `+++` front matter (`toml`) |
| `json` | yes | JSON-object front matter (`serde_json`) |
| `html` | yes | `extract_meta_tags` (`oxml`); implies `std` |
| `tokio` | yes | `metadata_gen::tokio` and `async_extract_metadata_from_file`; implies `std` |
| `futures-io` | no | `metadata_gen::futures_io`: read from any `futures_io::AsyncRead` (smol, async-std, `futures`); implies `std` |
| `embedded-io-async` | no | `metadata_gen::embedded_io`: read from an `embedded_io_async::Read` (Embassy) into a caller buffer; works without `std` |

```toml
# YAML only, no standard library:
metadata-gen = { version = "0.0.10", default-features = false, features = ["yaml"] }
```

A fence whose format is not compiled in is reported as `UnsupportedFormatError`, not as missing front matter.

---

## Requirements

- **Rust 1.89.0 or newer.** `rust-version` in `Cargo.toml` is the floor and Cargo enforces it; a CI job builds on exactly 1.89.0, and the test suite runs on stable across Linux, macOS and Windows. The floor is the one the first-party `noyalib` and `oxml` crates declare; `time` alone needs 1.88.0 ([`docs/POLICIES.md`](docs/POLICIES.md)).
- **`alloc`, with `std` optional.** With `default-features = false` the crate is `no_std + alloc` and `MetadataMap` is a `BTreeMap`; `cargo build --no-default-features --features yaml --target thumbv7em-none-eabihf` builds.
- **No async runtime is required.** Every synchronous entry point works without one. The `tokio` feature adds `metadata_gen::tokio` for callers who run Tokio; the dependency is trimmed to `fs` and `io-util`.

### Embedded targets

The `embedded` CI job builds the `no_std` core for each target below on every pull request; a `std` call in that path fails the build, and so does a `yaml` object over 50,000 B (#61). Sizes were measured locally on macOS with rustc 1.100.0 by the scripts named in each metric (for the same commit, CI on Linux reported the `yaml` object 4% lower, 72,400 B against 75,389 B):

| Target | Features | Metric | Size |
| :--- | :--- | :--- | ---: |
| `thumbv7em-none-eabihf` | `yaml,toml,json` | object | 64,056 B |
| `thumbv7em-none-eabihf` | `yaml` | object (budget 50,000 B) | 48,240 B |
| `thumbv7em-none-eabihf` | `yaml,embedded-io-async` | object | 48,344 B |
| `thumbv7em-none-eabihf` | `yaml` | linked firmware | 164,041 B |
| `riscv32imac-unknown-none-elf` | `yaml,toml,json` | object | 70,370 B |
| `riscv32imac-unknown-none-elf` | `yaml` | linked firmware | 194,255 B |
| `riscv32imc-unknown-none-elf` | `json` | object | 53,483 B |

- **object**: `scripts/embedded_size.sh <target> <features>`, the text + data + bss of metadata-gen's own rlib at `opt-level = "s"` without LTO. It leaves out every dependency.
- **linked firmware**: `scripts/embedded_firmware_size.sh <target>`, the `.text + .rodata + .data` that a yaml-only `extract_metadata` call adds to a minimal `no_std` image linked with LTO, `opt-level = "s"` and `--gc-sections` (`embedded/size-probe`). It includes `noyalib` and the `core` formatting and float code it pulls in; `opt-level = "z"` brings it to 135,101 B on `thumbv7em-none-eabihf`. Most of it is not this crate's code, so the 50,000 B budget applies to the object figure. The `riscv32imac` figure is not run in CI.

`riscv32imc-unknown-none-elf` has no atomic read-modify-write, so only the JSON format builds there: `noyalib` and `toml` use `Arc`.

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

// The same call reads TOML (`+++`) and JSON (`{`) front matter.
use metadata_gen::extract_metadata;

let toml = extract_metadata("+++\ntitle = \"From TOML\"\n[author]\nname = \"Ada\"\n+++\nbody").unwrap();
assert_eq!(toml.get("author.name").unwrap(), "Ada");

let json = extract_metadata("{\"title\": \"From JSON\", \"draft\": false}\nbody").unwrap();
assert_eq!(json.get("draft").unwrap(), "false");
```

`extract_and_prepare_metadata` detects the format, flattens the metadata into a map, extracts keywords, and generates formatted meta tags in one call. `extract_metadata` returns only the map; `extract_typed::<T>` deserialises the block into your own struct.

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
| Meta tag extraction | Single streaming pass over HTML documents with `oxml` | Stable |
| Utilities | Single-pass HTML entity escaping and async file loading | Stable |
| WASI 0.2 component | `extract`, `extract-meta-tags`, `escape-html` over a WIT interface (`wasi/`) | New in 0.0.10 |
| Agent tools | MCP server example over stdio (`examples/mcp_server.rs`) | Example |

---

## Ecosystem comparison

This summary identifies API shape, not a universal winner. Workload-specific trade-offs and the evidence behind each cell are documented separately.

| Project | YAML | TOML | JSON | Typed | Body returned | Meta tags |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`metadata-gen`** | Yes | Yes | Yes | Yes | Yes | Yes |
| `gray_matter` | Yes | Yes | Yes | Yes | Yes | No |
| `yaml-front-matter` | Yes | No | No | Yes | Yes | No |
| `matter` | Split only | Split only | No | No | Yes | No |

The matrix reflects each crate's published source at the time of this release ("split only": the block is separated from the body but not parsed). See [`docs/COMPARISON.md`](docs/COMPARISON.md) for the evidence and the complete matrix.

---

## Benchmarks

Measured with Criterion on a GitHub-hosted `ubuntu-latest` runner (Ubuntu 26.04, x86_64), rustc 1.100.0, single thread, on 2026-11-23 ([Bench run 45157730562](https://github.com/sebastienrousseau/metadata-gen/actions/runs/45157730562), its `criterion` artifact). Mean of the samples from each benchmark's `estimates.json`. Shared CI runners are noisy, so read these as orders of magnitude; run `cargo bench` to measure on your own hardware.

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| `extract_metadata` (1 KiB YAML block) | 19.4 µs (49.5 MiB/s) | ubuntu-latest, rustc 1.100.0 |
| `extract_metadata` (32 KiB YAML block) | 588 µs (53.1 MiB/s) | ubuntu-latest, rustc 1.100.0 |
| `extract_metadata` (1 KiB header, 10 MiB document) | 20.2 µs | ubuntu-latest, rustc 1.100.0 |
| `extract_meta_tags` (1 KiB) | 21.4 µs (43.3 MiB/s) | ubuntu-latest, rustc 1.100.0 |
| `extract_meta_tags` (1 MiB) | 23.9 ms (41.9 MiB/s) | ubuntu-latest, rustc 1.100.0 |
| `escape_html` (10 KiB) | 17.0 µs (554 MiB/s) | ubuntu-latest, rustc 1.100.0 |
| `escape_html` (10 MiB) | 18.6 ms (519 MiB/s) | ubuntu-latest, rustc 1.100.0 |
| `unescape_html` (10 KiB) | 22.8 µs (873 MiB/s) | ubuntu-latest, rustc 1.100.0 |
| `extract_and_prepare_metadata` (292 B) | 4.95 µs | ubuntu-latest, rustc 1.100.0 |
| Sustained `extract_metadata` (1 KiB post) | 38,400 per second | ubuntu-latest, rustc 1.100.0 |

The 10 MiB `extract_metadata` row costs the same as the 1 KiB one: extraction reads the front matter, not the body.

Reproduce with `cargo bench --all-features`; the harnesses live in [`benches/`](benches/).

---

## Features

- **Three front-matter shapes**: YAML (`---` ... `---`), TOML (`+++` ... `+++`), and JSON (`{ ... }` at top of file).
- **Dual extraction APIs**:
  - `extract_metadata`: returns flat `Metadata` (`HashMap<String, String>`) with dot-separated keys (`author.name`), ideal for templates.
  - `extract_typed::<T>`: hands the raw block to serde, returning your strongly typed struct; `extract_typed_borrowed` lets `&str` fields point into the document.
- **Zero-copy and streaming**: `extract_metadata_cow` returns the same dotted keys with values borrowed from the document where no unescaping was needed; `for_each_event` walks the block as `Event`s (keys, scalars, container boundaries) without building a map, and stops parsing when the callback returns `ControlFlow::Break`. `events_into` fills a buffer you own.
- **Builder**: `Metadata::builder()` is a typestate builder; `build()` does not compile until `title` and `date` are set.
- **Schema.org JSON-LD**: `schema_org::json_ld` maps front matter onto `Article`, `Product` or `Organization`, and `schema_org::breadcrumb_list` builds a `BreadcrumbList`; no JSON dependency, works without `std`.
- **Document body access**: `extract_metadata_with_body` returns `(Metadata, &str)` with the body following the closing delimiter. `detect_front_matter` exposes format, raw block, and byte offset without parsing.
- **Bounded parsing**: every parse runs under `ParseLimits` (block size, nesting depth); the YAML parser gets its strict budget preset on the flat and typed paths alike. `extract_metadata_with_limits` takes your own limits.
- **Errors that locate the fault**: `MetadataError::Parse` carries the format, the byte span inside the block and the parser's error; an unclosed fence reports its byte offset.
- **Processing and normalization**: `process_metadata` and `process_metadata_with` normalize dates to `YYYY-MM-DD` (`DD/MM/YYYY` by default, `MM/DD/YYYY` with `DateOrder::MonthFirst`), verify required fields, and derive URL slugs from titles.
- **Meta tag synthesis**: `generate_metatags` creates grouped tags (`primary`, `og`, `twitter`, `apple`, `ms`); Open Graph tags use `property=`, and attribute values pass through `escape_attribute`. `MetaTag` and `MetaTagGroups::iter` give the same tags as values.
- **Streaming HTML tag extraction**: `extract_meta_tags` extracts `<meta>` tags in a single streaming pass using `oxml` without full DOM allocation; `extract_meta_tags_lenient` also reports where a malformed page stopped the scan.
- **WASI 0.2 component**: `wasi/` builds the crate for `wasm32-wasip2` with the WIT interface in `wit/metadata-gen.wit`; CI checks it returns what the native crate returns. [`docs/deploy/`](docs/deploy/README.md) runs it on Cloudflare Workers, Spin and wasmCloud.
- **HTML escaping and file utilities**: single-pass, single-allocation `escape_html` and `unescape_html`, readers and files in `metadata_gen::io`, Tokio-backed helpers in `metadata_gen::tokio`, and runtime-agnostic async readers in `metadata_gen::futures_io` (smol, async-std) and `metadata_gen::embedded_io` (Embassy).

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
| `date_order` | `DateOrder::DayFirst` | How a slash-separated date such as `01/02/2024` is read |
| `optional_fields` | `[]` | Known but not required; `unknown_fields` lists keys outside both lists |

The same options read fluently as a processor: `MetadataProcessor::new().require("author").optional("description").process(&metadata)`.

`ProcessOptions` is `#[non_exhaustive]`, so options can be added without breaking releases.

---

## Async

Every synchronous entry point works without a runtime. Three optional modules read from an async source instead; each reads to the end, then gives what `extract_and_prepare_metadata` gives for the same text.

| Module | Feature | Reader it takes |
| :--- | :--- | :--- |
| `metadata_gen::tokio` | `tokio` (default) | `tokio::io::AsyncRead`; `extract_from_file(path)` opens a `tokio::fs::File` |
| `metadata_gen::futures_io` | `futures-io` | `futures_io::AsyncRead`: smol, async-std and `futures` readers, no runtime named |
| `metadata_gen::embedded_io` | `embedded-io-async` | `embedded_io_async::Read` (Embassy), into a buffer you supply; works without `std` |

With Tokio:

```rust
use metadata_gen::tokio::extract_from_reader;

let runtime = tokio::runtime::Runtime::new().unwrap();
let (metadata, _, _) = runtime
    .block_on(extract_from_reader(b"---\ntitle: Tokio\n---\n" as &[u8]))
    .unwrap();
assert_eq!(metadata["title"], "Tokio");
```

With smol, or any runtime whose readers implement `futures_io::AsyncRead` (`features = ["futures-io"]`). `smol::fs::File` is `async_fs::File` and `smol::block_on` is `futures_lite::future::block_on`, so this is the smol code with the crates named directly:

```rust
use metadata_gen::futures_io::extract_from_reader;

let dir = tempfile::tempdir().unwrap();
let path = dir.path().join("post.md");
std::fs::write(&path, "---\ntitle: smol\n---\nbody").unwrap();

let (metadata, _, _) = futures_lite::future::block_on(async {
    let file = async_fs::File::open(&path).await?;
    extract_from_reader(file).await
})
.unwrap();
assert_eq!(metadata["title"], "smol");
```

A Tokio reader goes into the same function through `tokio_util::compat` (`file.compat()`). On a device, `metadata_gen::embedded_io::extract_from_reader(reader, &mut buffer)` (`default-features = false, features = ["yaml", "embedded-io-async"]`) reads into `buffer` and allocates nothing for the input. `examples/tokio_example.rs`, `examples/futures_io_example.rs` and `examples/embedded_io_example.rs` run each path.

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
| `required_fields_example` | `MetadataProcessor` with an extra required field and an unknown-field check |
| `schema_org_example` | Schema.org `Article` and `BreadcrumbList` JSON-LD from front matter |
| `streaming_example` | Walking a 100 000-URL manifest as events, and stopping early |
| `tokio_example` | `metadata_gen::tokio`: a file and a reader on the Tokio runtime |
| `futures_io_example` | `metadata_gen::futures_io`: any `futures_io::AsyncRead`, no runtime named (needs `--features futures-io`) |
| `embedded_io_example` | `metadata_gen::embedded_io`: an `embedded_io_async::Read` into a fixed buffer, run on the host (needs `--features embedded-io-async`) |
| `mcp_server` | An MCP server over stdio exposing the crate as agent tools ([`docs/MCP.md`](docs/MCP.md)) |

[`examples/deploy/cloudflare-workers/`](examples/deploy/cloudflare-workers/) is a Cloudflare Worker that runs the WASI component.

`make examples` runs all examples; CI executes the same suite on every push.

---

## When not to use metadata-gen

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
make sbom         # the release SBOM lists the whole default build
make wasi         # WASI component: validate, size, native parity
```

[`DEVELOPMENT.md`](DEVELOPMENT.md) maps each CI job to its local equivalent and explains reproduction steps. [`docs/MIGRATION.md`](docs/MIGRATION.md) lists what changes for consumers between releases.

---

## Security

**Reporting:** never open a public issue for a vulnerability. See [`SECURITY.md`](SECURITY.md) for disclosure instructions.

- `#![forbid(unsafe_code)]` proves the absence of unsafe code ([ADR-0001](docs/adr/0001-zero-unsafe-policy.md)).
- No C dependencies, no FFI, no network I/O, no environment reads.
- Meta-tag attribute values are escaped on generation, preventing markup injection.
- Supply chain checked by `cargo-audit`, `cargo-deny` and `cargo-vet` (imported audit sets, an exemption count that may only shrink); releases attach CycloneDX SBOMs with build-provenance attestations. [`docs/SUPPLY-CHAIN.md`](docs/SUPPLY-CHAIN.md) explains each and how to verify a download.
- Front matter is parsed under `ParseLimits` (block size and depth), with the YAML parser's strict resource budgets on every path, the typed one included.
- The first-party dependencies `noyalib` and `oxml` share this crate's maintainer and are pinned exactly ([ADR-0004](docs/adr/0004-first-party-exact-pins.md)); that shared authorship is disclosed in [`docs/SUPPLY-CHAIN.md`](docs/SUPPLY-CHAIN.md) so you can weigh it.
- Property-based testing via Proptest: YAML, TOML and JSON round trips at 10 000 cases each and the ASCII escape involution at 100 000 per CI run, with the seed printed for replay.
- Concurrency testing scaffold via Loom (`tests/loom_smoke.rs`, `MAX_PREEMPTIONS` in `tests/loom/mod.rs`) for thread schedule exploration.
- Kani proofs ([`tests/kani/escape_html.rs`](tests/kani/escape_html.rs)) on every push that `escape_html` never panics on any one-char string and that `unescape_html(escape_html(s)) == s` for every one-char ASCII string, and enumeration ([`tests/test_escape_exhaustive.rs`](tests/test_escape_exhaustive.rs)) over every Unicode char and every ASCII string of up to three chars that escaping leaves no raw markup and `unescape_html` inverts it.
- Miri on the unit tests per push, and on every test target with `-Zmiri-strict-provenance` nightly.
- Mutation testing via `cargo-mutants` (`.cargo/mutants.toml`), gated at an 85 % kill rate on each pull request's changed lines and nightly on the whole crate.
- Three fuzz targets replaying seed and regression corpora per push and fuzzed for ten minutes each nightly; `fuzz/oss-fuzz/` is ready for OSS-Fuzz.
- To fuzz locally, install nightly Rust and `cargo install --locked cargo-fuzz`, then run a target from the repository root (`-max_total_time` is in seconds; new corpus entries land in `fuzz/corpus/<target>/`):

  ```bash
  cargo +nightly fuzz run fuzz_extract_metadata -- -max_total_time=600
  cargo +nightly fuzz run fuzz_extract_meta_tags -- -max_total_time=600
  cargo +nightly fuzz run fuzz_html_escape -- -max_total_time=600
  ```

  `make fuzz` builds every target and replays the committed corpus and regressions once, as CI does per push.

---

## Documentation

The canonical entry points across the repository family:

- **[API reference](https://docs.rs/metadata-gen)**: rustdoc on docs.rs
- **[Developer docs](DEVELOPMENT.md)**: toolchain, task map, reproducing CI gates
- **[Architecture](docs/ARCHITECTURE.md)**: module map, pipeline, design decisions
- **[Engineering policies](docs/POLICIES.md)**: MSRV, SemVer, security, concurrency
- **[Decision records](docs/adr/README.md)**: irreversible architectural decisions
- **[FAQ](docs/FAQ.md)**: formats, async, `no_std`, MSRV, dates, required fields, escaping, typed structs, MCP, reporting vulnerabilities
- **[Roadmap](https://github.com/sebastienrousseau/metadata-gen/milestones)**: planned releases as GitHub milestones

| Document | Covers |
| :--- | :--- |
| [`CHANGELOG.md`](CHANGELOG.md) | Per-release notes, Keep a Changelog format |
| [`SECURITY.md`](SECURITY.md) | Disclosure policy, supported versions, security design |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Branch and commit conventions, PR expectations |
| [`GOVERNANCE.md`](GOVERNANCE.md) | Project stewardship, how changes land |
| [`SUPPORT.md`](SUPPORT.md) | Support channels and expectations |
| [`AGENTS.md`](AGENTS.md) | Invariants for AI-assisted contributions |
| [`docs/SUPPLY-CHAIN.md`](docs/SUPPLY-CHAIN.md) | Dependencies, cargo-vet policy, SBOM and provenance verification |
| [`docs/COMPARISON.md`](docs/COMPARISON.md) | Feature matrix against other front-matter crates, with sources |
| [`docs/MCP.md`](docs/MCP.md) | The MCP server example: tools, errors, client registration |
| [`docs/deploy/`](docs/deploy/README.md) | Running the WASI component on Cloudflare Workers, Spin, wasmCloud |
| [`docs/whitepapers/`](docs/whitepapers/README.md) | Engineering write-ups on the flat map, the WASI component and property testing |

---

## Stability guarantees

### Versioning

[SemVer 2.0.0](https://semver.org) is followed, with the pre-1.0 posture that releases increment strictly by `+0.0.1` along the `0.0.x` line. Every breaking change is documented in [`CHANGELOG.md`](CHANGELOG.md).

### Output stability

What the crate produces is part of the API: a change to how a document flattens, which key a value lands under, or what a meta-tag group renders is treated as a breaking change even when no Rust signature moves.

### Deprecations

Deprecations live for at least two releases with a `#[deprecated]` attribute naming the replacement before removal.

### Minimum-toolchain discipline

The floor is **Rust 1.89.0**, declared as `rust-version` in `Cargo.toml`. Raising it is a breaking change and occurs only on releases with rationale documented in the changelog. See [`docs/POLICIES.md`](docs/POLICIES.md) for the complete policy.

### Version-bearing files

Checked against the manifest by `scripts/verify-release-versions.sh` before a tag exists, ensuring install snippets and metadata stay synchronised.

---

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you shall be dual-licensed as above, without any additional terms or conditions.

<p align="right"><a href="#metadata-gen">Back to top</a></p>
