#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Generate the CycloneDX SBOM the release attaches and check it.

The SBOM must name every crate `cargo tree -e normal` resolves for the
default feature set, at the same version, each with a `pkg:cargo` purl.
Proc-macro crates may appear in the SBOM in addition (they run at build
time). Needs cargo-cyclonedx. Run from the repository root:
python3 -I scripts/check-sbom.py
"""
import json
import pathlib
import subprocess
import sys

NAME = "sbom-check.cdx"


def main():
    subprocess.run(
        ["cargo", "cyclonedx", "--spec-version", "1.5", "--format", "json",
         "--override-filename", NAME],
        check=True,
    )
    path = pathlib.Path(f"{NAME}.json")
    bom = json.loads(path.read_text(encoding="utf-8"))
    path.unlink()
    if bom.get("specVersion") != "1.5":
        sys.exit(f"specVersion is {bom.get('specVersion')}, expected 1.5")
    components = bom.get("components", [])
    in_bom = {(c["name"], c["version"]) for c in components}
    no_purl = [c["name"] for c in components if not c.get("purl", "").startswith("pkg:cargo/")]
    tree = subprocess.run(
        ["cargo", "tree", "-e", "normal", "--prefix", "none", "--no-dedupe", "--locked"],
        check=True, capture_output=True, text=True,
    ).stdout
    root = bom["metadata"]["component"]["name"]
    in_tree = {
        (parts[0], parts[1].lstrip("v"))
        for parts in (line.split() for line in tree.splitlines() if line.strip())
        if parts[0] != root
    }
    missing = sorted(in_tree - in_bom)
    if missing or no_purl:
        sys.exit(f"missing from SBOM: {missing}; without a cargo purl: {no_purl}")
    print(f"SBOM (CycloneDX 1.5) lists all {len(in_tree)} crates of the default build")


if __name__ == "__main__":
    main()
