// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Kani proofs for HTML escaping (#55).
//!
//! Kani checks a harness for every possible value of its symbolic
//! inputs, not a sample of them, so within the stated bound these are
//! proofs rather than tests. Run them with `make kani`; CI runs them in
//! the `kani` job of `.github/workflows/assurance.yml`.
//!
//! # Verified property
//!
//! `check_escape_html_totality`: for every one-char string (every
//! Unicode scalar value), `escape_html` returns without panicking,
//! overflowing or touching memory out of bounds. Verified in about a
//! minute.
//!
//! # Why there is no round-trip harness
//!
//! A harness for `unescape_html(escape_html(s)) == s` on one ASCII
//! char made CBMC run out of memory, locally and on a hosted runner,
//! even with `memchr` stubbed: the entity matching in `unescape_html`
//! over a heap `String` is more than the model checker can hold. The
//! round trip, and with it injectivity, is instead proved by
//! enumeration in `tests/test_escape_exhaustive.rs` for every Unicode
//! char and every ASCII string of up to three chars, and checked
//! statistically beyond that by the proptest suite and the
//! `fuzz_html_escape` target.

#[cfg(kani)]
mod kani_proofs {
    use metadata_gen::utils::escape_html;

    /// Every one-char string: the char is symbolic and encoded into a
    /// stack buffer, so the input itself needs no heap model.
    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(8)]
    fn check_escape_html_totality() {
        let c: char = kani::any();
        let mut buf = [0u8; 4];
        let _ = escape_html(c.encode_utf8(&mut buf));
    }
}
