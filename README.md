<p align="center">
  <img src="https://cloudcdn.pro/metadata-gen/v1/logos/metadata-gen.svg" alt="Metadata Gen logo" width="128" />
</p>

<h1 align="center">Metadata Gen</h1>

<p align="center">
  <strong>A Rust library for extracting, validating, and processing metadata in YAML, TOML, and JSON formats.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/metadata-gen/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/metadata-gen/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/metadata-gen"><img src="https://img.shields.io/crates/v/metadata-gen.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/metadata-gen"><img src="https://img.shields.io/badge/docs.rs-metadata-gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/metadata-gen"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/metadata-gen?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/metadata-gen"><img src="https://img.shields.io/badge/lib.rs-v0.0.3-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Install

```bash
cargo add metadata-gen
```

Or add to `Cargo.toml`:

```toml
[dependencies]
metadata-gen = "0.0.3"
```

You need [Rust](https://rustup.rs/) 1.56.0 or later. Works on macOS, Linux, and Windows.

---

## Overview

Metadata Gen extracts, validates, and processes metadata from content files in YAML, TOML, and JSON formats.

- **Multi-format extraction** from any content file
- **HTML meta tag generation** for SEO
- **Validation** of metadata structure and required fields
- **Serde integration** for typed metadata access

---

## Features

| | |
| :--- | :--- |
| **Multi-format** | Extract metadata from YAML, TOML, and JSON |
| **Validation** | Validate metadata structure and required fields |
| **Meta tags** | Generate HTML meta tags from metadata |
| **Content files** | Process metadata from any content or data file |
| **Serde integration** | Serialize/deserialize metadata to Rust structs |

---

## Usage

```rust
use metadata_gen::extract_metadata;

fn main() {
    let content = "---\ntitle: Example\n---\nBody text.";
    let meta = extract_metadata(content).unwrap();
    println!("Title: {}", meta.get("title").unwrap());
}
```

---

## Development

```bash
cargo build        # Build the project
cargo test         # Run all tests
cargo clippy       # Lint with Clippy
cargo fmt          # Format with rustfmt
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, signed commits, and PR guidelines.

---

**THE ARCHITECT** \u1d2b [Sebastien Rousseau](https://sebastienrousseau.com)
**THE ENGINE** \u1d5e [EUXIS](https://euxis.co) \u1d2b Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#metadata-gen">Back to Top</a></p>