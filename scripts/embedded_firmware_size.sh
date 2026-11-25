#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Linked firmware footprint of a yaml-only build (#61).
#
# Links embedded/size-probe, a minimal no_std image, twice with LTO and
# --gc-sections: once as a baseline that only allocates, once calling
# `extract_metadata` on YAML front matter. Prints, as a Markdown table,
# the flash each image needs (.text + .rodata + .data, from llvm-size)
# and the difference, which is what metadata-gen adds to a firmware
# that already has a heap.
#
# The probe resolves against a copy of the repository's Cargo.lock, so
# it measures exactly the dependency versions the crate ships with and
# has no lockfile of its own to drift.
#
# Usage: scripts/embedded_firmware_size.sh <target> [profile]
#   profile: release (opt-level s, the default) or release-z
# Needs: rustup component add llvm-tools; rustup target add <target>
set -euo pipefail

target="${1:?usage: embedded_firmware_size.sh <target> [profile]}"
profile="${2:-release}"
cd "$(dirname "$0")/../embedded/size-probe"
cp ../../Cargo.lock Cargo.lock

host="$(rustc -vV | sed -n 's/^host: //p')"
llvm_size="$(rustc --print sysroot)/lib/rustlib/${host}/bin/llvm-size"

# Builds the probe with the given features and prints its ELF path.
build() {
  cargo build --profile "$profile" --target "$target" \
    --features "$1" --message-format=json-render-diagnostics |
    sed -n 's/.*"executable":"\([^"]*\)".*/\1/p'
}

# Flash footprint: the sections a device stores, not .bss or the stack.
flash_of() {
  "$llvm_size" -A "$1" |
    awk '$1 == ".text" || $1 == ".rodata" || $1 == ".data" { s += $2 } END { print s }'
}

baseline="$(flash_of "$(build "")")"
with_crate="$(flash_of "$(build metadata-gen)")"
added=$((with_crate - baseline))

echo "| Target | Profile | Baseline image | yaml-only image | metadata-gen adds |"
echo "| :--- | :--- | ---: | ---: | ---: |"
printf "| \`%s\` | \`%s\` | %d B | %d B | %d B |\n" \
  "$target" "$profile" "$baseline" "$with_crate" "$added"
