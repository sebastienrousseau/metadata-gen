#!/bin/sh
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Builds the metadata-gen component (../../../wasi) and the HTTP
# handler (../http-handler), then plugs the first into the second with
# wac. The result, build/frontmatter.wasm, exports wasi:http and
# imports nothing from metadata-gen, so any wasmCloud host can run it.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "${here}/../../.." && pwd)"
target=wasm32-wasip2
cargo build --locked --release --target "$target" --lib \
  --manifest-path "${root}/wasi/Cargo.toml"
cargo build --locked --release --target "$target" \
  --manifest-path "${root}/examples/deploy/http-handler/Cargo.toml"
mkdir -p "${here}/build"
wac plug \
  "${root}/examples/deploy/http-handler/target/${target}/release/metadata_gen_http_handler.wasm" \
  --plug "${root}/wasi/target/${target}/release/metadata_gen_component.wasm" \
  -o "${here}/build/frontmatter.wasm"
