#!/usr/bin/env bash
# Scratch experiment: measure metadata-gen a11c4a7 against noyalib
# built from individual git revisions. Never merged.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
root=/tmp/bisect
mkdir -p "$root/bin"
export CARGO_TARGET_DIR="$PWD/target"
REVS=${REVS:?name=rev list}
lscpu | grep "Model name"
git clone -q https://github.com/sebastienrousseau/noyalib "$root/noyalib"
git -C "$root/noyalib" fetch -q origin 'refs/pull/*/head:refs/remotes/pr/*'
names=""
for spec in $REVS; do
  name=${spec%%=*}; rev=${spec#*=}; dir="$root/$name"; names="$names $name"
  git -C "$root/noyalib" worktree add -q --detach "$root/n-$name" "$rev"
  git worktree add -q --detach "$dir" a11c4a7
  mkdir -p "$dir/examples"; cp bisect/yb.rs "$dir/examples/yc.rs"
  ( cd "$dir"
    sed -i "s#^noyalib = { version = \"=0.0.63\"#noyalib = { path = \"$root/n-$name/crates/noyalib\"#" Cargo.toml
    grep -n '^noyalib' Cargo.toml
    cargo build -q --profile bench --all-features --example yc
    grep -A1 'name = "noyalib"' Cargo.lock | tail -1
    cp "$CARGO_TARGET_DIR/release/examples/yc" "$root/bin/$name"
  )
done
names=${names# }
echo "## cachegrind instructions per call"
for c in "small 2000" "block1k 300" "block10k 30" "doc1m 300"; do
  read -r cname n <<<"$c"
  line="$cname"
  for v in $names; do
    ir() { valgrind --tool=cachegrind --cache-sim=no --cachegrind-out-file=/dev/null "$root/bin/$v" count "$cname" "$1" 2>&1 | awk '/I *refs:/{gsub(",","",$NF); print $NF}'; }
    a=$(ir 0); b=$(ir "$n")
    line="$line $v=$(( (b - a) / n ))"
  done
  echo "$line"
done
echo "## interleaved wall time"
sed -i "s#root = .*#root = '$root'#; s#{root}/{v}/target/release/examples/yb#{root}/bin/{v}#" bisect/drive.py
first=${names%% *}
python3 -I bisect/drive.py "${names// /,}" "${ROUNDS:-30}" "$root/wall.json"
python3 -I bisect/report.py "$root/wall.json" "$first"
