<!-- SPDX-FileCopyrightText: 2026 metadata-gen contributors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# metadata-gen — Engineering Policies

This document is the single source of truth for metadata-gen's
engineering posture: MSRV, SemVer, security audits, performance
guarantees, concurrency, platform support, and panic policy. Every
README in the workspace links here; if there is a contradiction
between this file and the README, this file wins.

---

## Contents

1. [MSRV (Minimum Supported Rust Version)](#1-msrv-minimum-supported-rust-version)
2. [SemVer & API stability](#2-semver--api-stability)
3. [Security & audits](#3-security--audits)
4. [Performance & algorithmic complexity](#4-performance--algorithmic-complexity)
5. [Concurrency guarantees](#5-concurrency-guarantees)
6. [Platform support](#6-platform-support)
7. [Panic policy](#7-panic-policy)
8. [Error model](#8-error-model)
9. [Dependency policy](#9-dependency-policy)
10. [Release & changelog policy](#10-release--changelog-policy)

---

## 1. MSRV (Minimum Supported Rust Version)

| Crate | MSRV | Rationale |
|---|---|---|
| `metadata-gen` (library) | **1.88.0** | The floor is pulled by the `time` dependency (date normalisation), whose current releases declare `rust-version = "1.88.0"`. Lowering it would mean pinning an older `time` carrying a stack-exhaustion advisory. Enforced by Cargo's `rust-version` field and verified in CI across Linux, macOS, and Windows. |

**When we bump it.** Only when the toolchain we build and test at
moves, such as a dependency raising its floor, never speculatively.
An MSRV bump ships as a patch in the `0.0.x` line, documented under
an explicit heading in `CHANGELOG.md`.

---

## 2. SemVer & API stability

`metadata-gen` follows [SemVer 2.0.0](https://semver.org/), with the
pre-1.0 rule that the patch position increments strictly by `+0.0.1`
(e.g., `0.0.7` -> `0.0.8`).

### Output stability guarantee

What the crate produces is part of the API contract: a change to how
a document flattens, which key a value lands under, or what a meta-tag
group renders is treated as a breaking change even when no Rust signature
moves.

---

## 3. Security & audits

Reference: [`SECURITY.md`](../SECURITY.md).

- `#![forbid(unsafe_code)]` at the crate root. The compiler proves
  the absence of unsafe code across all library modules.
- No C dependencies, no FFI, no network I/O, no environment reads.
  The only filesystem access is the explicit async helper
  `async_extract_metadata_from_file`, which reads the path supplied
  by the caller.
- HTML entity escaping prevents markup injection in generated meta tags.
- Verified in CI by `cargo audit`, `cargo deny`, `cargo vet`, Miri,
  and libFuzzer fuzzing targets.

---

## 4. Performance & algorithmic complexity

- Delimiter detection is a hand-written scanner: one pass over the
  opening line and one `memchr::memmem` search for the closing fence,
  with no regex and no per-call compilation.
- HTML escaping runs in a single left-to-right pass with a single
  allocation.
- HTML unescaping uses a single-pass scanner that decodes each entity
  once without rescanning output text.
- Meta tag extraction streams HTML events with `quick-xml` without
  allocating a full DOM tree.

---

## 5. Concurrency guarantees

- `Metadata`, `MetaTag`, `MetaTagGroups`, and `ProcessOptions` are
  fully owned types implementing `Send` and `Sync`.
- All public parse and extraction functions are thread-safe and
  reentrant with no shared global state.

---

## 6. Platform support

### Tier 1 (CI-verified on every push)

| Target | Host | Toolchain |
|---|---|---|
| `aarch64-apple-darwin` | macOS | stable |
| `x86_64-unknown-linux-gnu` | Linux | stable |
| `x86_64-pc-windows-msvc` | Windows | stable |

---

## 7. Panic policy

The library API does not panic on well-formed or malformed input.
All errors return `Result<T, MetadataError>`.

---

## 8. Error model

`MetadataError` encompasses all failure classes, with conversions
from parser, I/O, and UTF-8 errors. `MetadataError::context` prefixes
context while preserving the underlying failure cause.

---

## 9. Dependency policy

- The first-party dependency `noyalib` is pinned exactly until 1.0
  (ADR-0004). `dtt` was dropped in 0.0.8 for `time`, which builds
  without `std` (ADR-0005).
- Dependencies are vetted with `cargo-vet` and recorded in
  `supply-chain/`.

---

## 10. Release & changelog policy

- `CHANGELOG.md` follows Keep a Changelog.
- Every release increments strictly by `+0.0.1`.
- Version-bearing files are verified by `scripts/verify-release-versions.sh`.
