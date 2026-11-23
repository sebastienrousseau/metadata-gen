// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Kani proofs for HTML escaping (#55).
//!
//! Kani checks a harness for every possible value of its symbolic
//! inputs, not a sample of them, so within the stated bound these are
//! proofs rather than tests. Run them with `make kani`; CI runs them in
//! the `kani` job of `.github/workflows/assurance.yml`.
//!
//! # Verified properties
//!
//! 1. `check_escape_html_totality`: for every one-char string,
//!    `escape_html` returns without panicking, overflowing or touching
//!    memory out of bounds.
//!
//! 2. `check_html_escape_ascii_roundtrip`: for every ASCII string
//!    of one char (`U+0000` to `U+007F`),
//!    `unescape_html(escape_html(s))` is `s` again. Since `unescape_html`
//!    is a function, this also proves `escape_html` injective on that
//!    domain: two different inputs can never escape to the same output.
//!
//! The bounds are what the model checker finishes on a hosted runner:
//! at two or three chars the String reallocations it must model ran
//! past 15 minutes or out of memory. Larger domains are covered by
//! enumeration in `tests/test_escape_exhaustive.rs` (every char, every
//! ASCII string of up to three chars) and, beyond that, statistically
//! by the proptest suite and the `fuzz_html_escape` target.

#[cfg(kani)]
mod kani_proofs {
    use metadata_gen::utils::{escape_html, unescape_html};

    /// `core::slice::memchr::memchr` as a plain loop. `str::find`
    /// reaches the word-at-a-time version through pointer alignment
    /// arithmetic that the model checker explores at great cost; this
    /// returns the same index, so the proof is about `unescape_html`,
    /// not about the standard library's search.
    fn naive_memchr(x: u8, text: &[u8]) -> Option<usize> {
        text.iter().position(|&b| b == x)
    }

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

    /// Every one-char ASCII string round-trips.
    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(8)]
    #[kani::stub(core::slice::memchr::memchr, naive_memchr)]
    fn check_html_escape_ascii_roundtrip() {
        let c: char = kani::any();
        kani::assume(c.is_ascii());
        let mut buf = [0u8; 4];
        let s: &str = c.encode_utf8(&mut buf);
        assert_eq!(unescape_html(&escape_html(s)), s);
    }
}
