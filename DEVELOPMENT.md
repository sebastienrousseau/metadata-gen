<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Developing metadata-gen

The single entry point for working on this repository. User-facing
documentation lives in the [README](README.md) and
[`docs/`](docs/); contribution etiquette and review expectations live in
[`CONTRIBUTING.md`](CONTRIBUTING.md). This file is the *how*:
toolchain, tasks, and reproducing every CI gate locally.

## Toolchain

| What | Version | Why |
| :--- | :--- | :--- |
| Rust stable | `rust-version` in `Cargo.toml` (1.88) or later | MSRV, enforced by Cargo and CI |
| Rust nightly | any recent | Miri, cargo-fuzz, coverage (`cargo-llvm-cov`) |
| cargo-deny, cargo-vet, cargo-audit | latest | supply-chain gates |
| cargo-llvm-cov | latest | coverage gate |
| cargo-fuzz | latest, **installed from source** (`cargo install --locked cargo-fuzz`) | the prebuilt musl binary infers its own build triple as the fuzz target and dies on "sanitizer is incompatible with statically linked libc" |
| uv (`uvx`) and npx | any | `reuse`, `codespell`, `markdownlint` for the docs lint |

```bash
git clone https://github.com/sebastienrousseau/metadata-gen
cd metadata-gen
make            # check + clippy + test — the default gate
```

## Task map

`make` targets are the canonical dev tasks (see the
[`Makefile`](Makefile) header for the full list):

| Task | Command |
| :--- | :--- |
| Everything a PR needs first | `make` |
| Full test suite | `make test` |
| Lints / formatting | `make clippy` / `make fmt` |
| Docs lint (markdownlint, codespell, REUSE) | `make lint` |
| Docs as CI builds them | `make doc` |
| Coverage gate | `make coverage` |
| Miri | `make miri` |
| Fuzz targets, corpus replay | `make fuzz` |
| All examples | `make examples` |
| Benches compile and run once | `make bench-smoke` |
| Version-bearing files agree | `make versions` |
| Supply chain | `make deny` / `make vet` / `make audit` |

## Reproducing the CI gates

CI has two workflows. [`ci.yml`](.github/workflows/ci.yml) calls the
shared pipelines from
[`sebastienrousseau/pipelines`](https://github.com/sebastienrousseau/pipelines)
plus cargo-deny and cargo-audit;
[`quality.yml`](.github/workflows/quality.yml) holds the gates the
repository standard requires that the shared pipeline does not cover.
Every gate has a local equivalent:

| CI job | Local reproduction | Gotcha |
| :--- | :--- | :--- |
| `ci` (rust-ci: fmt, clippy, test, coverage, cross-platform) | `make` then `make coverage` | coverage needs nightly; threshold is 98 % lines |
| `security` (pipelines security.yml) | `make audit` | fails on any advisory, ignores are in `audit.toml` and must mirror `deny.toml` |
| `cargo-deny` | `make deny` | licences, advisories, sources |
| `cargo-audit` | `make audit` | if a local cargo alias named `audit` shadows the subcommand, run `cargo-audit audit` |
| `coverage-gate` (quality.yml) | `make coverage` | needs nightly; threshold is 98 % lines |
| `miri` (quality.yml) | `make miri` | filesystem tests are skipped by `#[cfg_attr(miri, ignore)]` |
| `fuzz-replay` (quality.yml) | `make fuzz` | builds every target, replays corpus + regressions with `-runs=0` |
| `cargo-vet` (quality.yml) | `make vet` | the exemption count must not exceed `supply-chain/exemptions-baseline.txt` |
| `release-hygiene` (quality.yml) | `make versions && make examples && make bench-smoke && make doc` | |
| `docs-lint` | `make lint` | British spellings are house style; see `.codespellrc` |
| `reuse-lint` | `uvx --with chardet reuse lint` | files without an inline header are covered by `REUSE.toml` |
| `docs-strict` | `make doc` | warnings are errors; every public item is documented |

## Coverage: the threshold and why

The gate is **98 % lines** (`cargo llvm-cov --all-features
--fail-under-lines 98`). Test code inside `src/` counts, and the lines
that stay uncovered are the `_ => panic!` arms of existing tests, which
are unreachable by construction; they are a fraction of a percent. A
higher number would only be reachable by deleting those arms, which
would make the tests worse.

## Test layout

- `src/**` `#[cfg(test)]` — unit tests next to the code, including
  every `MetadataError::context` arm and every flattener leaf kind.
- `tests/test_<module>.rs` — one integration suite per module;
  `tests/test_integration.rs` drives the whole pipeline.
- `examples/` — one runnable example per module, run in CI.
- `benches/` — Criterion harness, smoke-run in CI.
- `fuzz/fuzz_targets/` — `fuzz_extract_metadata`, `fuzz_extract_meta_tags`,
  `fuzz_html_escape`; `fuzz/corpus/<target>` is the committed seed set and
  `fuzz/regressions/<target>` holds every fixed-bug reproducer. Both
  replay per push. A crash found by fuzzing lands as a regression input
  in the same commit as its fix.

## Release model

Versions increment strictly by `+0.0.1`. Before tagging, run
`make versions`: it checks `Cargo.toml`, `Cargo.lock`, the `noyalib`
pin, `CITATION.cff`, the `CHANGELOG.md` heading and every install
snippet. Tags are signed (`git tag -s vX.Y.Z`); the key is in
[`KEYS.asc`](KEYS.asc). Publishing to crates.io is manual today
(`cargo publish` from the tagged commit); moving it to a tag-triggered
workflow with Trusted Publishing is tracked in the changelog's next
release.

## House rules

- CI must be green in the same session that turned it red.
- Commits are signed; releases are signed tags.
- Structure cleanups never couple to code changes.
- New behaviour lands with its test in the same commit; a regression
  fix lands with the input that found it.
