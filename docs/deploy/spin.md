<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Fermyon Spin

**Status: not run.** Nothing on this page was executed for this
release: the Spin CLI was not available where the guide was written.
The steps rely on Spin's component dependencies (Spin 3.0 and later),
which let an HTTP component import an interface that another component
exports. Treat them as a starting point and report what differs.

## 1. Build the component

```bash
rustup target add wasm32-wasip2
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen/wasi
cargo build --locked --release --target wasm32-wasip2 --lib
cp target/wasm32-wasip2/release/metadata_gen_component.wasm ~/metadata-gen.wasm
```

## 2. An HTTP component that imports the interface

Create a Spin HTTP app (`spin new -t http-rust frontmatter`), copy
`wit/metadata-gen.wit` from this repository into the app's `wit/`
directory, and add the import to the app's world:

```wit
world frontmatter {
    import metadata-gen:lib/metadata@0.0.9;
}
```

Generate bindings for that world with `wit-bindgen` in the handler
crate, and call `metadata::extract(&body)` from the request handler.

## 3. Wire the dependency in `spin.toml`

```toml
[component.frontmatter.dependencies]
"metadata-gen:lib/metadata" = { path = "/home/you/metadata-gen.wasm" }
```

Spin composes the two components at load time.

## 4. Run

```bash
spin build
spin up
curl -X POST --data-binary $'---\ntitle: Hi\n---\n' http://127.0.0.1:3000/
```

`spin deploy` publishes to Fermyon Cloud with an account; also not run.
