#!/usr/bin/env bash
# Scratch experiment: noyalib's own deserialize benches, v0.0.63
# against the fix, twice in A/B order. Never merged.
set -euo pipefail
root=/tmp/bench4
git clone -q https://github.com/sebastienrousseau/noyalib "$root/noyalib"
cd "$root/noyalib"
git worktree add -q --detach "$root/a" v0.0.63
git worktree add -q --detach "$root/b" "${FIX:?}"
export CARGO_TARGET_DIR="$root/target" CRITERION_HOME="$root/criterion"
f='deserialize/from_str_(value|with_config_strict|with_config_default)$'
for pass in 1 2; do
  (cd "$root/a" && cargo bench -q -p noyalib --bench benchmarks -- "$f" --save-baseline "v63_$pass" > /dev/null)
  (cd "$root/b" && cargo bench -q -p noyalib --bench benchmarks -- "$f" --baseline "v63_$pass" 2>&1 | grep -E '^deserialize|time:|change:' || true)
done
