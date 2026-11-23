<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 0008. YAML front matter is parsed by `noyalib`

- **Status:** accepted
- **Date:** 2026-04-18 (recorded 2026-11-23)

## Context

Up to 0.0.2 YAML went through a hand-rolled `serde_yml` shim kept
inside the repository, and later `yaml-rust2` sat in the graph as well.
The widely used `serde_yaml` is archived, and its forks
(`serde_yml`, `serde_yaml_ng`) are third-party crates whose
maintenance and safety posture this project does not control.
Front matter is untrusted input in most pipelines (user-submitted
posts, pulled repositories), so the YAML parser is the crate's main
attack surface.

## Decision

YAML is parsed by `noyalib`, a first-party YAML 1.2.2 parser from the
same maintainer that is `#![forbid(unsafe_code)]` and exposes
resource budgets (`ParserConfig`). The migration landed in 0.0.3, and
`yaml-rust2` was dropped in #84. `noyalib` is:

- optional, behind the `yaml` feature ([ADR-0005](0005-feature-flags-and-no-std.md));
- pinned exactly ([ADR-0004](0004-first-party-exact-pins.md));
- run with its strict budget preset on the flat and typed paths alike,
  inside the crate's own `ParseLimits`, since RUSTSEC-2026-0333
  showed that a budget enforced on one path only is not a budget.

## Consequences

- One YAML implementation, audited by the same people who maintain
  this crate, and no `unsafe` anywhere in the YAML path.
- A fix in `noyalib` reaches users only through a `metadata-gen`
  release that moves the pin; RUSTSEC-2026-0333 was handled that way
  in 0.0.8.
- The shared author is a concentration of risk as well as of control.
  `docs/SUPPLY-CHAIN.md` discloses it so users can weigh it.
