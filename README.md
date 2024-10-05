<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/metadata-gen/images/logos/metadata-gen.svg"
alt="Metadata Gen logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# Metadata Gen (metadata-gen)

A powerful Rust library for extracting, validating, and processing metadata in YAML, TOML, and JSON formats from any content or data file.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

`metadata-gen` is a robust Rust library designed for extracting, validating, and processing metadata from various content and data file formats. It focuses on the following key areas:

- Parsing and serialization of metadata in YAML, TOML, and JSON formats
- Extraction of frontmatter metadata from content files
- Generation and management of meta tags for web content
- Efficient processing of both local files and string content
- Flexible integration with static site generators and content management systems

Key features include:

- A unified API for handling metadata across different formats
- Robust error handling and reporting
- Customizable parsing options to accommodate various metadata structures
- Asynchronous file processing capabilities
- Utility functions for HTML entity escaping and unescaping
- Type-safe metadata value handling through a custom `Value` enum
- Generation of SEO-friendly meta tags from extracted metadata

`metadata-gen` aims to provide a stable and powerful foundation for metadata management across all platforms supported by Rust.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
metadata-gen = "0.0.1"
```

## Usage

Here are some examples of how to use the library:

### Extracting Metadata

```rust
use metadata_gen::extract_and_prepare_metadata;

let content = r#"---
title: My Post
description: A sample post
keywords: rust, metadata, extraction
---
Content here"#;

let result = extract_and_prepare_metadata(content);
if let Ok((metadata, keywords, meta_tags)) = result {
    println!("Metadata: {:?}", metadata);
    println!("Keywords: {:?}", keywords);
    println!("Meta tags: {}", meta_tags);
}
```

### HTML Escape/Unescape Example

```rust
use metadata_gen::utils::{escape_html, unescape_html};

let original = "Hello <World> & \"Friends\"";
let escaped = escape_html(original);
let unescaped = unescape_html(&escaped);

println!("Original: {}", original);
println!("Escaped: {}", escaped);
println!("Unescaped: {}", unescaped);
```

### Asynchronous File Processing

```rust
use metadata_gen::utils::async_extract_metadata_from_file;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (metadata, keywords, meta_tags) = async_extract_metadata_from_file("./examples/post.md").await?;
    println!("Metadata: {:?}", metadata);
    println!("Keywords: {:?}", keywords);
    println!("Meta tags: {}", meta_tags);
    Ok(())
}
```

## Documentation

For full API documentation, please visit [docs.rs/metadata-gen][04].

## Examples

To run the examples, clone the repository and use the following command:

```shell
cargo run --example example_name
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Acknowledgements

Special thanks to all contributors who have helped build the `metadata-gen` library.

[00]: https://metadata-gen.com
[01]: https://lib.rs/crates/metadata-gen
[02]: https://github.com/sebastienrousseau/metadata-gen/issues
[03]: https://crates.io/crates/metadata-gen
[04]: https://docs.rs/metadata-gen
[05]: https://github.com/sebastienrousseau/metadata-gen/blob/main/CONTRIBUTING.md
[06]: https://codecov.io/gh/sebastienrousseau/metadata-gen
[07]: https://github.com/sebastienrousseau/metadata-gen/actions?query=branch%3Amain
[08]: https://www.rust-lang.org/
[09]: https://github.com/sebastienrousseau/metadata-gen

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/metadata-gen/release.yml?branch=main&style=for-the-badge&logo=github
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/metadata-gen?style=for-the-badge&token=Q9KJ6XXL67&logo=codecov
[crates-badge]: https://img.shields.io/crates/v/metadata-gen.svg?style=for-the-badge&color=fc8d62&logo=rust
[docs-badge]: https://img.shields.io/badge/docs.rs-metadata--gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/metadata--gen-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.1-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
