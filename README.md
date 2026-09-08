<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/metadata-gen/v1/logos/metadata-gen.svg" alt="metadata-gen logo" width="128" />
</p>

<h1 align="center">metadata-gen</h1>

<p align="center">
  Front matter in, metadata and SEO meta tags out. YAML, TOML and JSON,
  with zero <code>unsafe</code> code.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/metadata-gen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/metadata-gen/ci.yml?branch=main&style=for-the-badge&label=build" alt="Build status" /></a>
  <a href="https://crates.io/crates/metadata-gen"><img src="https://img.shields.io/crates/v/metadata-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="crates.io version" /></a>
  <a href="https://docs.rs/metadata-gen"><img src="https://img.shields.io/badge/docs.rs-metadata--gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="API docs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/metadata-gen"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/metadata-gen?style=for-the-badge&token=hidden&logo=codecov" alt="Coverage" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/metadata-gen"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/metadata-gen?style=for-the-badge&label=scorecard" alt="OpenSSF Scorecard" /></a>
  <a href="https://www.bestpractices.dev/projects/14536"><img src="https://img.shields.io/cii/level/14536?style=for-the-badge&label=OpenSSF%20Best%20Practices&logo=openssf" alt="OpenSSF Best Practices" /></a>
  <a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License" /></a>
  <a href="#minimum-toolchain-policy"><img src="https://img.shields.io/badge/MSRV-1.88.0-orange.svg?style=for-the-badge" alt="MSRV 1.88.0" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) — Cargo, source
