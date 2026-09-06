# 0004. First-party crates are pinned exactly (`=0.0.X`)

- **Status:** accepted
- **Date:** 2026-06-28

## Context

`noyalib` and `dtt` are 0.0.x crates from the same maintainer. Under
Cargo's SemVer rules a 0.0.x release may break, and a caret requirement
on `0.0.8` would silently accept `0.0.9`. Consumers were breaking on
releases they never chose to take.

## Decision

`noyalib` and `dtt` are required as `=0.0.X`. A bump is a deliberate,
tested change to this crate, released with a changelog entry.

## Consequences

- No surprise upgrades in consumers; `Cargo.lock` and the pin move
  together (checked by `scripts/verify-release-versions.sh`).
- Every noyalib or dtt release needs a metadata-gen release to reach
  consumers. The family accepts that cost; it is the same model noyalib's
  own satellites use.
