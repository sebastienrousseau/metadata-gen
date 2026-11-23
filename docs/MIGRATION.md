# Migration guide

What changes for a consumer between releases, in the order they shipped.
Each entry names the release, what moved, and what to do about it. The
[CHANGELOG](../CHANGELOG.md) has the full record; this page keeps only
the entries that need action.

Releases increment by 0.0.1 and the crate is pre-1.0, so under semver
every release may break. The public API was additive from 0.0.5 to
0.0.7; 0.0.8 and 0.0.9 carry breaking changes, listed below, and
0.0.10 is additive again.

## 0.0.9 to 0.0.10

Additive: no signature changes and no change to what
`extract_metadata` or `generate_metatags` produce.

- **`extract_typed_borrowed` on YAML that cannot be borrowed** (a tag
  such as `!x`) returns `MetadataError::ExtractionError` naming
  `extract_typed`, instead of a `MetadataError::Parse` whose message
  was `$__noyalib_streaming_fallback`. Use `extract_typed` for such
  documents.
- **`ProcessOptions` has a new public field, `optional_fields`.** It is
  `#[non_exhaustive]`, so code that builds it through
  `ProcessOptions::default()` or `ProcessOptions::new()` is unaffected.
- New, additive: `for_each_event`, `for_each_event_with_limits`,
  `events_into`, `events`, `Event`,
  `Scalar`, `extract_metadata_cow`, `CowMetadata`, `Metadata::builder`,
  `Metadata::keywords`, `Metadata::keys`, `ProcessOptions::{new,
  require, optional, unknown_fields, process}`, `MetadataProcessor`,
  the `schema_org` module, and the `futures-io` and
  `embedded-io-async` features.

## 0.0.8 to 0.0.9

- **Minimum Supported Rust Version (MSRV) raised to Rust 1.89.0** (from 1.88.0).
- **`extract_meta_tags` (feature `html`) uses `oxml = "=0.0.13"` instead of `quick-xml`.**
  Public function signatures and extracted `MetaTag` values are unchanged; if your crate also depends on `oxml` or `noyalib`, align with `oxml = "0.0.13"` and `noyalib = "0.0.58"`.

## 0.0.7 to 0.0.8

The release that fixes RUSTSEC-2026-0333 (through `noyalib` 0.0.56) and
makes every format a Cargo feature. A default build keeps every entry
point 0.0.7 had; what needs action is below.

### Errors

- **A front-matter parser's rejection is `MetadataError::Parse`.**
  `extract_metadata`, `extract_typed` and friends used to return
  `YamlError`, `TomlError` or `JsonError` (or an `ExtractionError`
  reading "JSON parse error in frontmatter") for a block that failed to
  parse. They now return `Parse { format, span, source }`: match on
  `format` instead of the variant, and read the parser's error through
  `source`. The old variants and their `From` impls remain for errors
  you build yourself.
- **An opening fence with no closing fence says so.** `---` with no
  closing `---` used to be "No valid front matter found."; it is now an
  `ExtractionError` naming the format and the fence's byte offset.
- **`MetadataError` is `#[non_exhaustive]`.** Add a wildcard arm to
  exhaustive matches:

  ```rust
  use metadata_gen::{extract_metadata, MetadataError};

  fn describe(content: &str) -> String {
      match extract_metadata(content) {
          Ok(_) => "ok".to_string(),
          Err(MetadataError::Parse { format, .. }) => {
              format!("{format} front matter did not parse")
          }
          Err(MetadataError::MissingFieldError(field)) => {
              format!("missing {field}")
          }
          // Needed from 0.0.8 on: new variants may be added.
          Err(other) => other.to_string(),
      }
  }
  ```
- **`From<Box<dyn Error + Send + Sync>>` is gone.** A boxed error no
  longer turns into `MetadataError::Other` through `?` by accident; call
  `MetadataError::other(err)` or build `MetadataError::Other(boxed)`
  where you mean it.
- **`MetadataError::Utf8Error` is deprecated.** Nothing in the crate
  produces it; it stays for code that matches on it.

### Output

`generate_metatags` and `process_metadata` output changes, which the
README's output-stability policy treats as breaking:

- **Open Graph tags use `property=`.** `og:*` (and `article:`, `fb:`,
  `profile:`, `book:`, `music:`, `video:`) tags render as
  `<meta property="og:title" …>`, as the Open Graph protocol requires.
  Twitter and every other tag keep `name=`.
- **Attribute values escape `&`, `<` and `>` as well as `"`.** A value
  that already holds an entity (`Fish &amp; Chips`) is now escaped again
  (`Fish &amp;amp; Chips`); pass plain text.
