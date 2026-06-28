<p align="center">
  <img src="https://cloudcdn.pro/metadata-gen/v1/logos/metadata-gen.svg" alt="Metadata Gen logo" width="128" />
</p>

<h1 align="center">metadata-gen</h1>

<p align="center">
  <strong>A typed, audited frontmatter parser for Rust — YAML, TOML, and JSON extraction with HTML meta-tag emission for SEO, Open Graph, Twitter Cards, and Apple Web Apps.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/metadata-gen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/metadata-gen/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/metadata-gen"><img src="https://img.shields.io/crates/v/metadata-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/metadata-gen"><img src="https://img.shields.io/badge/docs.rs-metadata--gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/metadata-gen"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/metadata-gen?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/metadata-gen"><img src="https://img.shields.io/badge/lib.rs-metadata--gen-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Table of contents

- [What it does](#what-it-does)
- [When to use it](#when-to-use-it)
- [Install](#install)
- [Quick start](#quick-start)
- [Examples](#examples)
  - [YAML frontmatter](#yaml-frontmatter)
  - [TOML frontmatter](#toml-frontmatter)
  - [JSON frontmatter](#json-frontmatter)
  - [HTML meta tag generation](#html-meta-tag-generation)
  - [Asynchronous file extraction](#asynchronous-file-extraction)
- [Comparisons](#comparisons)
- [Performance](#performance)
- [Supply chain](#supply-chain)
- [MSRV policy](#msrv-policy)
- [Roadmap](#roadmap)
- [FAQ](#faq)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

---

## What it does

`metadata-gen` parses a content file's *frontmatter* — the structured block at
the top of a Markdown/HTML document — and turns it into a usable Rust value
plus a set of SEO meta tag groups (`primary`, `og`, `twitter`, `apple`, `ms`).

It accepts three frontmatter formats out of the box:

| Format | Delimiters       | Example header               |
|--------|------------------|------------------------------|
| YAML   | `---` / `---`    | `title: Hello`               |
| TOML   | `+++` / `+++`    | `title = "Hello"`            |
| JSON   | `{` / `}`        | `{"title": "Hello"}`         |

Format detection is automatic. The detection order is YAML → TOML → JSON; the
first format whose opening delimiter matches is used. Nested values are
flattened with dot-separated keys (e.g. `author.name`), and sequences are
serialized as `[a, b, c]` strings in the current map-based API.

## When to use it

Choose `metadata-gen` when you want:

- A **single dependency** that handles YAML, TOML, *and* JSON frontmatter
  without forcing you to pick a serializer first.
- A built-in **HTML meta tag generator** (Open Graph, Twitter Cards, Apple
  Mobile, Microsoft Tiles, primary SEO) wired to the same metadata map.
- A library with an **explicit supply-chain posture** — `cargo-deny`,
  `cargo-audit`, SBOM emission, and `#![forbid(unsafe_code)]` enforced.
- A library actively heading toward **typed extraction**, **zero-copy values**,
  and a **WASI 0.2 component** (see the [Roadmap](#roadmap)).

Choose something else when you only need raw YAML parsing (use `serde_yaml_ng`
or `noyalib` directly) or when you need typed extraction *today* (track
issue [#42](https://github.com/sebastienrousseau/metadata-gen/issues/42) for
v0.0.6).

## Install

```bash
cargo add metadata-gen
```

Or add to `Cargo.toml`:

```toml
[dependencies]
metadata-gen = "0.0.5"
```

Minimum Supported Rust Version: **1.88.0** — see the
[MSRV policy](#msrv-policy). Tested on Linux, macOS, and Windows on x86_64
and ARM64.

## Quick start

```rust
use metadata_gen::extract_and_prepare_metadata;

let content = "---\n\
title: Hello, world!\n\
description: A short greeting\n\
keywords: rust, frontmatter, seo\n\
---\n\
# Body starts here";

let (metadata, keywords, tags) =
    extract_and_prepare_metadata(content).expect("valid frontmatter");

assert_eq!(metadata.get("title"), Some(&"Hello, world!".to_string()));
assert_eq!(keywords, vec!["rust", "frontmatter", "seo"]);
assert!(tags.primary.contains("description"));
```

## Examples

Run any example with `cargo run --example <name>`:

| Example                | Demonstrates                                                |
|------------------------|-------------------------------------------------------------|
| `lib_example`          | High-level `extract_and_prepare_metadata` + meta tag flow   |
| `metadata_example`     | Per-format extraction (YAML, TOML, JSON) + nested mappings  |
| `metatags_example`     | Generating + extracting HTML `<meta>` tags                  |
| `utils_example`        | HTML escape/unescape, async file extraction                 |
| `error_example`        | Every `MetadataError` variant + recovery patterns           |

### YAML frontmatter

```rust
use metadata_gen::metadata::extract_metadata;

let content = "---\n\
title: My Post\n\
date: 2026-06-28\n\
author:\n  name: Ada\n  handle: ada@example.com\n\
tags:\n  - rust\n  - parsing\n---\n";

let meta = extract_metadata(content).unwrap();
assert_eq!(meta.get("title"),       Some(&"My Post".to_string()));
assert_eq!(meta.get("author.name"), Some(&"Ada".to_string()));
assert_eq!(meta.get("tags"),        Some(&"[rust, parsing]".to_string()));
```

### TOML frontmatter

```rust
use metadata_gen::metadata::extract_metadata;

let content = "+++\n\
title = \"My Post\"\n\
date  = \"2026-06-28\"\n\
\n\
[author]\n\
name = \"Ada\"\n\
+++\n";

let meta = extract_metadata(content).unwrap();
assert_eq!(meta.get("author.name"), Some(&"Ada".to_string()));
```

### JSON frontmatter

```rust
use metadata_gen::metadata::extract_metadata;

let content = "{\
\"title\":\"My Post\",\
\"description\":\"Inline JSON header\"\
}\n# Body";

let meta = extract_metadata(content).unwrap();
assert_eq!(meta.get("title"), Some(&"My Post".to_string()));
```

> Nested JSON objects in the current API: see
> [issue #26](https://github.com/sebastienrousseau/metadata-gen/issues/26)
> — the v0.0.5 fix uses `serde_json::Deserializer` to correctly handle
> balanced braces and arrays of objects.

### HTML meta tag generation

```rust
use std::collections::HashMap;
use metadata_gen::metatags::generate_metatags;

let mut map = HashMap::new();
map.insert("description".to_string(), "About the page".to_string());
map.insert("og:title".to_string(),    "Page Title".to_string());
map.insert("twitter:card".to_string(),"summary_large_image".to_string());

let groups = generate_metatags(&map);
assert!(groups.primary.contains("description"));
assert!(groups.og.contains("og:title"));
assert!(groups.twitter.contains("twitter:card"));
```

### Asynchronous file extraction

```rust,no_run
use metadata_gen::utils::async_extract_metadata_from_file;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (metadata, keywords, tags) =
        async_extract_metadata_from_file("post.md").await?;
    println!("title    = {:?}", metadata.get("title"));
    println!("keywords = {:?}", keywords);
    println!("og tags  =\n{}", tags.og);
    Ok(())
}
```

## Comparisons

| Crate                 | YAML | TOML | JSON | Typed extraction | Meta-tag emit | no_std (planned) |
|-----------------------|:----:|:----:|:----:|:----------------:|:-------------:|:----------------:|
| **`metadata-gen`**    |  ✅  |  ✅  |  ✅  |  v0.0.6 roadmap  |       ✅      |  v0.0.9 roadmap  |
| `gray_matter`         |  ✅  |  ✅  |  ✅  |        ✅        |       —       |        —         |
| `yaml-front-matter`   |  ✅  |  —   |  —   |        ✅        |       —       |        —         |
| `matter`              |  ✅  |  —   |  —   |        —         |       —       |        ✅        |

`gray_matter` is the closest incumbent. `metadata-gen` differentiates on the
bundled meta-tag emitter, the supply-chain posture, and the WASI/no_std
roadmap. See the [audit deck](docs/AUDIT-2026.md) for the strategic context.

## Performance

Per-call latency on a 2024 reference laptop (M-class CPU, single thread):

| Target                            | Input size | Latency  |
|-----------------------------------|-----------:|---------:|
| `extract_metadata` (YAML)         | ~200 B     | ~10 µs   |
| `process_metadata`                | ~200 B     | ~1 µs    |
| `generate_metatags`               | ~200 B     | ~1 µs    |
| `escape_html`                     | ~80 B      | ~0.3 µs  |

Run the suite yourself:

```bash
cargo bench --bench metadata_benchmark
```

A 10–100× throughput improvement is planned for v0.0.7 via `Cow<'a, str>`
values, `LazyLock<Regex>` statics, single-pass HTML escape, and
`memchr::memmem` delimiter scanning. See
[v0.0.7](https://github.com/sebastienrousseau/metadata-gen/milestones)
for details.

## Supply chain

`metadata-gen` enforces an explicit supply-chain posture:

- **`cargo-deny`** runs on every PR (`advisories`, `licenses`, `bans`,
  `sources`); CI fails on any violation.
- **`cargo-audit`** runs on every PR and on a daily schedule against the
  RUSTSEC database.
- **`#![forbid(unsafe_code)]`** is enforced crate-wide.
- **First-party 0.0.x dependencies** (`noyalib`, `dtt`) are pinned strictly
  in `Cargo.toml` so an upstream patch cannot break downstream consumers
  without a deliberate `metadata-gen` release.
- **SBOM emission** (CycloneDX) and cosign signing land in v0.0.5 — see the
  [Roadmap](#roadmap).

Documented advisory exemptions live in
[`audit.toml`](audit.toml); each entry carries a rationale referencing the
upstream tracking issue.

## MSRV policy

Minimum Supported Rust Version: **1.88.0**.

We treat the MSRV as part of the public API: increases are batched into
minor (`0.x.0`) releases and called out in `CHANGELOG.md`. The current
1.88.0 floor is pinned transitively by `dtt 0.0.10 → time 0.3.47 →
time-core =0.1.8` (edition2024). Downgrading the floor would re-introduce a
medium-severity stack-exhaustion advisory in `time`, so we hold the line.

If you need an older toolchain, please open an issue describing your
constraint — we are happy to discuss MSRV-segmented branches.

## Roadmap

The post-v0.0.4 roadmap is split into six themed releases. Every milestone
is tracked on
[GitHub Milestones](https://github.com/sebastienrousseau/metadata-gen/milestones)
with full user stories and acceptance criteria per issue.

| Version  | Theme                            | Highlights                                                                |
|----------|----------------------------------|---------------------------------------------------------------------------|
| v0.0.5   | **Foundation Hardening**         | Drop `tokio = "full"`, `LazyLock<Regex>` statics, fix JSON nested-brace bug, `cargo-deny`/`cargo-audit` gating, SBOM emission, rustdoc Actions deploy, README/FAQ overhaul. |
| v0.0.6   | **Typed API & Ergonomics**       | `extract_typed::<T: Deserialize>`, `(Metadata, body: &str)` return, builder pattern, per-format Cargo features, schema validation. |
| v0.0.7   | **Zero-copy & Performance**      | `Cow<'a, str>` value API, single-pass HTML escape, `memchr::memmem` scan, throughput benches at 1 KB → 10 MB, Codspeed CI gate. |
| v0.0.8   | **Correctness & Verification**   | `proptest` harness, `cargo-fuzz` target, Miri in nightly CI, `cargo-mutants` ≥ 85 % kill, Kani proof, ≥ 98 % coverage gate. |
| v0.0.9   | **Portability**                  | `no_std + alloc` core, async-runtime-agnostic IO, optional Tokio/smol/Embassy adapters, embedded CI matrix. |
| v0.0.10  | **WASI / Blue Ocean / 1.0 RC**   | `wasm32-wasip2` Component with WIT interface, Cloudflare Workers / Spin / wasmCloud guides, PQC-signed metadata, MCP server example, ADR series. |

## FAQ

### 1. Why three frontmatter formats instead of just YAML?

Real-world content pipelines aren't homogeneous. Jekyll/Hugo use YAML and
TOML; static-site generators built on `serde_json` prefer JSON; documentation
toolchains routinely encounter all three. `metadata-gen` accepts all three so
your downstream code only depends on one crate.

### 2. How does this compare to `gray_matter`?

`gray_matter` is the dominant frontmatter parser in the Rust ecosystem and
has been since 2020. It does typed extraction (via its `Pod`) today, which
`metadata-gen` will reach in v0.0.6. `metadata-gen` differentiates on the
bundled HTML meta-tag emitter, the documented supply-chain posture, the
WASI/no_std roadmap, and the strict pinning of first-party transitive
dependencies. If you need typed extraction *today*, use `gray_matter`. If you
want the v0.0.10 WASI Component, follow this crate.

### 3. Do I need an async runtime to use this library?

No. The synchronous entry points (`extract_metadata`, `process_metadata`,
`extract_and_prepare_metadata`, `generate_metatags`, `escape_html`) do not
require Tokio. The async helper `async_extract_metadata_from_file` is a
convenience for callers who already use Tokio; we trim Tokio to its `fs` +
`io-util` features so it doesn't bloat your build. A runtime-agnostic
`AsyncRead` boundary lands in v0.0.9.

### 4. Can I use `metadata-gen` in `no_std` / WASM?

Not in v0.0.5 — `regex`, `scraper`, and `tokio` are all unconditionally
pulled. `no_std + alloc` support is the v0.0.9 milestone, and a full
`wasm32-wasip2` Component lands in v0.0.10. Track milestones
[v0.0.9](https://github.com/sebastienrousseau/metadata-gen/milestones)
and [v0.0.10](https://github.com/sebastienrousseau/metadata-gen/milestones)
for status.

### 5. What is the MSRV policy?

MSRV is part of the public API. Increases happen on `0.x.0` boundaries and
are documented in `CHANGELOG.md`. The current floor (1.88.0) is pinned
transitively by a security advisory in `time`; lowering it would re-introduce
the vulnerability.

### 6. How are dates parsed?

`process_metadata` tries, in order:
1. ISO-8601 / RFC 3339 (`2026-06-28`, `2026-06-28T15:30:00Z`).
2. `YYYY-MM-DD` explicit format.
3. `MM/DD/YYYY` US format.
4. `DD/MM/YYYY` European format (recognised by length + slash pattern).

The output is always normalized to `YYYY-MM-DD`. Out-of-range or ambiguous
inputs return `MetadataError::DateParseError`.

### 7. How do I add custom required fields?

In v0.0.5 the required fields are hard-coded to `title` and `date`. A
configurable `MetadataProcessor` builder lands in v0.0.6
(issue [#47](https://github.com/sebastienrousseau/metadata-gen/issues/47)).
Until then, validate your own required fields with
`metadata.contains_key("…")` after `extract_metadata`.

### 8. How is HTML escaping handled?

`escape_html` maps `& < > " '` to their entity equivalents. `unescape_html`
maps them back (plus `&#x2F;` / `&#x2f;` to `/`). The pair is round-trip
safe on every ASCII input — a property-test corpus + Kani proof of that
invariant land in v0.0.8. The implementation is currently a five-pass
`str::replace` chain; a single-pass rewrite (with optional SIMD via
`v_htmlescape`) ships in v0.0.7
([#52](https://github.com/sebastienrousseau/metadata-gen/issues/52)).

### 9. How do I extract typed structs (instead of a `HashMap<String, String>`)?

Not in v0.0.5. The v0.0.6 milestone adds
`metadata_gen::extract_typed::<T: serde::Deserialize>(content)` that
preserves typed information (dates as `time::Date`, integers as integers,
nested objects as nested structs). Track
[issue #45](https://github.com/sebastienrousseau/metadata-gen/issues/45).

### 10. Where do I report a vulnerability?

Please do **not** open a public GitHub issue. Email the maintainer per the
[SECURITY.md](.github/SECURITY.md) policy. We acknowledge within 48 hours and
publish a fix on the most recent stable line. New vulnerability classes
trigger a `cargo-fuzz` target so the same shape can't reappear.

### 11. Will my dependency tree grow when I add this crate?

Today, yes — `metadata-gen` v0.0.5 still pulls `scraper`, `tokio`, and `regex`.
v0.0.5 issue [#22](https://github.com/sebastienrousseau/metadata-gen/issues/22)
replaces `scraper` with `quick-xml`, dropping ~30 transitive crates and
silencing two RUSTSEC advisories. Per-format Cargo feature gates land in
v0.0.6 ([#41](https://github.com/sebastienrousseau/metadata-gen/issues/41)).

### 12. Is there a CLI?

No. `metadata-gen` is a library crate. We removed the
`command-line-utilities` category from `Cargo.toml` in v0.0.5 because no
`[[bin]]` ships. If you want a CLI wrapper, please open a discussion — there
is a credible case for a `metadata-gen-cli` companion crate.

## Contributing

Pull requests welcome. See
[CONTRIBUTING.md](CONTRIBUTING.md) for setup, signed-commit policy, and
the issue-template format used across the roadmap.

Quick local loop:

```bash
cargo fmt --all
cargo clippy --all-features --all-targets -- -D warnings
cargo test  --all-features
cargo bench --bench metadata_benchmark
```

## Security

- Report vulnerabilities per [`.github/SECURITY.md`](.github/SECURITY.md).
- The crate enforces `#![forbid(unsafe_code)]`.
- Supply-chain controls (`cargo-deny`, `cargo-audit`, SBOM, `cargo-vet`
  audits) are documented in the [Supply chain](#supply-chain) section.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at
your option.

<p align="right"><a href="#metadata-gen">Back to top</a></p>
