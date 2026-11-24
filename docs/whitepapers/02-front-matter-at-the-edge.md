<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Front matter at the edge: a WASI 0.2 component

*2026-11-23. Sources: ADR-0005, ADR-0009, `wit/metadata-gen.wit`,
`wasi/`, `examples/deploy/cloudflare-workers/`, and measurements taken
while writing it, listed with the tools that produced them.*

## The problem

A content pipeline increasingly runs in more than one place: a static
build on a laptop, a preview service on a server, and a request
handler at the edge that rewrites `<meta>` tags or validates front
matter submitted through a CMS. Shipping one parser to all of them
usually means one implementation per language, and three
implementations disagree on edge cases sooner or later.

The WebAssembly component model offers another route: compile the
Rust crate once, describe its interface in WIT, and let any
component-aware host call it with typed values. This paper describes
how `metadata-gen` became such a component in 0.0.10, how the
repository proves the component returns what the native crate
returns, and what running it in a Cloudflare Worker took.

## What made it possible

Most of the work was done two releases earlier, for a different
reason. 0.0.8 made every format a Cargo feature and the core
`no_std + alloc` ([ADR-0005](../adr/0005-feature-flags-and-no-std.md)):
the regex-based fence matcher gave way to a `memchr` scanner, and the
`dtt` date crate to `time`, because neither built without `std`. A
crate that builds for `thumbv7em-none-eabihf` has already shed the
assumptions (threads, a filesystem, a global allocator it can grow
without limit) that make WebAssembly ports painful.

## The shape of the component

[ADR-0009](../adr/0009-wasi-component-shape.md) records the decisions.
The short version:

- The component lives in `wasi/`, a separate package that is not
  published and not a workspace member. `wit-bindgen`'s generated glue
  contains `unsafe`, and the library is `#![forbid(unsafe_code)]`
  (ADR-0001); keeping the glue in its own package keeps that promise
  and keeps `wit-bindgen` out of the library's dependency graph.
- It builds with plain `cargo build --release --target wasm32-wasip2`.
  The target's linker emits a component directly, so `cargo-component`
  is not needed.
- The interface is small. `wit/metadata-gen.wit` declares package
  `metadata-gen:lib` with one interface, `metadata`, and three
  functions: `extract(content) -> result<list<entry>, string>`,
  `extract-meta-tags(html) -> list<meta-tag>`, and
  `escape-html(text) -> string`. Records hold strings only, because
  that is the shape of the flat map (ADR-0002).
- `extract` sorts its entries by key. The Rust map is a `HashMap`,
  whose order differs from run to run; a component that is meant to
  give byte-identical answers on every host cannot leak that order.
- Errors cross as the `MetadataError` display text, not as a WIT
  variant. The Rust enum is `#[non_exhaustive]` and still moving;
  freezing it into WIT would turn every new variant into a breaking
  interface change.

## Proving parity

"Compiles to WebAssembly" says little about whether it behaves the
same. `wasi/check.py` is the gate, run by the `WASI 0.2 component` CI
job and before every release:

1. `cargo fmt --check` and clippy with warnings denied, for the
   component target and natively.
2. Build the component and run `wasm-tools validate` on it.
3. Fail if it is larger than 1 MiB.
4. For each file in `wasi/cases/`, call the matching export under
   `wasmtime run --invoke`, and compare its output, byte for byte, with
   what the native `expect` binary prints for the same input. `expect`
   is built from the same `core.rs` adapters the component exports, and
   prints WAVE, the value notation wasmtime uses.

The cases cover YAML, TOML and JSON front matter, a document with no
front matter, a malformed block (so error text is compared too), a
value full of quotes, backslashes and tabs, `<meta>` extraction with
an entity reference, and HTML escaping. When this paper was written,
all eight matched (wasmtime 50.0.2, wasm-tools 1.261.0).

The component measured 634,763 bytes, built with `opt-level = "z"`,
LTO and symbols stripped. The issue that asked for the component
suggested `wasm-opt -Oz`; Binaryen does not read components yet, so
that step is not applied, and the size budget is met without it.

## Running it in a Cloudflare Worker

Workers run JavaScript and core WebAssembly modules, not components.
[`jco`](https://github.com/bytecodealliance/jco) bridges the gap:
`jco transpile --instantiation async` turns the component into three
core modules plus JavaScript glue, and the Worker passes the core
modules and the WASI shims from `@bytecodealliance/preview2-shim` to
the generated `instantiate` function. The sample is about thirty lines
(`examples/deploy/cloudflare-workers/src/index.js`): instantiate once
per isolate, call `metadata.extract` on the request body, return JSON
or a 422 with the error text.

What was checked, with jco 1.42.0 and wrangler 4.162.0 on macOS:

- `wrangler dev` (workerd, locally) served the Worker; a YAML document
  came back as `{"author.name":"Ada","title":"Hello from a Worker"}`
  and a document without front matter as HTTP 422 with the crate's
  message.
- `wrangler deploy --dry-run` bundled the Worker to 998 KiB, 286 KiB
  gzipped, inside the Workers Free plan's 3 MiB compressed limit.

What was not checked: a deployment to a Cloudflare account, and any
latency or cold-start timing. The issue that prompted this paper asked
for timing measurements of an end-to-end deploy; there are none here,
because none were taken. Spin and wasmCloud guides exist in
[`docs/deploy/`](../deploy/README.md) and are marked as not run for
the same reason.

One practical trap is worth repeating. `npx jco` without a local
install fetches an unrelated npm package called `jco`, a placeholder
published to block dependency confusion, instead of
`@bytecodealliance/jco`. The sample's build script calls
`npx --no-install jco` so it fails rather than fetch the wrong thing.

## What the component does not do

- It does not export typed extraction. WIT has no way to be generic
  over a caller's struct, so hosts get the flat map and map it
  themselves.
- It does not import `wasi:http`. It is a library component; an HTTP
  handler on Spin or wasmCloud imports its interface and is composed
  with it, as the deploy guides describe.
- Its dependencies (`wit-bindgen` and its tree) sit outside the
  library's `cargo-vet` graph and are tracked by `wasi/Cargo.lock`.

## Lessons

- Make the core `no_std` first. The component port was small because
  the hard part had been done for embedded targets.
- Fix iteration order at the boundary. A `HashMap` inside is fine; a
  `HashMap` order in the output is a portability bug.
- Test the artifact, not the source. A byte-for-byte comparison
  against native output catches encoding and escaping differences
  that a "does it build" check never will.
- Say what was not run. A deploy guide that claims a production
  deployment nobody did costs the reader more than one that says
  "verified locally".

## Reproduce

```bash
rustup target add wasm32-wasip2
python3 -I wasi/check.py
cd examples/deploy/cloudflare-workers && npm install && npm run build && npm run dev
```

Links: [repository](https://github.com/sebastienrousseau/metadata-gen),
[crates.io](https://crates.io/crates/metadata-gen),
[ADR-0009](../adr/0009-wasi-component-shape.md).
