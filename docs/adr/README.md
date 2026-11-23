# Architecture Decision Records

This directory holds the architectural decisions that shape
metadata-gen — the choices that would be expensive to reverse and that
contributors should understand before proposing structural changes.

## Format

Every ADR uses [Michael Nygard's format](https://github.com/architecture-decision-record/architecture-decision-record/tree/main/locales/en-gb/templates/decision-record-template-by-michael-nygard)
— see [`TEMPLATE.md`](./TEMPLATE.md). Sections are:

- **Status:** proposed / accepted / superseded / deprecated
- **Context:** what forces are at play
- **Decision:** what we're doing
- **Consequences:** what becomes easier and harder

ADRs are **immutable** once accepted — if a decision changes, the old
ADR moves to "superseded" and a new ADR is added with a reference back.
Nothing is silently rewritten.

## Index

| # | Title | Status | Date |
|---|---|---|---|
| [0001](./0001-zero-unsafe-policy.md) | `#![forbid(unsafe_code)]`, pure Rust, no FFI | accepted | 2024-10-03 |
| [0002](./0002-flat-string-metadata.md) | Front matter flattens to `HashMap<String, String>` with dotted keys | accepted | 2024-10-03 |
| [0003](./0003-streaming-meta-extraction.md) | `<meta>` extraction is a streaming `quick-xml` pass, not a DOM | superseded by 0006 | 2026-06-28 |
| [0004](./0004-first-party-exact-pins.md) | First-party crates are pinned exactly (`=0.0.X`) | accepted | 2026-06-28 |
| [0005](./0005-feature-flags-and-no-std.md) | Formats and integrations are Cargo features; the core is `no_std + alloc` | accepted | 2026-10-09 |
| [0006](./0006-meta-extraction-on-oxml.md) | `<meta>` extraction scans with `memchr` and parses tags with `oxml` | accepted | 2026-11-01 |
| [0007](./0007-front-matter-detection-order.md) | Front matter is detected by its opening delimiter: YAML, then TOML, then JSON | accepted | 2026-11-23 |
| [0008](./0008-noyalib-for-yaml.md) | YAML front matter is parsed by `noyalib` | accepted | 2026-04-18 |
| [0009](./0009-wasi-component-shape.md) | The WASI build is a separate component package with a three-function WIT interface | accepted | 2026-11-23 |

Numbers are sequential and four digits wide. A superseded ADR keeps
its text; only its status line changes, in the same change that adds
its replacement.

## When to add an ADR

Add one when you're about to make a decision that:

- Is hard to reverse (changes the data model, public API surface,
  dependency floor, or core invariants like the unsafe policy)
- Will surprise a future contributor reading the code
- Has plausible alternatives that someone might propose later

Don't add ADRs for routine implementation choices — those go in commit
messages and code comments. The bar is "would I want a new contributor
to read this before proposing the opposite?"
