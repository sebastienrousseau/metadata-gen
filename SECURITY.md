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
pass over the input with `quick-xml`, and malformed markup ends the
scan rather than failing it: the tags found so far are returned.

There are no configurable resource limits in this crate today. Callers
handling untrusted input of unbounded size should cap it before calling
`extract_metadata`; this is documented rather than silently assumed.

### Fuzzing

`fuzz/` holds libFuzzer targets for `extract_metadata`,
`extract_meta_tags` and the HTML escape pair. A committed seed corpus
and every fixed-bug reproducer are replayed on each push; see
[`DEVELOPMENT.md`](DEVELOPMENT.md) for how to run the targets locally.

### Supply Chain

- Runtime deps audited in CI: `cargo-deny` (licences, advisories,
  sources) and `cargo-audit`.
- Dependency provenance recorded with `cargo-vet`
  (`supply-chain/`); exemptions are regenerated on every dependency
  change, never added by hand.
- The first-party crates `noyalib` and `dtt` are pinned exactly
  (`=0.0.X`); a bump is a deliberate release of this crate (ADR-0004).
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