- [Requirements](#requirements) — toolchain floor, platforms
- [Quick Start](#quick-start) — front matter to meta tags in ten lines

**Library reference**

- [What it does](#what-it-does) — the three formats and the flat map
- [Two APIs, one parser](#two-apis-one-parser) — flat map vs. typed
- [Library Usage](#library-usage) — extraction, the body, processing, meta tags
- [Configuration](#configuration) — `ProcessOptions`
- [Ecosystem comparison](#ecosystem-comparison) — where this sits
- [Benchmarks](#benchmarks) — measured numbers with the host stated
- [Examples](#examples) — runnable example index

**Operational**

- [When not to use metadata-gen](#when-not-to-use-metadata-gen) — limitations
- [Development](#development) — make targets, fuzzing, Miri, CI
- [Security](#security) — hardening, fuzzing, supply chain
- [Documentation](#documentation) — all reference docs
- [Stability guarantees](#stability-guarantees) — SemVer axis, output stability
- [Minimum-toolchain policy](#minimum-toolchain-policy)
- [License](#license)

---

## Install

### As a Rust library (crates.io)

```toml
[dependencies]
metadata-gen = "0.0.7"
```

Or from the command line:

```bash
cargo add metadata-gen
```

There is no CLI. `metadata-gen` is a library crate and ships no
`[[bin]]`; the `command-line-utilities` category was removed in v0.0.7
because it advertised something that does not exist.

### Build from source

```bash
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen
make          # check + clippy + test
```

### Cargo features

None. Every capability is on by default, and the manifest declares no
optional features — the previous `advanced_parsing` flag gated no code
and was removed in v0.0.7 rather than left as a claim the crate did not
keep. Per-format feature gates are on the roadmap; when they land they
will be additive and documented here.

---

## Requirements

- **Rust 1.88.0 or newer.** `rust-version` in `Cargo.toml` is the floor
  and Cargo enforces it; CI builds on stable across Linux, macOS and
  Windows. See the [minimum-toolchain policy](#minimum-toolchain-policy)
  for when and why the floor may move.
- **A `std` platform.** The crate uses `std` unconditionally today. A
  `no_std + alloc` core is roadmap work, not a current capability.
- **No async runtime is required.** Every synchronous entry point works
  without one. `async_extract_metadata_from_file` is a convenience for
  callers who already run Tokio; the dependency is trimmed to `fs`,
  `io-util`, `rt` and `macros`.

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

---

## What it does

`metadata-gen` reads the structured block at the top of a content file
and turns it into two things: a metadata map your templates can index,
and the `<meta>` element groups a page needs for search engines and
social cards.

Three front-matter shapes are recognised, in this order:

| Format | Delimiters | Opening line |
|---|---|---|
| YAML | `---` … `---` | `title: Hello` |
| TOML | `+++` … `+++` | `title = "Hello"` |
| JSON | a `{ … }` object at the top of the file | `{"title": "Hello"}` |

The first shape whose opening delimiter matches wins. A shape that
matches but fails to parse reports that parser's error rather than
falling through to the next, so a broken YAML block never silently
becomes "no front matter".

The crate also runs in reverse: `extract_meta_tags` pulls `<meta>`
elements back out of an HTML document in a single streaming pass.

---

## Two APIs, one parser

The same detection front-ends two shapes of result, and which one you
want depends on whether your consumer is a template or a struct.

| | `extract_metadata` | `extract_typed::<T>` |
|---|---|---|
| Returns | `Metadata`, a flat `HashMap<String, String>` | your `T: Deserialize` |
| Nested tables | dotted keys: `author.name` | nested structs |
| Sequences | `"[a, b]"` | `Vec<T>` |
| Numbers, booleans | their `Display` form | typed |
| Best for | templates, meta-tag generation | typed config, validation |

Both accept the same three formats. Pick the flat map when the
destination is a template that will stringify everything anyway; pick
the typed form when a wrong type should be an error rather than a
surprise later. [ADR-0002](docs/adr/0002-flat-string-metadata.md)
records why the flat map exists at all.

---

## Library Usage

### Extract front matter

```rust
use metadata_gen::metadata::extract_metadata;

let content = "---\n\
title: My Post\n\
date: 2026-06-28\n\
author:\n  name: Ada\n\
tags:\n  - rust\n  - parsing\n---\n";

let meta = extract_metadata(content).unwrap();
assert_eq!(meta.get("title"),       Some(&"My Post".to_string()));
assert_eq!(meta.get("author.name"), Some(&"Ada".to_string()));
assert_eq!(meta.get("tags"),        Some(&"[rust, parsing]".to_string()));
```

TOML and JSON work identically:

```rust
use metadata_gen::metadata::extract_metadata;

let toml = "+++\ntitle = \"My Post\"\n[author]\nname = \"Ada\"\n+++\n";
assert_eq!(
    extract_metadata(toml).unwrap().get("author.name"),
    Some(&"Ada".to_string())
);

let json = "{\"title\": \"My Post\", \"author\": {\"name\": \"Ada\"}}\n# Body";
assert_eq!(
    extract_metadata(json).unwrap().get("author.name"),
    Some(&"Ada".to_string())
);
```

### Typed extraction

```rust
use metadata_gen::extract_typed;

#[derive(serde::Deserialize)]
struct Front {
    title: String,
    tags: Vec<String>,
    draft: bool,
}

let doc = "---\ntitle: Typed\ntags: [rust, seo]\ndraft: false\n---\nBody";
let front: Front = extract_typed(doc).unwrap();

assert_eq!(front.tags, ["rust", "seo"]);
assert!(!front.draft);
```

### Keep the document body

```rust
use metadata_gen::extract_metadata_with_body;

let doc = "---\ntitle: T\n---\n# Heading\n\nText";
let (meta, body) = extract_metadata_with_body(doc).unwrap();

assert_eq!(meta.get("title").map(String::as_str), Some("T"));
assert_eq!(body, "# Heading\n\nText");
```

`detect_front_matter` exposes the same information without parsing: the
format, the raw block, and the byte offset where the body begins.

### Process and validate

`process_metadata` normalises dates to `YYYY-MM-DD`, checks that the
required fields are present, and derives a `slug` from the title when
one is absent.

```rust
use metadata_gen::{process_metadata, Metadata};
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("title".to_string(), "Hello World".to_string());
map.insert("date".to_string(), "01/02/2024".to_string());

let processed = process_metadata(&Metadata::new(map)).unwrap();
assert_eq!(processed.get("date").map(String::as_str), Some("2024-02-01"));
assert_eq!(processed.get("slug").map(String::as_str), Some("hello-world"));
```

### Generate meta tags

```rust
use metadata_gen::generate_metatags;
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("description".to_string(), "About the page".to_string());
map.insert("og:title".to_string(), "Page Title".to_string());
map.insert("twitter:card".to_string(), "summary_large_image".to_string());

let groups = generate_metatags(&map);
assert!(groups.primary.contains("description"));
assert!(groups.og.contains("og:title"));
assert!(groups.twitter.contains("twitter:card"));
```

Five groups are produced: `primary`, `og`, `twitter`, `apple` and `ms`.
Attribute values pass through `escape_html`, so a title containing `<`
or `"` cannot break out of the element.

### Read tags back from HTML

```rust
use metadata_gen::metatags::extract_meta_tags;

let html = r#"<html><head>
  <meta name="description" content="A &amp; B">
  <meta property="og:title" content="T" />
</head></html>"#;

let tags = extract_meta_tags(html).unwrap();
assert_eq!(tags.len(), 2);
assert_eq!(tags[0].content, "A & B");
```

Extraction is deliberately tolerant: malformed markup ends the scan and
returns what was found so far, because the usual input is a whole HTML
page that a strict XML reader will not accept end to end
([ADR-0003](docs/adr/0003-streaming-meta-extraction.md)).

### Read from a file, asynchronously

```rust,no_run
use metadata_gen::utils::async_extract_metadata_from_file;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let (metadata, keywords, tags) =
    async_extract_metadata_from_file("post.md").await?;
println!("title = {:?}", metadata.get("title"));
# Ok(())
# }
```

---

## Configuration

`process_metadata` uses a fixed policy: `title` and `date` are required,
and `slug` is derived. `process_metadata_with` takes a `ProcessOptions`
when that policy does not fit.

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
|---|---|---|
| `required_fields` | `["title", "date"]` | a missing field is `MissingFieldError`, naming it |
| `derive_slug` | `true` | derive `slug` from `title` when absent |

`ProcessOptions` is `#[non_exhaustive]`, so options can be added without
a breaking release.

---

## Ecosystem comparison

| Crate | YAML | TOML | JSON | Typed | Body returned | Meta tags |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **`metadata-gen`** | yes | yes | yes | yes | yes | yes |
| `gray_matter` | yes | yes | yes | yes | yes | no |
| `yaml-front-matter` | yes | no | no | yes | yes | no |
| `matter` | yes | no | no | no | yes | no |

`gray_matter` is the closest incumbent and the honest recommendation if
all you need is front matter: it is older, more widely used, and does
the same job. What this crate adds is the meta-tag half — generation and
extraction against the same metadata map — and the supply-chain posture
described under [Security](#security).

---

## Benchmarks

Measured with Criterion on an Apple A18 Pro, rustc 1.98.0, single
thread. Middle estimate of the confidence interval; run
`cargo bench` to reproduce on your own hardware, because these numbers
are worth nothing without the host they came from.

| Workload | Input | Time | Throughput |
|---|---:|---:|---:|
| `extract_metadata` (YAML) | 1 KB | 631 µs | 1.5 MiB/s |
| `extract_metadata` (YAML) | 10 KB | 1.81 ms | 5.4 MiB/s |
| `extract_metadata` (YAML) | 1 MB | 181 ms | 5.5 MiB/s |
| `extract_meta_tags` | 1 KB | 50 µs | 18.7 MiB/s |
| `extract_meta_tags` | 1 MB | 34.9 ms | 28.7 MiB/s |
| `escape_html` | 10 KB | 31.9 µs | 295 MiB/s |
| `extract_and_prepare_metadata` | ~250 B | 23 µs | — |

The shape to note is that extraction is dominated by the underlying
format parser, not by this crate's flattening: throughput is flat from
10 KB to 1 MB. Typical front matter is a few hundred bytes, where the
whole pipeline costs tens of microseconds.

---

## Examples

Run any of these with `cargo run --example <name>`:

| Example | Shows |
|---|---|
| `lib_example` | the high-level `extract_and_prepare_metadata` flow |
| `metadata_example` | per-format extraction, nested tables, typed extraction, the body |
| `metatags_example` | generating `<meta>` groups and reading them back |
| `utils_example` | HTML escape/unescape and the async file helper |
| `error_example` | every `MetadataError` variant and how to recover |

`make examples` runs all of them; CI does the same on every push, so an
example that stops working fails the build.

---

## When not to use metadata-gen

Cases where something else fits better, listed because the honest answer
is "not yet" rather than a disagreement about priorities.

- **You need `no_std` or a WASM component today.** The crate uses `std`
  unconditionally and pulls `tokio` for the async file helper. A
  `no_std + alloc` core is roadmap work.
- **You need element-level access to arrays of objects from the flat
  map.** `[a, b]` is a rendered string there by design. Use
  `extract_typed::<T>` instead, which keeps the structure.
- **You need to round-trip front matter byte-for-byte.** The crate
  parses; it does not preserve comments, key order or quoting style, and
  there is no serialiser back to a fenced block.
- **You need every `<meta>` element from arbitrary broken HTML.**
  Extraction stops at the first unrecoverable reader error and returns
  what it has. A real HTML parser (`html5ever`, `scraper`) is the right
  tool if you need error recovery over whole pages.

If you hit a case that should be on this list, please open an issue —
that is how it gets fixed or moved into the supported set.

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
make fuzz         # build every target, replay corpus and regressions
make examples     # run every example
make bench-smoke  # compile and run each bench once
make versions     # every version-bearing file agrees
make deny / vet / audit   # supply chain
```

[`DEVELOPMENT.md`](DEVELOPMENT.md) maps each CI job to its local
equivalent and explains the gotchas.

### Fuzzing

Three `cargo-fuzz` targets live under `fuzz/fuzz_targets/`:

```bash
cargo +nightly fuzz run fuzz_extract_metadata   # all three front-matter shapes
cargo +nightly fuzz run fuzz_extract_meta_tags  # the streaming <meta> reader
cargo +nightly fuzz run fuzz_html_escape        # escape/unescape identity
```

`fuzz/corpus/<target>` holds the committed seeds and
`fuzz/regressions/<target>` every fixed-bug input; both replay on each
push, so a fixed crash cannot silently return. The first target that ran
found one: `unescape_html` decoded its own output, so `&amp;lt;` came
back as `<`. That input is now the first regression.

### Miri (UB / aliasing verification)

The crate is `#![forbid(unsafe_code)]`, so Miri does not police its own
code. The job exists to check the interaction with dependencies that do
use `unsafe` internally.

```bash
make miri     # cargo +nightly miri test --lib
```

The eight tests that touch the filesystem are skipped under Miri, whose
isolation forbids `open` and `mkdir`.

### CI

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | push, PR | fmt, clippy, tests across three OSes, coverage, cargo-deny, cargo-audit |
| `docs.yml` | push to main | build and deploy rustdoc to GitHub Pages |

See [CONTRIBUTING.md](CONTRIBUTING.md) for signed commits and PR
guidelines.

---

## Security

**Reporting:** never open a public issue for a vulnerability. See
[`SECURITY.md`](SECURITY.md) for the private channel and disclosure
policy.

### Architectural posture

- `#![forbid(unsafe_code)]` — the compiler proves the absence of unsafe
  blocks ([ADR-0001](docs/adr/0001-zero-unsafe-policy.md)).
- No C dependencies, no FFI, no network I/O, no environment reads. The
  only file access is the explicit async helper, which reads the path
  its caller names.
- Meta-tag values are escaped on generation, so metadata cannot inject
  markup into a page.

### Resource limits, stated plainly

Front matter is parsed by `noyalib`, `toml` and `serde_json`, each with
its own bounds on nesting and size. This crate adds no recursion of its
own beyond flattening the parsed tree. It also adds **no configurable
limits of its own**: callers handling untrusted input of unbounded size
should cap it before calling `extract_metadata`. That is documented
rather than silently assumed.

### Fuzzing

Three targets, a committed seed corpus, and a regression corpus replayed
per push (see [Development](#development)). Not yet on OSS-Fuzz.

### Supply chain

- `cargo-deny` (licences, advisories, sources) and `cargo-audit` in CI.
- `cargo-vet` provenance in `supply-chain/`, with an exemption baseline
  the CI ratchet cannot exceed.
- First-party crates `noyalib` and `dtt` pinned exactly
  ([ADR-0004](docs/adr/0004-first-party-exact-pins.md)); a bump is a
  deliberate release of this crate.
- `Cargo.lock` committed; CI builds `--locked`. Actions pinned by SHA.
- REUSE 3.3 compliant, linted in CI.
- Ten direct runtime dependencies, 43 crates in the resolved runtime
  tree.

### Commit integrity

Commits on `main` are signed and releases are signed tags; the key is in
[`KEYS.asc`](KEYS.asc).

---

## Documentation

The four entry points, identical across every repo in the family:

- **[API reference](https://docs.rs/metadata-gen)** — rustdoc on docs.rs
- **[Developer docs](DEVELOPMENT.md)** — toolchain, task map, reproducing
  every CI gate locally
- **[Architecture](docs/ARCHITECTURE.md)** — module map, pipeline,
  design decisions
- **[Decision records](docs/adr/README.md)** — the choices that would be
  expensive to reverse

| Document | Covers |
|---|---|
| [`CHANGELOG.md`](CHANGELOG.md) | per-release notes, Keep a Changelog format |
| [`SECURITY.md`](SECURITY.md) | disclosure policy, supported versions, security design |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | branch and commit conventions, PR expectations, code standards |
| [`GOVERNANCE.md`](GOVERNANCE.md) | who decides what, how changes land |
| [`SUPPORT.md`](SUPPORT.md) | where to ask, what to expect |
| [`AGENTS.md`](AGENTS.md) | invariants for AI-assisted contributions |

---

## Stability guarantees

- **Versioning.** [SemVer](https://semver.org), with the pre-1.0 posture
  that the patch number is the breaking axis during `0.0.x`. Releases
  increment by `+0.0.1`. Every breaking change is called out in
  [`CHANGELOG.md`](CHANGELOG.md).
- **Output stability.** What the crate *produces* is part of the API: a
  change to how a document flattens, which key a value lands under, or
  what a meta-tag group renders is treated as breaking even when no Rust
  signature moves.
- **Deprecations** live for at least two releases with a `#[deprecated]`
  note naming the replacement before removal.
- **Version-bearing files** are checked against the manifest by
  `scripts/verify-release-versions.sh` before a tag exists, so an install
  snippet cannot go stale.

---

## Minimum-toolchain policy

The floor is **Rust 1.88.0**, declared as `rust-version` in
`Cargo.toml` so Cargo refuses older toolchains with a clear message.

- **When it may rise:** only on a release, never silently, and always
  with the reason in the changelog entry.
- **Why it is where it is:** the floor is pulled by the transitive
  `time` crate through `dtt`, which requires edition 2024. Lowering it
  would mean pinning an older `time` that carries a stack-exhaustion
  advisory.
- **What is verified:** CI builds and tests on stable. The floor is the
  version Cargo enforces from the manifest.

No claim is made about distro-LTS toolchains. Making one would require a
table mapping current distro versions to this floor, and an
aspirational claim there is worse than none.

---

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT),
at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you shall be dual-licensed as
above, without any additional terms or conditions.

<p align="right"><a href="#metadata-gen">Back to top</a></p>
