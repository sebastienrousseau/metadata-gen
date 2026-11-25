#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Benchmark regression gate over Criterion's own baseline comparison.

Run the suite twice on the same machine: once on the base commit with
`cargo bench -- --save-baseline base`, then on the change with
`cargo bench -- --baseline base`. The second run writes, for every
benchmark, `target/criterion/<id>/change/estimates.json`: the relative
change in mean time against the baseline, with a confidence interval.

A benchmark regresses when the *lower* bound of that interval is above
the threshold, so the slowdown is larger than the threshold even in the
most favourable reading of the samples. A point estimate alone would
fail on runner noise.

Usage:
    scripts/bench_compare.py [--threshold 0.05] [--criterion-dir DIR] [--advisory]

Writes a Markdown table to stdout (and to `$GITHUB_STEP_SUMMARY` when
set) and exits 1 naming every regressed benchmark. With `--advisory`
it names them as warnings and exits 0: CI passes it once CodSpeed's
instruction counts are the blocking gate, because wall-clock timings
on shared runners move by more than the threshold between runs. A benchmark the
baseline run did not have (its bench target does not build on the base
commit) has no `change/` directory; it is listed as new, not compared.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


def load_changes(criterion_dir: Path) -> list[tuple[str, dict]]:
    """Every `change/estimates.json` under `criterion_dir`, keyed by id."""
    rows = []
    for path in sorted(criterion_dir.glob("**/change/estimates.json")):
        bench_dir = path.parent.parent
        bench_id = bench_dir.relative_to(criterion_dir).as_posix()
        with path.open(encoding="utf-8") as handle:
            rows.append((bench_id, json.load(handle)["mean"]))
    return rows


def load_new(criterion_dir: Path) -> list[str]:
    """Ids measured by the change run that had no baseline to compare."""
    ids = []
    for path in sorted(criterion_dir.glob("**/new/estimates.json")):
        bench_dir = path.parent.parent
        if not (bench_dir / "change").is_dir():
            ids.append(bench_dir.relative_to(criterion_dir).as_posix())
    return ids


def render(
    rows: list[tuple[str, dict]], threshold: float, new: list[str] | None = None
) -> tuple[str, list[str]]:
    """The summary table and the ids whose lower bound breaks `threshold`."""
    lines = [
        f"Regression threshold: {threshold:.0%} (lower bound of the 95% CI)",
        "",
        "| Benchmark | Mean change | 95% CI | Verdict |",
        "| :--- | ---: | :--- | :--- |",
    ]
    regressed = []
    for bench_id, mean in rows:
        point = mean["point_estimate"]
        low = mean["confidence_interval"]["lower_bound"]
        high = mean["confidence_interval"]["upper_bound"]
        verdict = "ok"
        if low > threshold:
            verdict = "REGRESSED"
            regressed.append(f"{bench_id} (+{point:.1%})")
        lines.append(
            f"| `{bench_id}` | {point:+.1%} | {low:+.1%} .. {high:+.1%} | {verdict} |"
        )
    for bench_id in new or []:
        lines.append(f"| `{bench_id}` | n/a | n/a | new, no baseline |")
    return "\n".join(lines) + "\n", regressed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--threshold", type=float, default=0.05)
    parser.add_argument(
        "--criterion-dir", type=Path, default=Path("target/criterion")
    )
    parser.add_argument(
        "--advisory",
        action="store_true",
        help="report regressions as warnings and exit 0",
    )
    args = parser.parse_args()

    rows = load_changes(args.criterion_dir)
    if not rows:
        print("no change/estimates.json found: run with --baseline first")
        return 1
    table, regressed = render(rows, args.threshold, load_new(args.criterion_dir))
    print(table)
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as handle:
            handle.write("## Benchmark comparison\n\n" + table)
    level = "warning" if args.advisory else "error"
    for item in regressed:
        print(f"::{level}::benchmark regressed: {item}")
    return 1 if regressed and not args.advisory else 0


if __name__ == "__main__":
    sys.exit(main())
