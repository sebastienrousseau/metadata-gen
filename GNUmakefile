# GNUmakefile for metadata-gen: the Unix task contract (all, check, test,
# clean, dist, distcheck) plus every CI gate as a local target.
# Works on macOS, Linux, and WSL without modification.
#
# Usage:
#   make           — run check + clippy + test (default)
#   make test      — run all tests (all features)
#   make clippy    — run clippy lints, warnings denied
#   make fmt       — check formatting
#   make lint      — docs lint: markdownlint + codespell + REUSE
#   make deny      — cargo-deny supply-chain checks
#   make vet       — cargo-vet provenance check (--locked)
#   make audit     — cargo-audit advisory check
#   make doc       — build documentation with warnings denied
#   make coverage  — line/function coverage via cargo-llvm-cov (nightly)
#   make miri      — run the lib test suite under Miri (nightly)
#   make fuzz      — build every fuzz target and replay the corpus (nightly)
#   make examples  — run every example to completion
#   make bench-smoke — compile and run each bench once, no measurement
#   make bench-gate — benches against a saved baseline, 5% regression gate
#   make bench-compare — escape_html against v_htmlescape (separate workspace)
#   make proptest  — property tests at CI scale (10 000 cases, seed printed)
#   make kani      — the escape_html totality proof (needs cargo-kani)
#   make mutants   — mutation testing with the 85% kill-rate gate
#   make embedded  — no_std builds and object sizes on the embedded targets
#   make versions  — every version-bearing file agrees with Cargo.toml
#   make sbom      — CycloneDX software bill of materials (cargo-cyclonedx)
#   make complexity — per-function complexity gate against the baseline
#   make links     — link check every Markdown file (lychee)
#   make msrv      — the crate builds on the declared minimum Rust
#   make semver    — public API against the last crates.io release
#   make hack      — every feature combination compiles
#   make no-std    — no_std + alloc build on a bare-metal target
#   make dist      — package the crate as crates.io would receive it
#   make distcheck — package, then verify the archive's contents
#   make preflight TAG=vX.Y.Z — blocking release preflight for a local tag
#   make tools     — install the cargo tools the gates above need
#   make clean     — remove build artifacts

.PHONY: all check clippy test fmt lint deny vet audit doc coverage miri proptest loom kani mutants fuzz examples bench-smoke bench-gate bench-compare embedded versions sbom wasi demo complexity links msrv semver hack no-std dist distcheck preflight tools clean

all: check clippy test

check:
	cargo check --all-features --all-targets

clippy:
	cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo test --all-features

fmt:
	cargo fmt --all -- --check

# Docs lint: structure only for Markdown, British spellings allowed,
# REUSE 3.3 compliance. `reuse` runs through uvx so the gate does not
# depend on a system Python install.
lint:
	npx --yes markdownlint-cli2 "**/*.md" "!target/**" "!fuzz/target/**" "!node_modules/**"
	uvx codespell
	uvx --with chardet reuse lint

deny:
	cargo deny check

vet:
	cargo vet --locked

audit:
	cargo audit --deny warnings

doc:
	RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features

# Coverage as CI measures it. The gate is 98 % lines; the rationale is
# in DEVELOPMENT.md.
coverage:
	cargo +nightly llvm-cov --all-features --fail-under-lines 98

miri:
	cargo +nightly miri test --lib

# CI scale, optimised; rerun a failure with the printed seed:
# `make proptest PROPTEST_RNG_SEED=<seed>`.
ifndef PROPTEST_RNG_SEED
PROPTEST_RNG_SEED := $(shell od -An -N8 -tu8 /dev/urandom | tr -d ' ')
endif
proptest:
	@echo "PROPTEST_RNG_SEED=$(PROPTEST_RNG_SEED)"
	PROPTEST_RNG_SEED=$(PROPTEST_RNG_SEED) PROPTEST_CASES=10000 \
	  CARGO_PROFILE_TEST_OPT_LEVEL=3 cargo test --test test_proptest

loom:
	RUSTFLAGS="--cfg loom" cargo test --test loom_smoke --release

kani:
	cargo kani --tests --harness check_escape_html_totality

# A full run takes a while; `cargo mutants --in-diff <file>` narrows it
# to a diff, as the PR job does.
mutants:
	cargo mutants --no-shuffle || { rc=$$?; [ $$rc -eq 2 ] || [ $$rc -eq 3 ]; }
	python3 -I scripts/mutants_gate.py --min 0.85

