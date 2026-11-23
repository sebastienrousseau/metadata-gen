#!/bin/sh
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Builds the WASI component from this checkout and transpiles it with
# jco into src/gen/, which src/index.js imports.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "${here}/../../.." && pwd)"
(cd "${root}/wasi" && cargo build --locked --release --target wasm32-wasip2 --lib)
npx --no-install jco transpile \
  "${root}/wasi/target/wasm32-wasip2/release/metadata_gen_component.wasm" \
  -o "${here}/src/gen" --name metadata-gen --instantiation async
