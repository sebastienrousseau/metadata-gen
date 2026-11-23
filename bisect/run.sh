#!/usr/bin/env bash
# Scratch experiment: which dependency slows the YAML benches on
# ubuntu-latest. Never merged.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
root=/tmp/bisect
mkdir -p "$root/bin"
export CARGO_TARGET_DIR="$PWD/target"
VARIANTS=${VARIANTS:-"base v58 v59 v60 v62 v63 r63"}

mk() { # name sha noyalib-version(or -) revert-regex-toml(0/1)
  local name=$1 sha=$2 ver=$3 rev=$4 dir="$root/$1"
  git worktree add -q --detach "$dir" "$sha"
  mkdir -p "$dir/examples"; cp bisect/yb.rs "$dir/examples/yc.rs"
  ( cd "$dir"
    if [ "$ver" != - ]; then
      sed -i "s/noyalib = { version = \"=0.0.63\"/noyalib = { version = \"=$ver\"/" Cargo.toml
      cargo update -q -p noyalib --precise "$ver"
    fi
    if [ "$rev" = 1 ]; then
      cargo update -q -p regex --precise 1.13.1
      cargo update -q -p regex-automata --precise 0.4.16
      cargo update -q -p regex-syntax --precise 0.8.11
      cargo update -q -p toml --precise 1.2.0+spec-1.1.0
      cargo update -q -p toml_datetime --precise 1.1.2+spec-1.1.0
      cargo update -q -p serde_spanned --precise 1.1.2
    fi
    git diff --stat Cargo.lock | tail -1
    cargo build -q --locked --profile bench --all-features --example yc
    cp "$CARGO_TARGET_DIR/release/examples/yc" "$root/bin/$name"
    exe=$(cargo bench -q --locked --all-features --bench metadata_benchmark --no-run --message-format=json \
      | python3 -Ic 'import sys,json
for l in sys.stdin:
    m = json.loads(l)
    if m.get("executable"): print(m["executable"])')
    cp "$exe" "$root/bin/crit-$name"
  )
}
HEAD_SHA=$(git rev-parse a11c4a7)   # a11c4a7, without this scratch commit
mk base 7b87c75 - 0
mk v58 "$HEAD_SHA" 0.0.58 0
mk v59 "$HEAD_SHA" 0.0.59 0
mk v60 "$HEAD_SHA" 0.0.60 0
mk v62 "$HEAD_SHA" 0.0.62 0
mk v63 "$HEAD_SHA" - 0
mk r63 "$HEAD_SHA" - 1
ls -la "$root/bin"

echo "## cachegrind instructions per call"
for c in "small 2000" "block1k 300" "block10k 30" "block32k 10" "doc1m 300"; do
  read -r name n <<<"$c"
  line="$name"
  for v in $VARIANTS; do
    ir() { valgrind --tool=cachegrind --cache-sim=no --cachegrind-out-file=/dev/null "$root/bin/$v" count "$name" "$1" 2>&1 | awk '/I *refs:/{gsub(",","",$NF); print $NF}'; }
    a=$(ir 0); b=$(ir "$n")
    line="$line $v=$(( (b - a) / n ))"
  done
  echo "$line"
done

echo "## interleaved wall time"
sed -i "s#root = .*#root = '$root'#; s#{root}/{v}/target/release/examples/yb#{root}/bin/{v}#" bisect/drive.py
python3 -I bisect/drive.py "${VARIANTS// /,}" "${ROUNDS:-30}" "$root/wall.json"
python3 -I bisect/report.py "$root/wall.json" base

echo "## criterion ABAB: v58 vs v63, gate benches"
export CRITERION_HOME="$root/crit"
f='throughput/(extract_metadata|parse_yaml_block)|^extract_metadata$'
for pass in 1 2; do
  "$root/bin/crit-v58" --bench "$f" --save-baseline "v58_$pass" > /dev/null
  "$root/bin/crit-v63" --bench "$f" --baseline "v58_$pass" 2>&1 | grep -E '^(throughput|extract)|change:' || true
done
echo "## criterion ABAB: base vs v58 (regex/toml only)"
"$root/bin/crit-base" --bench "$f" --save-baseline base1 > /dev/null
"$root/bin/crit-v58" --bench "$f" --baseline base1 2>&1 | grep -E '^(throughput|extract)|change:' || true
