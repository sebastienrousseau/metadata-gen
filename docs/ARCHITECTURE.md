# `metadata-gen` architecture

How the crate is put together, for contributors. The user-facing story
is in the [README](../README.md); this page is about the shape of the
code and the decisions behind it.

## Layout

```text
metadata-gen/
├── src/
│   ├── lib.rs        # public surface, type aliases, extract_and_prepare_metadata
│   ├── metadata.rs   # front-matter detection, parsing, flattening, processing
│   ├── metatags.rs   # <meta> generation (groups) and extraction (quick-xml)
│   ├── utils.rs      # HTML escape/unescape, async file helper
│   └── error.rs      # MetadataError and ContextError
├── tests/            # one integration suite per module + an end-to-end pass
├── examples/         # one runnable example per module
├── benches/          # Criterion harness
├── fuzz/             # libFuzzer targets, seed corpus, regression inputs
├── docs/adr/         # architecture decision records
└── supply-chain/     # cargo-vet state
```

Four modules, one direction of dependency: `lib.rs` → `metadata.rs` →
`error.rs`, and `lib.rs` → `metatags.rs` → `utils.rs`. Nothing in
`metadata.rs` knows about HTML; nothing in `metatags.rs` knows about
front matter.

## End-to-end pipeline

`extract_and_prepare_metadata(content)` is the whole crate in one call:

1. **Detect and parse** (`metadata::extract_metadata`). The content is
   tried against three front-matter shapes in order: a `---` YAML fence,
   a `+++` TOML fence, then a JSON object at the top of the document.
   The first shape that *matches* wins; a shape that matches but fails
   to parse is reported as that parser's error rather than falling
   through, so a broken YAML block never silently becomes "no front
   matter".
2. **Flatten** into `Metadata`, a `HashMap<String, String>`. Nested
   tables become dotted keys (`author.name`); sequences render as
   `[a, b]`; every other scalar takes its `Display` form. The three
   flatteners (`flatten_yaml_recursive`, `flatten_toml`, `flatten_json`)
   are deliberately parallel so a change to one is visibly missing from
   the others.
3. **Process** (`metadata::process_metadata`): dates are normalised to
   `YYYY-MM-DD` (accepting ISO and `DD/MM/YYYY`), required fields
   (`title`, `date`) are checked, and a `slug` is derived from the title
   when absent.
4. **Derive keywords** (`lib::extract_keywords`) from the `keywords`
   field, split on commas.
5. **Generate tags** (`metatags::generate_metatags`): primary,
   Open Graph, Twitter, Apple and Microsoft groups, each a string of
   `<meta>` elements with attribute values passed through
   `utils::escape_html`.

The reverse direction, `metatags::extract_meta_tags`, is a single
streaming pass with `quick-xml` over an HTML document. Both `<meta …>`
and `<meta … />` shapes are handled; attribute names are compared
case-insensitively; entity references in values are decoded. Malformed
markup ends the scan and returns what was found so far, because the
common input is a whole HTML page that a strict XML reader will not
accept end to end.

## Errors

`MetadataError` is one enum with a variant per failure class and
`#[from]` conversions for the three parsers, I/O and UTF-8.
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