# Build every target, then replay the seed corpus and the regression
# inputs without generating new ones. Mirrors the per-push CI gate.
fuzz:
	cd fuzz && cargo +nightly fuzz build
	cd fuzz && for t in $$(cargo +nightly fuzz list); do \
	  cargo +nightly fuzz run "$$t" -- -runs=0 "corpus/$$t" "regressions/$$t" || exit 1; \
	done

examples:
	@for f in examples/*.rs; do \
	  name=$$(basename "$$f" .rs); \
	  echo "== $$name"; cargo run --quiet --all-features --example "$$name" < /dev/null || exit 1; \
	done

bench-smoke:
	cargo bench --all-features -- --test

# Save a baseline on the commit you start from (`make bench-gate
# BENCH_SAVE=1`), change the code, then run `make bench-gate`.
bench-gate:
ifdef BENCH_SAVE
	cargo bench --all-features --bench '*' -- --save-baseline base
else
	cargo bench --all-features --bench '*' -- --baseline-lenient base
	python3 -I scripts/bench_compare.py --threshold 0.05
endif

bench-compare:
	cd perf/escape-compare && cargo bench

# `rustup target add thumbv7em-none-eabihf riscv32imac-unknown-none-elf
# riscv32imc-unknown-none-elf` and `rustup component add llvm-tools`
# once. Mirrors the `embedded` CI matrix.
embedded:
	./scripts/embedded_size.sh thumbv7em-none-eabihf yaml,toml,json
	./scripts/embedded_size.sh thumbv7em-none-eabihf yaml
	./scripts/embedded_size.sh thumbv7em-none-eabihf yaml,embedded-io-async
	./scripts/embedded_size.sh riscv32imac-unknown-none-elf yaml,toml,json
	./scripts/embedded_size.sh riscv32imc-unknown-none-elf json
	./scripts/embedded_firmware_size.sh thumbv7em-none-eabihf

versions:
	./scripts/verify-release-versions.sh

# CycloneDX 1.5 JSON, the format the release workflow attaches and
# attests. Written next to Cargo.toml as metadata-gen.cdx.json.
sbom:
	python3 -I scripts/check-sbom.py

wasi:
	python3 -I wasi/check.py

# Re-renders .github/demo.gif from .github/demo.tape (needs vhs). The
# README links the GIF by commit SHA, so it resolves before the commit
# reaches main; point it at the new commit after re-rendering.
demo:
	cargo build --quiet --all-features --example mcp_server
	vhs .github/demo.tape

# Ceilings: cyclomatic <= 10, cognitive <= 15, Halstead difficulty <= 30,
# <= 60 lines per function, <= 500 per file. The baseline may only shrink.
complexity:
	python3 -I scripts/complexity_check.py

links:
	lychee --config lychee.toml './**/*.md'

msrv:
	cargo +1.89.0 check --all-features --all-targets --locked

# While the crate is 0.0.x every release is a major bump under semver,
# so this proves the version moved; the lints bite from 0.1.0.
semver:
	cargo semver-checks check-release --all-features

hack:
	cargo hack check --feature-powerset --all-targets --locked

# `rustup target add thumbv7em-none-eabihf` once. Mirrors the CI job.
no-std:
	cargo build --lib --locked --no-default-features --features yaml,toml,json --target thumbv7em-none-eabihf
	cargo test --lib --locked --no-default-features --features yaml

dist:
	cargo package --locked

# What crates.io would receive: the archive must carry the current
# README, CHANGELOG and manifest, not a stale copy.
distcheck: dist
	@v=$$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2); \
	  f="target/package/metadata-gen-$$v.crate"; \
	  for want in Cargo.toml README.md CHANGELOG.md LICENSE-APACHE LICENSE-MIT; do \
	    tar tzf "$$f" | grep -q "^metadata-gen-$$v/$$want$$" || { echo "missing $$want in $$f"; exit 1; }; \
	  done; \
	  tar xzOf "$$f" "metadata-gen-$$v/CHANGELOG.md" | grep -q "^## \[$$v\]" || { echo "packaged CHANGELOG has no [$$v] section"; exit 1; }; \
	  echo "distcheck ok: $$f"

preflight:
	./scripts/release-preflight.sh $(TAG)

# Everything the gates need beyond rustup. The devcontainer runs only
# `rustup component add` on create so it boots fast; run this next.
tools:
	rustup toolchain install nightly --component miri,llvm-tools-preview
	cargo install --locked cargo-hack cargo-deny cargo-vet cargo-llvm-cov \
	  cargo-fuzz cargo-audit cargo-semver-checks cargo-cyclonedx \
	  rust-code-analysis-cli

clean:
	cargo clean
