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
#   make versions  — every version-bearing file agrees with Cargo.toml
#   make sbom      — CycloneDX software bill of materials (cargo-cyclonedx)
#   make complexity — per-function complexity gate against the baseline
#   make links     — link check every Markdown file (lychee)
#   make msrv      — the crate builds on the declared minimum Rust
#   make semver    — public API against the last crates.io release
#   make hack      — every feature combination compiles
#   make dist      — package the crate as crates.io would receive it
#   make distcheck — package, then verify the archive's contents
#   make preflight TAG=vX.Y.Z — blocking release preflight for a local tag
#   make tools     — install the cargo tools the gates above need
#   make clean     — remove build artifacts

.PHONY: all check clippy test fmt lint deny vet audit doc coverage miri proptest loom kani mutants fuzz examples bench-smoke versions sbom complexity links msrv semver hack dist distcheck preflight tools clean

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

proptest:
	cargo test --test test_proptest

loom:
	RUSTFLAGS="--cfg loom" cargo test --test loom_smoke --release

kani:
	cargo kani --harness check_escape_html_totality
	cargo kani --harness check_html_escape_ascii_roundtrip

mutants:
	cargo mutants --no-shuffle --check

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
	  echo "== $$name"; cargo run --quiet --all-features --example "$$name" || exit 1; \
	done

bench-smoke:
	cargo bench --all-features -- --test

versions:
	./scripts/verify-release-versions.sh

# CycloneDX 1.5 JSON, the format the release workflow attaches and
# attests. Written next to Cargo.toml as metadata-gen.cdx.json.
sbom:
	cargo cyclonedx --format json --all-features

# Ceilings: cyclomatic <= 10, cognitive <= 15, Halstead difficulty <= 30,
# <= 60 lines per function, <= 500 per file. The baseline may only shrink.
complexity:
	python3 -I scripts/complexity_check.py

links:
	lychee --config lychee.toml './**/*.md'

msrv:
	cargo +1.88.0 check --all-features --all-targets --locked

# While the crate is 0.0.x every release is a major bump under semver,
# so this proves the version moved; the lints bite from 0.1.0.
semver:
	cargo semver-checks check-release --all-features

hack:
	cargo hack check --feature-powerset --all-targets --locked

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
