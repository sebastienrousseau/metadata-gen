#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Per-function complexity gate, measured with rust-code-analysis.

Ceilings come from the repository standard (~/Code/AGENTS.md section 0):

    cyclomatic (McCabe)      <= 10 per function
    cognitive (SonarSource)  <= 15 per function
    Halstead difficulty      <= 30 per function
    lines of code            <= 60 per function, <= 500 per file

`complexity-baseline.txt` lists the offenders that predate the gate, one
per line as `path::function metric=value ...`. A function in the baseline
may not get worse; any other breach fails the run. `--update` rewrites
the baseline from the current measurements and refuses to let it grow,
so the baseline only ever records an improvement.

Usage:
    scripts/complexity_check.py            # gate against the baseline
    scripts/complexity_check.py --update   # record a shrunken baseline
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "complexity-baseline.txt"
SOURCE_DIRS = ["src"]
CEILINGS = {"cyclomatic": 10, "cognitive": 15, "halstead": 30, "sloc": 60}
FILE_LINE_CEILING = 500


def measure() -> list[dict]:
    """Run rust-code-analysis-cli and return one record per function."""
    cmd = ["rust-code-analysis-cli", "-m", "-O", "json"]
    for d in SOURCE_DIRS:
        cmd += ["-p", str(ROOT / d)]
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, check=True)
    except FileNotFoundError:
        sys.exit(
            "rust-code-analysis-cli not found: "
            "cargo install --locked rust-code-analysis-cli"
        )
    records: list[dict] = []
    for line in out.stdout.splitlines():
        if line.strip():
            doc = json.loads(line)
            rel = Path(doc["name"]).resolve().relative_to(ROOT)
            _walk(doc, str(rel), records)
    return records


def _walk(space: dict, path: str, records: list[dict]) -> None:
    for sp in space.get("spaces", []):
        if sp["kind"] == "function":
            m = sp["metrics"]
            records.append(
                {
                    "key": f"{path}::{sp['name']}",
                    "cyclomatic": m["cyclomatic"]["sum"],
                    "cognitive": m["cognitive"]["sum"],
                    "halstead": m["halstead"].get("difficulty") or 0.0,
                    "sloc": m["loc"]["sloc"],
                }
            )
        _walk(sp, path, records)


def breaches(records: list[dict]) -> dict[str, dict[str, float]]:
    """Map each offending function to the metrics it breaches."""
    found: dict[str, dict[str, float]] = {}
    for r in records:
        over = {k: r[k] for k, cap in CEILINGS.items() if r[k] > cap}
        if over:
            found[r["key"]] = over
    return found


def oversized_files() -> list[tuple[str, int]]:
    """Source files longer than the per-file ceiling."""
    found = []
    for d in SOURCE_DIRS:
        for f in sorted((ROOT / d).rglob("*.rs")):
            n = sum(1 for _ in f.open(encoding="utf-8"))
            if n > FILE_LINE_CEILING:
                found.append((str(f.relative_to(ROOT)), n))
    return found


def load_baseline() -> dict[str, dict[str, float]]:
    base: dict[str, dict[str, float]] = {}
    if not BASELINE.exists():
        return base
    for line in BASELINE.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        key, *metrics = line.split()
        base[key] = {
            m.split("=")[0]: float(m.split("=")[1]) for m in metrics
        }
    return base


def write_baseline(found: dict[str, dict[str, float]]) -> None:
    lines = [
        # Split so REUSE's scanner reads the file's own header, not these.
        "# SPDX-FileCopyrightText" + ": 2024 - 2026 metadata-gen contributors",
        "# SPDX-License-Identifier" + ": Apache-2.0 OR MIT",
        "#",
        "# Functions over the complexity ceilings when the gate was added.",
        "# Regenerated only by scripts/complexity_check.py --update, and",
        "# only when the list shrinks. Empty means every function complies.",
    ]
    for key in sorted(found):
        metrics = " ".join(f"{k}={v:g}" for k, v in sorted(found[key].items()))
        lines.append(f"{key} {metrics}")
    BASELINE.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main(argv: list[str]) -> int:
    update = "--update" in argv
    found = breaches(measure())
    base = load_baseline()
    files = oversized_files()
    failures = []
    for key, over in sorted(found.items()):
        known = base.get(key)
        for metric, value in over.items():
            if known is None or value > known.get(metric, 0):
                failures.append(f"{key}: {metric}={value:g} (ceiling {CEILINGS[metric]})")
    for path, n in files:
        failures.append(f"{path}: {n} lines (ceiling {FILE_LINE_CEILING})")
    if update:
        if len(found) > len(base) and base:
            print("refusing to grow the baseline", file=sys.stderr)
            return 1
        write_baseline(found)
        print(f"baseline written: {len(found)} offender(s)")
        return 0
    if failures:
        print("complexity gate failed:")
        for f in failures:
            print(f"  {f}")
        return 1
    print(
        f"complexity gate passed: {len(found)} baselined offender(s), "
        f"no file over {FILE_LINE_CEILING} lines"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
