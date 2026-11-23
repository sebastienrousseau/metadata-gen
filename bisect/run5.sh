#!/usr/bin/env bash
# Scratch experiment: hardware counters and alignment sensitivity of
# the YAML path across noyalib revisions. Never merged.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
root=/tmp/bisect5
mkdir -p "$root/bin"
REVS=${REVS:?}
git clone -q https://github.com/sebastienrousseau/noyalib "$root/noyalib"
lscpu | grep -E 'Model name|^CPU\(s\)'
sudo apt-get install -yq "linux-tools-$(uname -r)" linux-tools-common >/dev/null 2>&1 \
  || sudo apt-get install -yq linux-tools-azure linux-tools-common >/dev/null 2>&1 || true
sudo sysctl -q kernel.perf_event_paranoid=-1 || true
PERF=$(ls /usr/lib/linux-tools/*/perf 2>/dev/null | head -1 || true); PERF=${PERF:-perf}
"$PERF" --version || true
build() { # name rev flags
  local name=$1 rev=$2 flags=$3 dir="$root/$1"
  [ -d "$root/n-$rev" ] || git -C "$root/noyalib" worktree add -q --detach "$root/n-$rev" "$rev"
  git worktree add -q --detach "$dir" a11c4a7
  mkdir -p "$dir/examples"; cp bisect/yb.rs "$dir/examples/yc.rs"
  ( cd "$dir"
    sed -i "s#^noyalib = { version = \"=0.0.63\"#noyalib = { path = \"$root/n-$rev/crates/noyalib\"#" Cargo.toml
    CARGO_TARGET_DIR="$root/t-${flags:+a}" RUSTFLAGS="$flags" cargo build -q --profile bench --all-features --example yc
    cp "$root/t-${flags:+a}/release/examples/yc" "$root/bin/$name" )
}
names=""
for spec in $REVS; do
  name=${spec%%=*}; rev=${spec#*=}
  build "$name" "$rev" ""; names="$names $name"
done
ALIGN="-C llvm-args=-align-all-functions=6 -C llvm-args=-align-all-nofallthru-blocks=5"
for spec in $REVS; do
  name=${spec%%=*}; rev=${spec#*=}
  build "${name}a" "$rev" "$ALIGN"; names="$names ${name}a"
done
names=${names# }
echo "## perf stat, block10k x3000 (median of 5 runs each)"
for r in 1 2 3 4 5; do
  for v in $names; do
    "$PERF" stat -x, -e cycles,instructions,branch-misses,L1-icache-load-misses,iTLB-load-misses \
      "$root/bin/$v" count block10k 3000 2> "$root/ps-$v-$r.csv" >/dev/null || true
  done
done
for v in $names; do
  printf '%s' "$v"
  for ev in cycles instructions branch-misses L1-icache-load-misses iTLB-load-misses; do
    vals=$(for r in 1 2 3 4 5; do awk -F, -v e="$ev" '$3 ~ e {print $1}' "$root/ps-$v-$r.csv"; done | sort -n)
    med=$(echo "$vals" | sed -n 3p)
    printf ' %s=%s' "$ev" "${med:-na}"
  done
  echo
done
echo "## interleaved wall time"
sed -i "s#root = .*#root = '$root'#; s#{root}/{v}/target/release/examples/yb#{root}/bin/{v}#" bisect/drive.py
first=${names%% *}
python3 -I bisect/drive.py "${names// /,}" "${ROUNDS:-20}" "$root/wall.json"
python3 -I bisect/report.py "$root/wall.json" "$first"