- **Slugs keep letters and digits only.** `slug` used to be the title
  lower-cased with spaces turned into `-` (`"Hello, World!"` gave
  `hello,-world!`); runs of anything else now collapse to one `-` and
  leading or trailing separators are dropped (`hello-world`). Set
  `slug` in front matter to keep an old URL.
- **An empty block is an empty map.** `---\n---` and `---\n\n---`
  used to give one entry under the empty key (`{"": "null"}`) or no
  match at all; both are now empty metadata, as `+++\n+++` and `{}`
  already were.
- **Front matter after a UTF-8 BOM is found.** It used to be missed.
- **A TOML fence sits on a line of its own.** `+++ a = 1 +++` on one
  line used to parse; it is now not front matter.

### Types and features

- **`MetaTag`, `MetaTagGroups` are `#[non_exhaustive]`.** Build a tag
  with `MetaTag::new(name, content)` and groups with
  `MetaTagGroups::default()`; struct literals no longer compile.

  ```rust
  use metadata_gen::{MetaTag, MetaTagGroups};

  // Was: MetaTag { name: "og:title".into(), content: "Hello".into() }
  let tag = MetaTag::new("og:title", "Hello");
  // Was: MetaTagGroups { primary: String::new(), .. }
  let groups = MetaTagGroups::default();
  ```
- **New features, all on by default.** `std`, `yaml`, `toml`, `json`,
  `html`, `tokio`. If you set `default-features = false`, list the
  formats you parse; without `std` the crate is `no_std + alloc` and
  `MetadataMap` is a `BTreeMap`.
- **`tokio` dropped its `rt` and `macros` features.** The crate only
  needs `fs` and `io-util`. If you leaned on this crate for
  `#[tokio::main]`, enable `rt` and `macros` on your own `tokio`.
- **`dtt` and `regex` left the dependency tree.** Dates are parsed with
  `time`. A consumer that used either through this crate must depend on
  it directly.
- New, additive: `ParseLimits` and `extract_metadata_with_limits`,
  `extract_typed_borrowed`, `DateOrder` and `ProcessOptions::date_order`,
  `MetaTag::new`, `MetaTag::render`, `MetaAttribute`,
  `MetaTagGroups::iter`, `extract_meta_tags_lenient`,
  `escape_attribute`, `escape_html_cow`, `MetadataError::other`, and the
  `metadata_gen::io` and `metadata_gen::tokio` file helpers.
- Unchanged from the 0.0.8 cycle: TOML front matter that contains no `:`
  is accepted by `extract_and_prepare_metadata` instead of being
  rejected.

## 0.0.6 to 0.0.7

- **The `advanced_parsing` Cargo feature is gone.** It gated nothing.
  Remove it from `features = [...]` in your `Cargo.toml`; nothing else
  changes.
- **`unescape_html` decodes each entity once.** It used to run a chain of
  `replace` calls, so `&amp;lt;` became `<`. Input that depended on the
  double decode now gets `&lt;`, which is the correct result.
- **`build.rs` is gone; `rust-version = "1.88.0"` is the floor.** Cargo
  enforces it. A toolchain older than 1.88 that used to fail with the
  build script's message now fails with Cargo's.
- New, additive: `extract_typed`, `extract_metadata_with_body`,
  `detect_front_matter`, `ProcessOptions` and `process_metadata_with`.
  `ProcessOptions` and `FrontMatterFormat` are `#[non_exhaustive]`, so
  construct them through their constructors and match with a wildcard arm.

## 0.0.4 to 0.0.5

- **`tokio` is no longer pulled in with `full`.** The crate enables only
  `fs`, `io-util`, `rt` and `macros`. A consumer that leaned on this crate
  to bring in `net`, `signal` or `process` must add them to its own
  `tokio` dependency.
- **`anyhow` and `tempfile` left `[dependencies]`.** Neither was part of
  the public API; a consumer that used them transitively must depend on
  them directly.
- **`extract_meta_tags` runs on `quick-xml` instead of `scraper`.**
  Output order is document order and entities are decoded; a consumer
  that sorted or re-decoded the result can stop.
- **First-party crates are pinned exactly** (`noyalib`, `dtt`). If your
  tree also depends on either crate, align the version with the pin in
  this crate's `Cargo.toml` or Cargo will carry two copies.

## Earlier releases

0.0.1 to 0.0.4 predate this guide; their changes are listed in the
[CHANGELOG](../CHANGELOG.md).
