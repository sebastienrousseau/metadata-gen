# Security Policy

## Supported Versions

| Version | Supported |
|:--------|:---------:|
| 0.0.x   | Yes       |

## Reporting a Vulnerability

Report security vulnerabilities by emailing **sebastian.rousseau@gmail.com**.

Do not open a public issue for security reports.

Include:

- A description of the vulnerability.
- Steps to reproduce.
- Affected versions.
- Any suggested fix (optional).

Expect an initial response within 48 hours. A fix or mitigation plan will follow within 7 days of confirmation.

## Security Design

metadata-gen enforces safety at the compiler level:

- `#![forbid(unsafe_code)]` — zero unsafe blocks, guaranteed.
- No C dependencies, no FFI calls. Pure Rust only.
- No network I/O and no environment variable reads. The only file
  system access is the explicit `async_extract_metadata_from_file`
  helper, which reads the path its caller names and nothing else.

### Input hardening

Front matter is parsed by `noyalib` (YAML), `toml` and `serde_json`,
each with its own bounds on nesting and size; metadata-gen adds no
recursion of its own beyond flattening the parsed tree, whose depth is
bounded by those parsers. `<meta>` extraction is a single streaming
pass over the input with `oxml`, and malformed markup ends the
scan rather than failing it: the tags found so far are returned.

There are no configurable resource limits in this crate today. Callers
handling untrusted input of unbounded size should cap it before calling
`extract_metadata`; this is documented rather than silently assumed.

### Fuzzing

`fuzz/` holds libFuzzer targets for `extract_metadata`,
`extract_meta_tags` and the HTML escape pair. A committed seed corpus
and every fixed-bug reproducer are replayed on each push, and a
nightly job fuzzes each target for ten minutes, carrying the grown
corpus from one run to the next. To fuzz locally (nightly Rust and
`cargo install --locked cargo-fuzz`):

```sh
cd fuzz
cargo +nightly fuzz run fuzz_extract_metadata -- -max_total_time=3600
```

A crash found by fuzzing is fixed, with its input committed under
`fuzz/regressions/<target>/` as a regression, within 14 days of being
found.

### Verification

- Kani proves, for every input within a stated bound, that
  `escape_html` never panics and leaves no raw `<`, `>`, `"` or `'`,
  and that `unescape_html` inverts it on ASCII
  ([`tests/kani/escape_html.rs`](tests/kani/escape_html.rs)).
- Miri runs the unit tests on every push and every test target with
  `-Zmiri-strict-provenance` nightly.
- Mutation testing (`cargo-mutants`) holds an 85 % kill-rate floor:
  on the lines a pull request changes, and nightly on the whole crate.

### Supply Chain

- Runtime deps audited in CI: `cargo-deny` (licences, advisories,
  sources) and `cargo-audit`.
- Dependency provenance recorded with `cargo-vet`
  (`supply-chain/`); exemptions are regenerated on every dependency
  change, never added by hand.
- The first-party crate `noyalib` is pinned exactly (`=0.0.X`); a
  bump is a deliberate release of this crate (ADR-0004).
- Front matter is untrusted input: every parse runs under
  `ParseLimits` (block size, nesting depth), and the YAML parser gets
  its strict resource budgets on the flat and typed paths alike.
- `Cargo.lock` committed for deterministic builds; CI builds
  `--locked`.
- All GitHub Actions SHA-pinned.
- REUSE/SPDX compliance linted in CI.

### Commit Integrity

All commits on the main branch are signed, and releases are signed
tags. The release-signing key is published in [`KEYS.asc`](KEYS.asc):

```text
4B7F16C909C7A8EE9BED338A4F047EDF5F90F638
```

Signing key `Sebastien Rousseau <sebastian.rousseau@gmail.com>`,
ed25519, signing-only, expires 2028-08-16. Verify the fingerprint out
of band before trusting it.
