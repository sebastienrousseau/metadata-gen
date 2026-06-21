# Changelog

All notable changes to `metadata-gen` are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.4] — 2026-06-21

### Changed

- **MSRV bumped to `1.88.0`** (was `1.56.0`), pinned transitively by
  `dtt 0.0.10` → `time 0.3.47` → `time-core =0.1.8` (edition2024). The
  `time` upgrade closes a medium-severity stack-exhaustion DoS
  ([RUSTSEC time advisory](https://rustsec.org/)) and cannot be
  downgraded without re-introducing the vulnerability.
- **`dtt` 0.0.9 → 0.0.10** — pulls in the `time` security fix, optional
  `serde` feature, expanded API (`DateTime::iso_year()`, disambiguated
  timezone codes), and `pastey` migration to close RUSTSEC-2024-0436.
- **`quick-xml` 0.39 → 0.40** — `DecodingReader` BOM auto-detection,
  XML attribute normalization API, and a serde-deserializer
  `unreachable!()` panic fix discovered via libFuzzer.
- **`scraper` 0.23 → 0.27** — bumps `selectors`, `cssparser`,
  `html5ever`, and `indexmap`; no API breakage.
- **`toml` 0.8 → 1.1** — first stable major release. The public API
  surface used here (`toml::Value`, `toml::from_str`,
  `toml::de::Error`) is unchanged.
- **`yaml-rust2` 0.10 → 0.11** — bumps `hashlink` to `0.11`.
- **`criterion` 0.5 → 0.8** (dev) — drop `async-std`, MSRV bump to
  `1.86`, throughput-on-summary plots, alloca-based memory-layout
  randomisation. Bench imports updated to use `std::hint::black_box`
  (criterion's re-export is now deprecated).

### Internal

- `cargo update` refresh of the lockfile to absorb transitive bumps
  (`wasm-bindgen 0.2.125`, `web-sys 0.3.102`, `winnow 1.0.3`,
  `zerocopy 0.8.52`, `toml_datetime 1.1.1`, etc).
- Supersedes dependabot PRs #13, #14, #16, #17, #18, #19.

### Compatibility

- Source-level API surface is unchanged. Library callers only need to
  bump their MSRV to `1.88.0`.

## [0.0.3] — 2026-04-18

### Changed

- Migrated YAML extraction from the hand-rolled `crates/serde_yml`
  shim to `noyalib 0.0.8` (default-features off, `std` feature only).
- Documentation, CI, and CDN URL housekeeping.

## [0.0.2] — Earlier

- See `git log` for pre-CHANGELOG history.

## [0.0.1] — Initial release

- Initial publication of `metadata-gen` to crates.io.

[0.0.4]: https://github.com/sebastienrousseau/metadata-gen/releases/tag/v0.0.4
[0.0.3]: https://github.com/sebastienrousseau/metadata-gen/releases/tag/v0.0.3
[0.0.2]: https://github.com/sebastienrousseau/metadata-gen/releases/tag/v0.0.2
[0.0.1]: https://github.com/sebastienrousseau/metadata-gen/releases/tag/v0.0.1
