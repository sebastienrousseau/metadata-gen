<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 0010. Metadata is signed with ML-DSA-65 over a length-prefixed, key-sorted encoding

- **Status:** rejected (the `pqc-sign` feature was withdrawn before release, 2026-11-23: its ML-DSA dependency tree could not pass the repository's strict cargo-vet policy; see #64)
- **Date:** 2026-11-23

## Context

Issue #64 asks for front matter that carries a post-quantum
signature, so a build can prove which metadata it published. A
signature needs a byte string, and `Metadata` is a hash map whose
iteration order is unspecified and differs between the `std`
(`HashMap`) and `no_std` (`BTreeMap`) builds. Signing the raw
front-matter block instead would make the signature depend on
formatting (YAML versus JSON, key order, comments) that the crate
deliberately discards.

Alternatives for the encoding: JSON with sorted keys (needs
`serde_json` in every signing build and leaves number and escape
formatting to that crate's version), CBOR's deterministic profile (a
new dependency for a format the crate does not otherwise read), or a
small fixed binary layout.

Alternatives for the algorithm: Ed25519 (classical only), SLH-DSA
(FIPS 205, signatures of 7,856 bytes or more), or ML-DSA (FIPS 204).

## Decision

- The encoding is the fixed binary layout in
  `docs/spec/canonical-encoding.md` (removed with the feature):
  a magic, an entry count, and entries sorted by key bytes, each field
  length-prefixed with a big-endian `u64`. It needs no dependency and
  is the same in `std` and `no_std` builds.
- The algorithm is ML-DSA-65 from the RustCrypto `ml-dsa` crate, with
  FIPS 204's deterministic signing variant (no RNG needed, works in
  `no_std`) and the context string `metadata-gen/canonical/v1`.
- The code is behind the `pqc-sign` feature, which is not in the
  default set, and lives in `metadata_gen::sign`.
- The public types are aliases of `ml_dsa` types, and `ml_dsa` is
  re-exported, so callers can encode, decode and store keys and
  signatures with that crate's API.

## Consequences

- The signature covers the extracted metadata, not the document's
  bytes. Two documents that flatten to the same map share signatures;
  the body is not covered.
- A breaking release of `ml-dsa` is a breaking change of the
  `pqc-sign` API. The dependency is required as `0.1.2`, so Cargo will
  not move it past `0.1.x` silently.
- Deterministic signing means signing the same metadata twice with the
  same key leaks nothing new, but it gives up the fault-attack
  hardening of the hedged variant. Callers that sign on hardware
  exposed to fault injection should sign the canonical bytes
  themselves with `ml_dsa`'s hedged API.
- Any change to the encoding is a new version with a new magic and
  context string, so old signatures fail closed rather than verifying
  against a different interpretation.
