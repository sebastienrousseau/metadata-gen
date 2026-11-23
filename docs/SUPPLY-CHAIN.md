<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Supply chain

What `metadata-gen` depends on, how each dependency is vetted, and how
to verify what a release ships. The checks named here run in CI on
every push; a release cannot be tagged while one fails.

## First-party dependencies

Two runtime dependencies share a maintainer with this crate:

| Crate | Feature | Role | Requirement |
| :--- | :--- | :--- | :--- |
| `noyalib` | `yaml` | YAML 1.2.2 parser ([ADR-0008](adr/0008-noyalib-for-yaml.md)) | `=0.0.58` |
| `oxml` | `html` | `<meta>` attribute and entity parsing ([ADR-0006](adr/0006-meta-extraction-on-oxml.md)) | `=0.0.13` |

This is disclosed so you can weigh it: the same person maintains the
parser that reads your front matter and the crate that calls it. It
gives this project control over the most exposed code path (both
crates are `#![forbid(unsafe_code)]`), and it concentrates the risk in
one maintainer and one set of release credentials.

Both are pinned exactly while they are on `0.0.x`
([ADR-0004](adr/0004-first-party-exact-pins.md)), so a new upstream
release reaches you only through a `metadata-gen` release.
`scripts/verify-release-versions.sh` fails if either requirement loses
its `=` or disagrees with `Cargo.lock`. The policy for contributors is
in [`CONTRIBUTING.md`](../CONTRIBUTING.md#first-party-dependency-policy).
A build with `default-features = false` and only the formats you parse
leaves out whichever of the two you do not need.

## Advisories and licences

- `cargo deny check` (licences, advisories, sources, bans) and
  `cargo audit` run in CI. Accepted advisories must be listed with a
  reason in both `deny.toml` and `audit.toml`; today both lists are
  empty.
- A dependency with a published advisory is constrained in
  `Cargo.toml` or `Cargo.lock` so no affected version resolves.

## cargo-vet

[`cargo-vet`](https://mozilla.github.io/cargo-vet/) checks that every
crate in the graph has been reviewed by someone this project relies
on. The `cargo-vet` job runs `cargo vet --locked`, so it uses the
audit data recorded in `supply-chain/imports.lock` and does not depend
on the network.

A crate version passes when one of these covers it:

1. **An imported audit.** `supply-chain/config.toml` imports the
   published audit sets of Mozilla, Google, the Bytecode Alliance,
   ISRG, Zcash, Embark Studios, Fermyon, Ariel OS and Firefox, using
   the URLs the [cargo-vet registry](https://github.com/mozilla/cargo-vet/blob/main/registry.toml)
   lists. Refresh them with `cargo vet regenerate imports`.
2. **A trusted publisher.** The `[[trusted.*]]` entries in
   `supply-chain/audits.toml` trust a named crates.io publisher for a
   named crate and date range. New entries are a maintainer decision.
3. **An audit recorded here.** A delta or full audit in
   `supply-chain/audits.toml`, recorded with `cargo vet certify` by the
   person who did the review, under their own name.
4. **An exemption.** `supply-chain/config.toml` holds the few crates
   not yet covered. The `Exemption ratchet` step fails if the count
   grows past `supply-chain/exemptions-baseline.txt`, and the baseline
   is only ever lowered.

When a dependency change makes `cargo vet` fail:

1. Run `cargo vet regenerate imports`; a published audit may already
   cover it.
2. Run `cargo vet suggest` to see what is left and how large each diff
   is.
3. Review what is left (`cargo vet diff <crate> <from> <to>` or
   `cargo vet inspect <crate> <version>`): `unsafe`, `build.rs`,
   proc-macros, file, network and process access, and anything that
   looks obfuscated. Record it with `cargo vet certify`. If the diff is
   too large to read honestly, say so in the pull request instead of
   certifying it.
4. Never add an exemption or raise the baseline to make the check
   pass, and never record an audit in someone else's name.

## SBOM and provenance

Every tagged release attaches:

- `metadata-gen-<version>.crate`, the packaged crate;
- `metadata-gen-<version>.cdx.json` and `.cdx.xml`, CycloneDX 1.5
  SBOMs of the default feature set, each component with its
  `pkg:cargo` purl;
- `metadata-gen-<version>.wasm`, the WASI 0.2 component
  ([`wasi/README.md`](../wasi/README.md));
- a SHA-256 list in the release notes.

Each asset carries a GitHub build-provenance attestation, signed
through Sigstore by the release workflow's OIDC identity. Verify a
download with the GitHub CLI:

```bash
gh attestation verify metadata-gen-0.0.9.cdx.json \
  --repo sebastienrousseau/metadata-gen
```

The `CycloneDX SBOM` CI job regenerates the SBOM on every push and
fails unless it lists every crate `cargo tree -e normal` resolves for
the default build (`scripts/check-sbom.py`, or `make sbom`).
Assets attached before 0.0.10 carry a JSON SBOM of `--all-features`
in CycloneDX 1.3 only.

## The WASI component's dependencies

`wasi/` is a separate package with its own `Cargo.lock`. Its extra
dependency is `wit-bindgen` and that crate's tree, which is outside the
library's `cargo-vet` graph ([ADR-0009](adr/0009-wasi-component-shape.md)).
Users of the Rust crate do not compile it.
