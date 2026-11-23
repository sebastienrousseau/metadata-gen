#!/usr/bin/env bash
# Scratch helper: refresh noyalib's cargo-vet imports after the url
# 2.5.9 lock bump (network access the local machine lacks). Never merged.
set -euo pipefail
root=/tmp/vet
git clone -q --branch feat/v0.0.64 https://github.com/sebastienrousseau/noyalib "$root/noyalib"
cd "$root/noyalib"
cargo install -q --locked cargo-vet --version 0.10.2
cargo update -q -p url@2.5.8 --precise 2.5.9
cargo vet --locked || true
cargo vet
cargo vet --locked
git diff --stat
git diff > "$root/vet.patch"
