<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Property-testing a front-matter parser

*2026-11-23. Sources: `tests/test_proptest.rs`,
`tests/regression_corpus/`, `fuzz/`, `tests/test_json_nested.rs`,
`CHANGELOG.md` (0.0.7, 0.0.10).*

## The problem

A front-matter parser sits on untrusted input in most pipelines: posts
submitted through a CMS, pull requests to a documentation site, pages
scraped from elsewhere. Example-based tests cover the inputs someone
thought of. The inputs that break parsers (a quote inside a value, a
backslash before a quote, a brace in the body, an entity that decodes
to another entity) are the ones nobody thought of.

`metadata-gen` uses three tools that each generate inputs instead of
listing them: Proptest properties, libFuzzer targets, and Kani proofs.
This paper covers what each one checks, the bug they found, and how
the suite is sized so it is cheap on a laptop and thorough in CI.

## The properties

`tests/test_proptest.rs` states a handful of properties, each a claim
that must hold for every generated input.

**Round trip, per format.** Generate a map of one to eight keys
(`[a-z][a-z0-9_]{0,10}`) to values drawn from any printable Unicode up
to 30 characters, quotes and backslashes included. Serialise it as
YAML double-quoted scalars, as TOML basic strings, or with
`serde_json`, put it at the top of a document with a body, and extract
it. The extracted map must equal the generated one, whole. The value
strategy excludes leading and trailing whitespace on purpose: the flat
map is documented as trimmed text, so a property that demanded
whitespace survive would be testing a promise the crate does not make.

**Escape involution.** For any string, `unescape_html(escape_html(s))`
must equal `s`. One variant draws from the full ASCII range, control
characters included; another from printable Unicode.

**No raw specials.** The output of `escape_html` must contain none of
`<`, `>`, `"` or `'`. (`&` is left out of this property because every
entity the function writes starts with one; the involution property
covers it.)

**Equivalence with the old implementation.** 0.0.7 replaced a chain of
five `replace` calls in `escape_html` with a single pass and one
allocation (issue #47). The property `escape_html_matches_replace_chain`
keeps the old chain in the test file and demands byte-identical output
for every input, so the optimisation can never change what users get.

## The bug they found

The involution property is the one that paid for itself. Before 0.0.7,
`unescape_html` was also a chain of `replace` calls, so it decoded its
own output: `&amp;lt;` became `&lt;` after the first replacement and
`<` after the next. Text that `escape_html` had just protected was
unprotected again by the reverse function, which is the kind of
double-decoding that turns into markup injection.

The changelog for 0.0.7 records that the `fuzz_html_escape` libFuzzer
target found it on its seed corpus. The fix made `unescape_html` a
single left-to-right pass that decodes each entity once. The input,
`&amp;lt;`, is kept twice: in `fuzz/regressions/fuzz_html_escape/`,
which the fuzz job replays on every push, and in
`tests/regression_corpus/double-decode-amp-lt`, which
`test_regression_corpus_replay` replays in the ordinary test suite, so
the case is checked even where the fuzzer does not run.

A second finding came from a property over random nested JSON objects
(`tests/test_json_nested.rs`, issue #26). The 0.0.10 changelog notes
that the same nested document read with 0.0.4's regex-based detector
gave "No valid front matter found." The current scanner finds the end
of a JSON block with `serde_json`'s streaming deserializer, which
handles nesting and braces inside strings by construction.

## Sizing the suite

A property test is only as good as the number of cases it runs, and
only as useful as it is fast enough to run. The suite reads
`PROPTEST_CASES`:

- Locally it defaults to 1,000 cases per round-trip property.
- CI sets it so each round trip runs 10,000 cases and the ASCII escape
  involution runs 100,000, and prints the seed (changelog 0.0.10). A
  failure in CI is reproduced locally with `PROPTEST_RNG_SEED=<seed>`.
- Under Miri each property runs four cases and failures are not
  persisted (proptest's persistence calls `getcwd`, which Miri's
  isolation forbids), so Miri checks the same code paths for undefined
  behaviour without the interpreter's cost per case.

The title of this paper's issue promised "10 000 adversarial
frontmatters". That is the per-property case count in CI, per run; it
is not a corpus of 10,000 saved files, and the repository does not
ship one. What it ships is the generators, so any run produces a fresh
set, and the regression corpus, so every input that ever failed is
replayed forever.

## Beyond random inputs

Random generation finds bugs that are likely; it cannot show that a
bug is absent. Two other tools cover that ground:

- **Kani** (`tests/kani/`) proves properties of `escape_html` for every
  input up to a length bound: totality, the ASCII round trip, and that
  escaped output holds no raw markup. The 0.0.10 changelog notes the
  proofs build inputs character by character so the solver does not
  spend its time exploring UTF-8 validation.
- **libFuzzer** (`fuzz/`) runs three targets: `fuzz_extract_metadata`,
  `fuzz_extract_meta_tags` and `fuzz_html_escape`. CI replays their
  seed corpora and regressions on every push and runs them longer on a
  schedule.

Mutation testing (`cargo-mutants`) closes the loop by changing the
code and checking that some test fails; `tests/test_edge_cases.rs`
exists because it showed boundaries no test covered.

## What this does not prove

- The round-trip properties generate front matter with the crate's
  own serialisation rules (double-quoted YAML, TOML basic strings). They
  say nothing about block scalars, anchors, or TOML arrays of tables;
  those are covered by example tests and by the YAML parser's own
  suite.
- No property compares `metadata-gen` with another parser. Agreement
  with `noyalib`, `toml` and `serde_json` is assumed, because those
  crates do the parsing.
- The fuzz targets check for panics and for the properties they
  assert, not for performance cliffs.

## Lessons

- An involution property (`f⁻¹(f(x)) == x`) is the cheapest strong
  test an encoder can have. It found the one real security-relevant bug
  in this crate's history.
- Keep the old implementation in the tests when you optimise. A
  property that demands byte-identical output turns a risky rewrite
  into a safe one.
- Save every failing input twice: where the fuzzer replays it and where
  the ordinary test suite does.
- Scale case counts with an environment variable and print the seed.
  Thorough in CI and fast at a desk are both possible with one file.

## Reproduce

```bash
cargo test --all-features --test test_proptest
PROPTEST_CASES=10000 cargo test --all-features --test test_proptest
make fuzz     # build every target, replay corpus and regressions
make kani     # needs cargo-kani
```

Links: [repository](https://github.com/sebastienrousseau/metadata-gen),
[crates.io](https://crates.io/crates/metadata-gen),
[ADR index](../adr/README.md).
