#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Object size of the no_std build on an embedded target (#61).
#
# Builds the library optimised for size without LTO (so each rlib holds
# real object code, not bitcode) and prints, as a Markdown table, the
# text + data + bss that llvm-size reports for metadata-gen's own rlib
# and for every rlib in its dependency graph. The second figure is an
# upper bound: a firmware link keeps only the functions it calls.
#
# Usage: scripts/embedded_size.sh <target> <features>
# Needs: rustup component add llvm-tools; rustup target add <target>
set -euo pipefail

target="${1:?usage: embedded_size.sh <target> <features>}"
features="${2:?usage: embedded_size.sh <target> <features>}"
cd "$(dirname "$0")/.."

host="$(rustc -vV | sed -n 's/^host: //p')"
llvm_size="$(rustc --print sysroot)/lib/rustlib/${host}/bin/llvm-size"
out="target/embedded-size"

# The rlib paths come from cargo itself: the on-disk layout of the
# target directory is not stable across cargo releases.
rlibs="$(
  CARGO_PROFILE_RELEASE_LTO=false \
  CARGO_PROFILE_RELEASE_OPT_LEVEL=s \
  CARGO_TARGET_DIR="$out" \
    cargo build --lib --release --locked --no-default-features \
      --features "$features" --target "$target" \
      --message-format=json-render-diagnostics |
    python3 -I -c '
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    if msg.get("reason") == "compiler-artifact":
        for name in msg["filenames"]:
            if name.endswith(".rlib"):
                print(msg["target"]["name"] + "\t" + name)
'
)"

# `llvm-size -t` ends with a totals line: text data bss dec hex.
total_of() { "$llvm_size" -t "$@" | awk 'END { print $1 + $2 + $3 }'; }

own_rlib="$(awk -F'\t' '$1 == "metadata_gen" { print $2 }' <<<"$rlibs")"
mapfile -t all_rlibs < <(cut -f2 <<<"$rlibs")
own="$(total_of "$own_rlib")"
all="$(total_of "${all_rlibs[@]}")"

echo "| Target | Features | metadata-gen | With every dependency (upper bound) |"
echo "| :--- | :--- | ---: | ---: |"
printf '| `%s` | `%s` | %d B | %d B |\n' "$target" "$features" "$own" "$all"
