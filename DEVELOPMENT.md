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
| Rust stable | `rust-version` in `Cargo.toml` (1.89) or later | MSRV, enforced by Cargo and CI |
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
[`GNUmakefile`](GNUmakefile) header for the full list):

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
| Benches against a saved baseline (5 % gate) | `make bench-gate BENCH_SAVE=1`, change, `make bench-gate` |
| `escape_html` against `v_htmlescape` | `make bench-compare` |
| Property tests at CI scale | `make proptest` |
| Kani proofs | `make kani` |
| Mutation testing (85 % kill rate) | `make mutants` |
| Embedded builds and object sizes | `make embedded` |
| Version-bearing files agree | `make versions` |
| Supply chain | `make deny` / `make vet` / `make audit` |
| Complexity ceilings | `make complexity` |
| Links in every Markdown file | `make links` |
| Minimum Rust (1.89) | `make msrv` |
| Public API against the last release | `make semver` |
| Every feature combination | `make hack` |
| Package and verify the archive | `make distcheck` |
| CycloneDX SBOM lists the whole default build | `make sbom` |
| WASI component matches native output | `make wasi` |
| Release preflight for a local tag | `make preflight TAG=vX.Y.Z` |
| Install the tools the gates need | `make tools` |

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
| `cargo-vet` (quality.yml) | `make vet` | the exemption count must not exceed `supply-chain/exemptions-baseline.txt`; see `docs/SUPPLY-CHAIN.md` |
| `sbom` (quality.yml) | `make sbom` | needs `cargo-cyclonedx` 0.5.9 |
| `wasi-component` (quality.yml) | `make wasi` | needs the `wasm32-wasip2` target, `wasm-tools` and `wasmtime` |
| `proptest` (quality.yml) | `make proptest` | 10 000 cases per round trip, 100 000 for the ASCII involution; the seed is printed, rerun with `PROPTEST_RNG_SEED=<seed>` |
| `embedded` (quality.yml) | `make embedded` | `riscv32imc` has no atomic read-modify-write, so `noyalib` and `toml` (which use `Arc`) cannot build there; that row builds JSON only |
| `bench regression` (bench.yml) | `make bench-gate` | PRs only; fails when a benchmark's 95 % interval lies wholly above +5 % against the base commit |
| `codspeed` (bench.yml) | none | skipped until the `CODSPEED_TOKEN` secret exists |
| `kani` (assurance.yml) | `make kani` | needs `cargo install --locked kani-verifier && cargo kani setup` |
| `mutants (changed lines)` (assurance.yml) | `git diff main... -- src > d.diff && cargo mutants --in-diff d.diff` | PRs only; same 85 % floor |
| `miri-full`, `fuzz-long`, `mutants-full` (assurance.yml) | `MIRIFLAGS=-Zmiri-strict-provenance cargo +nightly miri test --all-features --lib --tests`, `cargo +nightly fuzz run <target> -- -max_total_time=600`, `make mutants` | nightly schedule and manual dispatch only |
| `release-hygiene` (quality.yml) | `make versions && make examples && make bench-smoke && make doc` | |
| `docs-lint` | `make lint` | British spellings are house style; see `.codespellrc` |
| `reuse-lint` | `uvx --with chardet reuse lint` | files without an inline header are covered by `REUSE.toml` |
| `docs-strict` | `make doc` | warnings are errors; every public item is documented |
| `msrv` (quality.yml) | `make msrv` | needs `rustup toolchain install 1.89.0` |
| `semver-checks` (quality.yml) | `make semver` | builds the crates.io baseline, so it needs the network |
| `feature-powerset` (quality.yml) | `make hack` | the crate has no features yet, so this is one build |
| `complexity` (quality.yml) | `make complexity` | ceilings in `scripts/complexity_check.py`; `complexity-baseline.txt` may only shrink |
| `links` (quality.yml) | `make links` | `lychee.toml` excludes loopback and `example.com` fixtures |
| `scorecard.yml` | none | runs on `main`; the score is read at <https://scorecard.dev> |
| `release.yml` | `make preflight TAG=vX.Y.Z` | only the local preflight can verify the tag signature |

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

### Release automation

A release is a signed, annotated tag `vX.Y.Z` whose subject line is
`metadata-gen vX.Y.Z`, pushed only after `make preflight TAG=vX.Y.Z`
passes. `release.yml` then verifies the tag and versions again, packages
the crate, writes CycloneDX SBOMs (JSON and XML), builds and checks the
WASI component, attests build provenance for every asset,
and publishes the GitHub release with the Highlights from
`docs/releases/vX.Y.Z.md`, the generated What's Changed list, the asset
checksums and the Full Changelog line. Publishing to crates.io stays a
manual `cargo publish` after the published release has been audited.

### Devcontainer

The container runs only `rustup component add rustfmt clippy` on create
so it boots in well under a minute; `make tools` installs the cargo
tools the other gates need.

## House rules

- CI must be green in the same session that turned it red.
- Commits are signed; releases are signed tags.
- Structure cleanups never couple to code changes.
- New behaviour lands with its test in the same commit; a regression
  fix lands with the input that found it.
