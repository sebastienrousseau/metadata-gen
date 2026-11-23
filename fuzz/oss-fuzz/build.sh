#!/bin/bash -eu
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# OSS-Fuzz build: every cargo-fuzz target, with its committed seed
# corpus zipped beside it as <target>_seed_corpus.zip.
cd "$SRC/metadata-gen"
cargo fuzz build -O
release="fuzz/target/x86_64-unknown-linux-gnu/release"
for target in $(cargo fuzz list); do
  cp "$release/$target" "$OUT/"
  (cd "fuzz/corpus/$target" && zip -q -r "$OUT/${target}_seed_corpus.zip" .)
done
