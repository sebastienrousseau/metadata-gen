# `metadata-gen` architecture

How the crate is put together, for contributors. The user-facing story
is in the [README](../README.md); this page is about the shape of the
code and the decisions behind it.

## Layout

```text
metadata-gen/
├── src/
│   ├── lib.rs        # public surface, type aliases, extract_and_prepare_metadata
│   ├── metadata.rs   # Metadata, ParseLimits, detection and extraction entry points
│   ├── metadata/
│   │   ├── scan.rs         # fence and JSON-object scanning (memchr, no regex)
│   │   ├── front_matter.rs # per-format parsing and flattening, under ParseLimits
│   │   ├── date.rs         # date normalisation on the `time` crate
│   │   └── process.rs      # ProcessOptions, DateOrder, required and derived fields
│   ├── metatags.rs   # MetaTag, MetaTagGroups and <meta> generation
│   ├── metatags/
│   │   └── html.rs   # <meta> extraction (quick-xml, feature `html`)
│   ├── utils.rs      # HTML and attribute escaping, the 0.0.7 async file helper
│   ├── io.rs         # std::io readers and files (feature `std`)
│   ├── tokio.rs      # Tokio readers and files (feature `tokio`)
│   └── error.rs      # MetadataError and ContextError
├── tests/            # one integration suite per module + an end-to-end pass
├── examples/         # one runnable example per module
├── benches/          # Criterion harness
├── fuzz/             # libFuzzer targets, seed corpus, regression inputs
├── docs/adr/         # architecture decision records
└── supply-chain/     # cargo-vet state
```

One direction of dependency: `lib.rs` → `metadata.rs` → `error.rs`,
and `lib.rs` → `metatags.rs` → `utils.rs`; `io.rs` and `tokio.rs` sit on
top and only read text before handing it to `lib.rs`. Nothing in
`metadata.rs` knows about HTML; nothing in `metatags.rs` knows about
front matter.

Every format and integration is a Cargo feature (`std`, `yaml`, `toml`,
`json`, `html`, `tokio`, all on by default). Without `std` the crate is
`no_std + alloc` and `MetadataMap` is a `BTreeMap`; a fence whose format
is compiled out is reported as `UnsupportedFormatError`, never as "no
front matter". See [ADR-0005](adr/0005-feature-flags-and-no-std.md).

## End-to-end pipeline

`extract_and_prepare_metadata(content)` is the whole crate in one call:

1. **Detect and parse** (`metadata::extract_metadata`). The content is
   tried against three front-matter shapes in order: a `---` YAML fence,
   a `+++` TOML fence, then a JSON object at the top of the document.
   `metadata/scan.rs` finds the fences with `memchr` (a fence sits on a
   line of its own; a UTF-8 BOM and leading whitespace are allowed). The
   first shape that *matches* wins; a shape that matches but fails to
   parse is reported as `MetadataError::Parse` for that format rather
   than falling through, so a broken YAML block never silently becomes
   "no front matter", and an opening fence with no closing fence says so
   with its byte offset. Every parse runs under `ParseLimits`: the block
   size is checked first, and the YAML parser gets its strict budget
   preset with the caller's depth limit, on the typed path as well.
2. **Flatten** into `Metadata`, a `MetadataMap` (`HashMap<String,
   String>` with `std`). An empty block flattens to an empty map. Nested
   tables become dotted keys (`author.name`); sequences render as
   `[a, b]`; every other scalar takes its `Display` form. The three
   flatteners (`flatten_yaml_recursive`, `flatten_toml`, `flatten_json`)
   are deliberately parallel so a change to one is visibly missing from
   the others.
3. **Process** (`metadata::process_metadata`): dates are normalised to
   `YYYY-MM-DD` (accepting RFC 3339, ISO 8601 and `DD/MM/YYYY`, or
   `MM/DD/YYYY` with `DateOrder::MonthFirst`), required fields
   (`title`, `date`) are checked, and a `slug` is derived from the title
   when absent.
4. **Derive keywords** (`lib::extract_keywords`) from the `keywords`
   field, split on commas.
5. **Generate tags** (`metatags::generate_metatags`): primary,
   Open Graph, Twitter, Apple and Microsoft groups, each a string of
   `<meta>` elements, one per line. Open Graph tags (`og:`, `article:`,
   `fb:`, `profile:`, `book:`, `music:`, `video:`) use `property=`, the
   rest `name=`, and both attribute values pass through
   `utils::escape_attribute`. `MetaTagGroups::iter` reads the groups back
   as `MetaTag` values.

The reverse direction, `metatags::extract_meta_tags`, is a single
streaming pass with `quick-xml` over an HTML document. Both `<meta …>`
and `<meta … />` shapes are handled; attribute names are compared
case-insensitively; entity references in values are decoded. Malformed
markup ends the scan and returns what was found so far, because the
common input is a whole HTML page that a strict XML reader will not
accept end to end; `extract_meta_tags_lenient` also returns where the
scan stopped.

## Errors

`MetadataError` is one `#[non_exhaustive]` enum with a variant per
failure class. A front-matter parser's rejection is `Parse { format,
span, source }`: the format, the byte range inside the block when the
parser gave one, and the parser's own error as the source. The
`#[from]` conversions for the three parsers and I/O remain for callers
that build errors themselves.
`MetadataError::context(msg)` rewrites every variant to carry a prefix;
parser errors become `custom` errors of the same parser type so the
variant is preserved. `Other` wraps a boxed error in `ContextError`,
which keeps the original reachable through `Error::source`.

## Tests

- `src/**` `#[cfg(test)]` modules pin behaviour next to the code,
  including every `context` arm and every flattener leaf kind.
- `tests/test_*.rs` are the per-module integration suites;
  `tests/test_integration.rs` drives the whole pipeline.
- Doc tests on every public item run under `cargo test`.
- `fuzz/` holds three libFuzzer targets: `fuzz_extract_metadata`,
  `fuzz_extract_meta_tags` and `fuzz_html_escape` (escape/unescape
  round trip). The seed corpus and every fixed-bug reproducer replay on
  each push.

## Coverage

The gate is 98% lines, measured with `cargo llvm-cov --all-features`.
Test-only code counts: the uncovered lines that remain are the
`_ => panic!` arms inside tests, unreachable by construction. See
[`DEVELOPMENT.md`](../DEVELOPMENT.md) for the command.

## Where to read next

- [`docs/adr/`](adr/README.md) for the decisions that shape the above.
- [`DEVELOPMENT.md`](../DEVELOPMENT.md) for reproducing every CI gate.
