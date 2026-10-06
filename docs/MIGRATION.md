# Migration guide

What changes for a consumer between releases, in the order they shipped.
Each entry names the release, what moved, and what to do about it. The
[CHANGELOG](../CHANGELOG.md) has the full record; this page keeps only
the entries that need action.

Releases increment by 0.0.1 and the crate is pre-1.0, so under semver
every release may break. In practice the public API has been additive
since 0.0.5; the items below are the exceptions.

## 0.0.7 to 0.0.8

No API change. One behaviour change:

- `extract_and_prepare_metadata` used to return `ExtractionError` for
  TOML front matter that contained no `:` character, even when the block
  was valid `key = "value"` TOML. It now parses it. Code that relied on
  the error for such input should check the returned map instead.

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
