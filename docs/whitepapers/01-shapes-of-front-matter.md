<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# One map is not enough: the shapes of front matter

*2026-11-23. Sources: `CHANGELOG.md` (0.0.7, 0.0.8, 0.0.10),
ADR-0002, ADR-0005, ADR-0011, `src/metadata/`, `README.md` benchmarks.*

## The problem

Front matter is the block of structured data at the top of a Markdown
file: a title, a date, an author, some tags. Static-site generators
read it in three syntaxes (YAML between `---` fences, TOML between
`+++` fences, a JSON object), and then do very different things with
it. A template engine wants strings it can drop into HTML. A build
step wants a `date` it can sort by and a `draft` flag it can branch
on. A sitemap generator over a hundred thousand pages wants one field
from each file and nothing else.

`metadata-gen` started with one answer for all of them:
`HashMap<String, String>`, with nested keys joined by dots. This paper
is about why that answer was right for its first users, why it was not
enough, and why the fix was to add shapes rather than replace the
first one.

## The flat map, and why it stayed

[ADR-0002](../adr/0002-flat-string-metadata.md) records the original
decision. The first consumers, `staticdatagen` and `ssg`, fed front
matter into templates and `<meta>` tag generation, and both want
strings. YAML, TOML and JSON have different type systems: YAML has
timestamps and tags, TOML has datetimes and arrays of tables, JSON has
neither. A typed tree would have pushed every consumer to handle three
shapes of the same data. Flattening to strings gave them one:

```text
author:
  name: Ada        ->  "author.name" = "Ada"
tags: [a, b]       ->  "tags"        = "[a, b]"
draft: false       ->  "draft"       = "false"
```

The cost is written into the same ADR: type information is lost at
the boundary, so `3` and `"3"` are the same string, and an array of
objects cannot be addressed element by element. The README's "When
not to use" section still says so.

The flat map is also an output contract. The README's output
stability guarantee treats a change in how a document flattens as a
breaking change even when no Rust signature moves, because templates
downstream depend on the exact keys. That is the main reason the map
was never replaced: replacing it would break every consumer's
templates at once.

## Where it was not enough

Three kinds of caller did not fit.

**Callers that need types.** A build step that sorts by date or skips
drafts had to parse strings back into the types the YAML already had.
0.0.7 added `extract_typed::<T: Deserialize>`, which hands the raw
block to the format's own serde deserialiser, so integers stay
integers and nested tables become nested structs (changelog 0.0.7,
issue #40). `extract_typed_borrowed` lets `&str` fields point into the
document instead of allocating.

**Callers that want less.** Every call to `extract_metadata` allocates
a `String` for every key and every value, even when the caller reads
one field and drops the map before the document. 0.0.10 added
`extract_metadata_cow`, which returns `CowMetadata<'a>`: the same
dotted keys, with values borrowed from the input wherever no
unescaping was needed ([ADR-0011](../adr/0011-borrowed-values-beside-owned-map.md)).
`CowMetadata::borrowed_values` makes that claim testable: the
changelog records that 100 plain YAML scalars give 100 borrowed
values (`tests/test_events.rs`).

**Callers that want to stop early.** A sitemap generator scanning a
large manifest does not need a map at all. 0.0.10 added
`for_each_event`, which walks the block as a stream of `Event`s
(`MapStart`, `Key`, `Scalar`, `SeqStart`, `SeqEnd`, `MapEnd`) through
the format's serde deserialiser. Returning `ControlFlow::Break` from
the callback stops the walk, and nothing after that point is parsed.
`examples/streaming_example.rs` walks a 100,000-URL manifest and stops
early.

## Why add, not replace

The alternative was to change `Metadata` itself, for example to hold
`Cow<'a, str>`. ADR-0011 explains why that was rejected: a lifetime on
the type every consumer names is a breaking change for all of them,
and it would have forced the template users, who are happy with owned
strings, to think about borrowing.

So the crate now has four shapes over one parser:

| API | Shape | Allocates | Types kept |
| :--- | :--- | :--- | :--- |
| `extract_metadata` | `Metadata` (owned flat map) | every key and value | no |
| `extract_metadata_cow` | `CowMetadata<'a>` | only escaped or nested values | no |
| `extract_typed::<T>` | your struct | as `T` requires | yes |
| `for_each_event` | callback per event | nothing for borrowed strings | yes, per scalar |

They share the front-matter scanner (one `memchr` pass to find the
fences, ADR-0007) and the parse limits (`ParseLimits`, block size and
nesting depth, enforced on every path since RUSTSEC-2026-0333 showed
that a budget on one path only is not a budget; changelog 0.0.8).

The shapes do not agree on everything, and the docs say where:
`extract_metadata_cow` renders numbers with Rust's `Display` (`1.0`
reads `1`), parses YAML as written where `extract_metadata` repairs
multi-line quoted scalars, and renders a mapping inside a sequence as
`{k: v}`. Writing those differences down is what makes them part of
the contract rather than bugs waiting to be reported.

## What the numbers say, and do not say

The README reports Criterion measurements of the flat path on an
Apple A18 Pro with rustc 1.98.0: `extract_metadata` takes 631 µs on a
1 KB YAML document and 1.81 ms on 10 KB. Those numbers
describe the owned path only. No benchmark in the repository compares
`extract_metadata` with `extract_metadata_cow` or `for_each_event` on
the same input, so this paper makes no speed claim for the newer
shapes; the claim it can make is the allocation count the tests pin.
A comparison bench belongs in `benches/` before anyone quotes a ratio.

## Lessons

- An output format that templates depend on is an API even when it is
  a `HashMap`. Treat changes to it as breaking, and say so in the
  README.
- When one shape serves the first users and fails later ones, a second
  function is usually cheaper than a second type parameter on the
  first.
- Write down where two paths to the same data differ. An undocumented
  difference is a bug report; a documented one is a choice.
- Measure what you claim. The allocation count is pinned by a test;
  the speed of the new paths is not yet measured, and the docs do not
  pretend otherwise.

## Reproduce

```bash
cargo test --all-features --test test_events
cargo run --example streaming_example
cargo bench --bench metadata_benchmark
```

Links: [repository](https://github.com/sebastienrousseau/metadata-gen),
[crates.io](https://crates.io/crates/metadata-gen),
[ADR index](../adr/README.md).
