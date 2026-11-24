#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Build the WASI component and prove it matches the native crate.

The package must be rustfmt-clean and clippy-clean (warnings denied).
For every file in wasi/cases/<name>.<function>.txt, the component is
called under wasmtime with the file's contents and its output must be
byte-for-byte what the native `expect` binary prints for the same
input. Also checks that the component validates and stays under 1 MiB.

Needs: rustup target wasm32-wasip2, wasm-tools, wasmtime.
Run from anywhere: python3 -I wasi/check.py
"""
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
WASM = HERE / "target/wasm32-wasip2/release/metadata_gen_component.wasm"
MAX_BYTES = 1024 * 1024


def run(cmd):
    return subprocess.run(cmd, cwd=HERE, check=True, capture_output=True, text=True).stdout


def wave(text):
    table = {"\\": "\\\\", '"': '\\"', "'": "\\'", "\n": "\\n", "\r": "\\r", "\t": "\\t"}
    return '"' + "".join(table.get(c, c) for c in text) + '"'


def main():
    run(["cargo", "fmt", "--", "--check"])
    for target in ("wasm32-wasip2", None):
        flags = ["--target", target, "--lib"] if target else ["--all-targets"]
        run(["cargo", "clippy", "--locked", "--release", *flags, "--", "-D", "warnings"])
    run(["cargo", "build", "--locked", "--release", "--target", "wasm32-wasip2", "--lib"])
    run(["cargo", "build", "--locked", "--release", "--bin", "expect"])
    run(["wasm-tools", "validate", str(WASM)])
    size = WASM.stat().st_size
    print(f"component: {size} bytes")
    if size > MAX_BYTES:
        sys.exit(f"component is {size} bytes, over the {MAX_BYTES}-byte budget")
    expect = HERE / "target/release/expect"
    failures = 0
    cases = sorted((HERE / "cases").glob("*.txt"))
    for case in cases:
        func = case.name.split(".")[1]
        text = case.read_text(encoding="utf-8")
        got = run(["wasmtime", "run", "--invoke", f"{func}({wave(text)})", str(WASM)]).strip()
        want = run([str(expect), func, str(case)]).strip()
        status = "ok  " if got == want else "FAIL"
        print(f"{status} {case.name}: {got}")
        if got != want:
            print(f"     native: {want}")
            failures += 1
    if failures or not cases:
        sys.exit(f"{failures} case(s) differ" if failures else "no cases found")
    print(f"{len(cases)} cases: component output equals native output")


if __name__ == "__main__":
    main()
