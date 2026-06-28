# Changelog

All notable changes to `metadata-gen` are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.5] — Unreleased

This release opens the post-audit roadmap (v0.0.5 → v0.0.10). v0.0.5 is the
**Foundation Hardening** milestone; the full breakdown lives at
<https://github.com/sebastienrousseau/metadata-gen/milestones>.

### Changed

- **`tokio` feature set trimmed** from `full` to `default-features = false`
  plus `["fs", "io-util", "rt", "macros"]`. The previous `full` opt-in
  dragged `net`, `signal`, `process`, `tokio-macros`, and `parking_lot`
  through every consumer build for the sake of one `read_to_string`. The
  full Tokio bundle is now a `dev-dependency` only, so doc-tests and
  examples still build, but library users no longer pay for it.
- **First-party crates pinned strictly** — `noyalib = "=0.0.8"` and
  `dtt = "=0.0.10"`. Pre-1.0 patch releases from the same maintainer can no
  longer silently break downstream consumers; future bumps go through a
  deliberate `metadata-gen` release. Captured as ADR-004.
- **`[profile.bench]` corrected** — dropped `debug = true` (which defeated
  inliner heuristics and inflated reported latency by ~10–30 %) in favour
  of `debug = "line-tables-only"` with `lto = true`, `codegen-units = 1`,
  `opt-level = 3` to mirror release codegen.
- **Crates.io metadata refresh** — removed the `command-line-utilities`
  category (the crate ships no `[[bin]]`); keyword set rotated to
  `frontmatter`, `yaml`, `toml`, `seo`, `static-site` (the crate name
  `metadata-gen` is implicit and was removed).

### Removed

- **`anyhow` dependency** — declared but never `use`d in `src/`.
- **`tempfile` from `[dependencies]`** — moved to `[dev-dependencies]`; it
  was only referenced by tests and examples.

### CI / supply chain

- **Local rustdoc workflow** (`.github/workflows/docs.yml`) replaces the
  external `pipelines/docs.yml` reusable workflow. Docs are built with
  `RUSTDOCFLAGS="--cfg docsrs --deny warnings"` and deployed via
  `actions/upload-pages-artifact@v3` + `actions/deploy-pages@v4`. The old
  `gh-pages` branch deploy is retired and the Pages source has been
  switched to "GitHub Actions".
- **`cargo-deny` and `cargo-audit` gating** added to the CI workflow.
  Advisories, licenses, bans, and source allowlists now fail the build on
  any violation.
- **Dependabot config sharpened** — security advisories ship as their own
  PR (not bundled with weekly maintenance), `versioning-strategy` set to
  `increase-if-necessary`, first-party 0.0.x crates (`noyalib`, `dtt`)
  are excluded from automated minor/patch bumps to honour ADR-004.
- **Repo positioning refreshed** — GitHub description rewritten for the
  2026 audit context, topics updated to surface `frontmatter`, `wasi`,
  `sbom`, `supply-chain`, `cargo-deny`, `no-std-friendly`, and the
  static-site-generator audience.

### Docs

- **README rewritten end-to-end** — value-prop, when-to-use, three
  per-format examples, a comparison table vs `gray_matter` /
  `yaml-front-matter` / `matter`, supply-chain section, MSRV policy,
  roadmap table, and a 12-question FAQ. Every code block is a passing
  doc-test under `cargo test --doc`.

### Roadmap (planned / tracked under v0.0.5 milestone)

The structural items from the audit are tracked as individual issues for
review and acceptance before merge. Highlights:

- **#22** — replace `scraper` with `quick-xml`-based meta extractor
  (silences RUSTSEC-2025-0057, RUSTSEC-2026-0097).
- **#25** — hoist YAML/TOML/JSON regexes to `LazyLock<Regex>` statics
  (expected 3–10× throughput on cold path).
- **#26** — fix the JSON front-matter detector to handle nested objects
  (the current non-greedy regex truncates at the first `}`).
- **#27** — wire `cargo-deny` and `cargo-audit` as gating CI checks.
- **#28** — emit a CycloneDX SBOM and attach it to GitHub Releases.

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
