#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Runs the Kani round-trip proofs in tests/kani/escape_html.rs (#55).
#
# One global unwind bound does not fit them: the entity table scan in
# `unescape_html` needs 9 iterations and `memcmp` 7, but raising every
# loop to 9 makes CBMC run past 20 minutes. So each of those loops gets
# its own bound and every other loop gets 3. CBMC names a loop by its
# mangled function name, which changes with the crate version, so the
# names are read from the compiled goto program rather than written
# down. Unwinding assertions stay on: a bound that is too small fails
# the proof instead of weakening it.
#
# Usage: scripts/kani-round-trip.sh
# Needs: cargo-kani (cargo install kani-verifier && cargo kani setup)
set -euo pipefail

cd "$(dirname "$0")/.."

harnesses=(
  check_round_trip_plain
  check_round_trip_amp
  check_round_trip_angle
  check_round_trip_quote
)

version="$(cargo kani --version | awk '/Kani Rust Verifier/ { print $4 }')"
goto_instrument="${HOME}/.kani/kani-${version}/bin/goto-instrument"

cargo kani --tests --harness "${harnesses[0]}" --only-codegen
goto="$(find target/kani -name "*${harnesses[0]}.out" | head -n 1)"
loops="$("${goto_instrument}" --show-loops "${goto}" | sed -n 's/^Loop \(.*\):$/\1/p')"

loop_id() {
  local id
  id="$(grep -E "$1" <<<"${loops}" | head -n 1)"
  if [[ -z "${id}" ]]; then
    echo "kani-round-trip: no loop matches $1" >&2
    exit 1
  fi
  printf '%s' "${id}"
}

entities="$(loop_id '4find.*13unescape_html0E')"
naive="$(loop_id '6memchr12memchr_naive')"

args=()
for harness in "${harnesses[@]}"; do
  args+=(--harness "${harness}")
done

cargo kani --tests "${args[@]}" -Z unstable-options \
  --cbmc-args --unwind 3 \
  --unwindset "${entities}:9,memcmp.0:7,${naive}:7"
