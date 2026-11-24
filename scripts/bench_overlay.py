#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Put this checkout's benchmarks into a checkout of the base commit.

The regression gate compares a baseline run on the base commit with a
run on the change. If each run used its own `benches/`, a pull request
that rewrites a benchmark would compare two different programs and
report the difference as a regression (or a speed-up) of the library.
This script makes the baseline run the change's benchmark code against
the base commit's library, so only the library differs.

It replaces `<base>/benches/` with this checkout's `benches/`, swaps the
`[[bench]]` tables of `<base>/Cargo.toml` for this checkout's, and
prints one bench target name per line. A target that does not build on
the base commit (it uses an API the base does not have) gets no
baseline; the gate then reports its benchmarks as new.

Usage:
    scripts/bench_overlay.py <base-checkout>
"""

from __future__ import annotations

import argparse
import shutil
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BENCH_HEADER = "[[bench]]"


def split_tables(text: str) -> list[list[str]]:
    """The manifest as chunks: the preamble, then one chunk per table."""
    chunks: list[list[str]] = [[]]
    for line in text.splitlines(keepends=True):
        if line.startswith("["):
            chunks.append([])
        chunks[-1].append(line)
    return chunks


def is_bench(chunk: list[str]) -> bool:
    """Whether `chunk` is a `[[bench]]` table."""
    return bool(chunk) and chunk[0].strip() == BENCH_HEADER


def overlay_manifest(base_text: str, head_text: str) -> str:
    """`base_text` with its `[[bench]]` tables replaced by `head_text`'s."""
    kept = [c for c in split_tables(base_text) if not is_bench(c)]
    benches = [c for c in split_tables(head_text) if is_bench(c)]
    out = "".join("".join(c) for c in kept).rstrip("\n") + "\n"
    for chunk in benches:
        out += "\n" + "".join(chunk).rstrip("\n") + "\n"
    return out


def bench_names(manifest_text: str) -> list[str]:
    """The `name` of every `[[bench]]` target in `manifest_text`."""
    return [b["name"] for b in tomllib.loads(manifest_text).get("bench", [])]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("base", type=Path, help="checkout of the base commit")
    args = parser.parse_args()

    head_text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    base_manifest = args.base / "Cargo.toml"
    merged = overlay_manifest(base_manifest.read_text(encoding="utf-8"), head_text)
    tomllib.loads(merged)  # refuse to write a manifest that does not parse
    base_manifest.write_text(merged, encoding="utf-8")

    shutil.rmtree(args.base / "benches", ignore_errors=True)
    shutil.copytree(ROOT / "benches", args.base / "benches")
    print("\n".join(bench_names(head_text)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
