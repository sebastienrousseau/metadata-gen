#!/usr/bin/env bash
# Scratch experiment: per-function cachegrind diff of the YAML path
# between two noyalib revisions. Never merged.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
root=/tmp/bisect
mkdir -p "$root/bin" "$root/cg"
export CARGO_TARGET_DIR="$PWD/target"
REVS=${REVS:?}
git clone -q https://github.com/sebastienrousseau/noyalib "$root/noyalib"
names=""
for spec in $REVS; do
  name=${spec%%=*}; rev=${spec#*=}; dir="$root/$name"; names="$names $name"
  git -C "$root/noyalib" worktree add -q --detach "$root/n-$name" "$rev"
  git worktree add -q --detach "$dir" a11c4a7
  mkdir -p "$dir/examples"; cp bisect/yb.rs "$dir/examples/yc.rs"
  ( cd "$dir"
    sed -i "s#^noyalib = { version = \"=0.0.63\"#noyalib = { path = \"$root/n-$name/crates/noyalib\"#" Cargo.toml
    cargo build -q --profile bench --all-features --example yc
    cp "$CARGO_TARGET_DIR/release/examples/yc" "$root/bin/$name"
  )
done
valgrind --version
for v in $names; do
  valgrind --tool=cachegrind --cache-sim=yes --branch-sim=yes --cachegrind-out-file="$root/cg/$v.out" "$root/bin/$v" count block1k 300 2>&1 | grep -E 'refs|misses|mispred|Mispred'
  cg_annotate --show-percs=no --threshold=0.3 "$root/cg/$v.out" 2>&1 | sed -n '/Function summary/,/Function:file summary/p' | head -60 || true
done
set -- $names
echo "## diff $1 -> $2"
cg_annotate --diff --mod-filename='s#/tmp/bisect/n-[^/]*/##' --threshold=0.1 "$root/cg/$1.out" "$root/cg/$2.out" 2>&1 | sed -n '/Function summary/,/Function:file summary/p' | head -80 || true
