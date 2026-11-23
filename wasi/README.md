<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# metadata-gen WASI 0.2 component

This package builds `metadata-gen` as a WebAssembly component for
`wasm32-wasip2`, exporting the interface in
[`../wit/metadata-gen.wit`](../wit/metadata-gen.wit):

| Export | Rust function behind it |
| :--- | :--- |
| `extract(content) -> result<list<entry>, string>` | `extract_metadata`, entries sorted by key |
| `extract-meta-tags(html) -> list<meta-tag>` | `metatags::extract_meta_tags_lenient` |
| `escape-html(text) -> string` | `escape_html` |

The design is recorded in
[ADR-0009](../docs/adr/0009-wasi-component-shape.md). The package is
not published to crates.io; tagged releases attach the built component
as `metadata-gen-<version>.wasm`.

## Build

```bash
rustup target add wasm32-wasip2
cd wasi
cargo build --locked --release --target wasm32-wasip2 --lib
# target/wasm32-wasip2/release/metadata_gen_component.wasm
```

No `cargo-component` is needed: the `wasm32-wasip2` target links a
component directly.

## Call it

```bash
W=target/wasm32-wasip2/release/metadata_gen_component.wasm
wasm-tools component wit "$W"     # print the embedded interface
wasmtime run --invoke 'extract("---\ntitle: Hello\n---\nbody")' "$W"
# ok([{key: "title", value: "Hello"}])
```

## Check it

```bash
python3 -I wasi/check.py
```

The check builds the component, runs `wasm-tools validate`, fails if
the file is over 1 MiB, and calls every export under `wasmtime` with
each input in [`cases/`](cases/), comparing the result byte for byte
with what the native build (`src/bin/expect.rs`) prints for the same
input. CI runs it in the `WASI 0.2 component` job.

Deploy guides for hosts: [`docs/deploy/`](../docs/deploy/README.md).
