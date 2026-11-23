<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 0009. The WASI build is a separate component package with a three-function WIT interface

- **Status:** accepted
- **Date:** 2026-11-23

## Context

Edge hosts (wasmtime, Spin, wasmCloud, and JavaScript runtimes through
`jco`) load WebAssembly components described by WIT. The crate already
builds without `std` ([ADR-0005](0005-feature-flags-and-no-std.md)),
but a component needs exported bindings, and `wit-bindgen`'s generated
glue contains `unsafe` code, which the library forbids
([ADR-0001](0001-zero-unsafe-policy.md)).

Three shapes were possible: put the bindings in the library behind a
feature, publish a second crate, or keep an unpublished wrapper package
in the repository.

## Decision

- The component lives in `wasi/`, a package of its own
  (`metadata-gen-component`, `publish = false`) that depends on the
  library by path. It is not a workspace member, so the library's
  `#![forbid(unsafe_code)]`, dependency graph and `cargo-vet` scope do
  not change.
- It is built with plain `cargo build --release --target wasm32-wasip2`;
  the target's linker emits a component, so `cargo-component` is not
  needed.
- The interface is `wit/metadata-gen.wit`, package
  `metadata-gen:lib@<crate version>`, exporting one interface,
  `metadata`, with `extract`, `extract-meta-tags` and `escape-html`.
  Values cross the boundary as strings and records of strings, which is
  the shape of `Metadata` ([ADR-0002](0002-flat-string-metadata.md)).
  `extract` sorts entries by key, so the same input gives the same
  bytes on every host.
- Errors cross as the `MetadataError` display text. A structured error
  variant would freeze the Rust enum into the WIT contract while the
  enum is still `#[non_exhaustive]` and moving.
- CI builds the component, validates it with `wasm-tools`, and runs
  each export under `wasmtime`, comparing the result with the native
  build's output for the same inputs.

## Consequences

- Typed extraction (`extract_typed`) is not exported: WIT has no
  generic over user structs. Hosts get the flat map and map it
  themselves.
- The WIT package version moves with the crate version and is checked
  by `scripts/verify-release-versions.sh`.
- `wit-bindgen` and its tree are outside the library's `cargo-vet`
  graph. The component package has its own `Cargo.lock`, and its
  dependencies are reviewed when they change.
