<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Comparison with other front-matter crates

Checked on 2026-11-23 against the latest stable release of each crate
on crates.io, by reading its published source. A cell describes the
crate's API surface, not its quality or speed; no benchmark across
crates has been run.

| Capability | `metadata-gen` 0.0.10 | `gray_matter` 0.3.2 | `yaml-front-matter` 0.1.0 | `matter` 0.1.0-alpha4 |
| :--- | :---: | :---: | :---: | :---: |
| YAML front matter parsed | Yes (`noyalib`) | Yes (`yaml-rust2`) | Yes (`serde_yaml` 0.8) | Split only |
| TOML front matter parsed | Yes (`toml`) | Yes (feature `toml`) | No | Split only |
| JSON front matter parsed | Yes (`serde_json`) | Yes (`serde_json`) | No | No |
| Typed extraction (`serde::Deserialize`) | Yes | Yes | Yes | No |
| Body returned | Yes | Yes | Yes | Yes |
| Flat dotted-key map | Yes | No | No | No |
| HTML `<meta>` generation and extraction | Yes | No | No | No |
| Parse limits (size, depth) | Yes | No | No | No |
| `no_std + alloc` build | Yes | No | No | No |

## Evidence

- **gray_matter 0.3.2**: `Matter::<Engine>::parse::<D: DeserializeOwned>`
  returns `ParsedEntity { data: Option<D>, content, .. }`
  (`src/matter.rs`, `src/entity.rs`); engines in `src/engine/` are
  `yaml.rs` (via `yaml-rust2`), `toml.rs` (feature `toml`) and `json.rs`.
- **yaml-front-matter 0.1.0**: `YamlFrontMatter::parse::<T>` returns
  `Document { metadata: T, content }` and parses with
  `serde_yaml::from_str` (`src/lib.rs`); only `---` fences.
- **matter 0.1.0-alpha4**: `matter(input) -> Option<(String, String)>`
  splits `---` or `+++` front matter from the content with a regular
  expression and returns both as strings without parsing them
  (`src/lib.rs`).
- **metadata-gen**: `extract_metadata`, `extract_typed`,
  `extract_metadata_with_body`, `metatags::generate_metatags`,
  `metatags::extract_meta_tags`, `ParseLimits`, and the `no_std` build
  checked in CI (ADR-0005).

"No" means the published API does not offer it, not that it cannot be
built on top. Corrections are welcome as issues with the crate version
and the source line.
