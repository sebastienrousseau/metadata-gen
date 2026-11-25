#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Instruction-count regression gate (#49).

Runs each workload of the `instructions` bench binary under Valgrind's
callgrind, once with the base commit's build and once with the change's,
and compares the instructions executed (callgrind's `summary` count).
Unlike wall-clock time on a shared runner, the count does not depend on
what else the machine is doing, so a small threshold holds.

A workload regresses when the change executes more than `threshold`
more instructions than the base. A workload the base binary does not
know (it exits with status 2) is listed as new, not compared.

Usage:
    scripts/instruction_compare.py --head BIN [--base BIN]
        [--threshold 0.01] [--valgrind valgrind]

Writes a Markdown table to stdout and exits 1 naming every regressed
workload with an `::error::` line.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

ABSENT = 2  # the bench binary's exit status for an unknown workload


def workloads(binary: Path) -> list[str]:
    """The workload names `binary --list` prints."""
    out = subprocess.run(
        [str(binary), "--list"], check=True, capture_output=True, text=True
    )
    return [line for line in out.stdout.splitlines() if line]


def summary(callgrind_out: Path) -> int:
    """The total instruction count from a callgrind output file."""
    for line in callgrind_out.read_text(encoding="utf-8").splitlines():
        if line.startswith("summary:"):
            return int(line.split()[1])
    raise ValueError(f"no summary line in {callgrind_out}")


def count(valgrind: str, binary: Path, workload: str) -> int | None:
    """Instructions `binary workload` executes, or None if it lacks it."""
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "callgrind.out"
        run = subprocess.run(
            [
                valgrind,
                "--tool=callgrind",
                f"--callgrind-out-file={out}",
                str(binary),
                workload,
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if run.returncode == ABSENT:
            return None
        if run.returncode != 0:
            sys.stderr.write(run.stderr)
            raise RuntimeError(f"{binary} {workload} exited {run.returncode}")
        return summary(out)


def row(name: str, base: int | None, head: int, threshold: float) -> tuple[str, bool]:
    """One table row, and whether it is a regression."""
    if base is None:
        return f"| `{name}` | n/a | {head:,} | n/a | new, no baseline |", False
    change = (head - base) / base
    regressed = change > threshold
    verdict = "REGRESSED" if regressed else "ok"
    line = f"| `{name}` | {base:,} | {head:,} | {change:+.2%} | {verdict} |"
    return line, regressed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--head", type=Path, required=True)
    parser.add_argument("--base", type=Path)
    parser.add_argument("--threshold", type=float, default=0.01)
    parser.add_argument("--valgrind", default="valgrind")
    args = parser.parse_args()

    lines = [
        f"Instruction-count threshold: {args.threshold:.0%} (callgrind `Ir`)",
        "",
        "| Workload | Base | Change | Change % | Verdict |",
        "| :--- | ---: | ---: | ---: | :--- |",
    ]
    regressed = []
    for name in workloads(args.head):
        base = count(args.valgrind, args.base, name) if args.base else None
        head = count(args.valgrind, args.head, name)
        if head is None:
            raise RuntimeError(f"the change's binary lacks its own workload {name}")
        line, bad = row(name, base, head, args.threshold)
        lines.append(line)
        if bad:
            regressed.append(f"{name} ({(head - base) / base:+.2%})")
    print("\n".join(lines))
    for item in regressed:
        print(f"::error::instruction count regressed: {item}")
    return 1 if regressed else 0


if __name__ == "__main__":
    sys.exit(main())
