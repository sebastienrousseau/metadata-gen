#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Mutation kill-rate gate over a cargo-mutants run.

Reads `mutants.out/outcomes.json` and computes

    kill rate = (caught + timeout) / (caught + timeout + missed)

Unviable mutants (ones that do not compile) say nothing about the tests
and are left out. A timeout counts as killed: the mutant changed
behaviour enough to hang the suite, which the suite noticed.

Usage:
    scripts/mutants_gate.py [--min 0.85] [--outcomes PATH]

Prints the counts and every missed mutant, appends them to
`$GITHUB_STEP_SUMMARY` when set, and exits 1 below the floor.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections import Counter
from pathlib import Path


def tally(outcomes_path: Path) -> tuple[Counter, list[str]]:
    """Counts per summary kind, and the names of the missed mutants."""
    with outcomes_path.open(encoding="utf-8") as handle:
        data = json.load(handle)
    counts: Counter = Counter()
    missed = []
    for outcome in data["outcomes"]:
        scenario = outcome["scenario"]
        if scenario == "Baseline":
            continue
        summary = outcome["summary"]
        counts[summary] += 1
        if summary == "MissedMutant":
            missed.append(scenario["Mutant"]["name"])
    return counts, sorted(missed)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--min", type=float, default=0.85)
    parser.add_argument(
        "--allow-empty",
        action="store_true",
        help="pass when no viable mutant was generated (a PR diff)",
    )
    parser.add_argument(
        "--outcomes", type=Path, default=Path("mutants.out/outcomes.json")
    )
    args = parser.parse_args()

    counts, missed = tally(args.outcomes)
    killed = counts["CaughtMutant"] + counts["Timeout"]
    viable = killed + counts["MissedMutant"]
    if viable == 0:
        print("no viable mutants were tested")
        return 0 if args.allow_empty else 1
    rate = killed / viable
    report = [
        f"Kill rate: {rate:.1%} ({killed}/{viable}; floor {args.min:.0%})",
        f"caught {counts['CaughtMutant']}, timeout {counts['Timeout']}, "
        f"missed {counts['MissedMutant']}, unviable {counts['Unviable']}",
    ]
    report += [f"- missed: `{name}`" for name in missed]
    text = "\n".join(report) + "\n"
    print(text)
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as handle:
            handle.write("## Mutation testing\n\n" + text)
    if rate < args.min:
        print(f"::error::kill rate {rate:.1%} is below {args.min:.0%}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
