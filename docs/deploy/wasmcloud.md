<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# wasmCloud

**Status: not run.** Nothing on this page was executed for this
release: the `wash` CLI was not available where the guide was written.
wasmCloud runs WASI 0.2 components, and the usual way to use a library
component is to compose it into the HTTP component that calls it before
deployment. Treat these steps as a starting point.

## 1. Build the component

```bash
rustup target add wasm32-wasip2
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen/wasi
cargo build --locked --release --target wasm32-wasip2 --lib
```

## 2. An HTTP component that imports the interface

Start from a wasmCloud Rust HTTP template (`wash new component
frontmatter --template-name hello-world-rust`), copy
`wit/metadata-gen.wit` into its `wit/` directory, add
`import metadata-gen:lib/metadata@0.0.10;` to its world, and call
`metadata::extract` from the `wasi:http` handler. Build it with
`wash build`.

## 3. Compose

Plug the library into the handler with
[`wac`](https://github.com/bytecodealliance/wac):

```bash
wac plug build/frontmatter_s.wasm \
  --plug ../metadata-gen/wasi/target/wasm32-wasip2/release/metadata_gen_component.wasm \
  -o frontmatter-composed.wasm
```

## 4. Deploy

Point the component entry of the app's `wadm.yaml` at
`file://./frontmatter-composed.wasm`, add an `httpserver` link as in the
template, then:

```bash
wash up -d
wash app deploy wadm.yaml
curl -X POST --data-binary $'---\ntitle: Hi\n---\n' http://127.0.0.1:8000/
```
