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
//! 1. `check_escape_html_totality`: for every string of at most
//!    `TOTALITY_LEN` chars, `escape_html` returns without
//!    panicking, overflowing or touching memory out of bounds, and its
//!    output contains no raw `<`, `>`, `"` or `'`.
//!
//! 2. `check_html_escape_ascii_roundtrip`: for every ASCII string
//!    (`U+0000` to `U+007F`) of at most `ROUNDTRIP_LEN` chars,
//!    `unescape_html(escape_html(s))` is `s` again. Since `unescape_html`
//!    is a function, this also proves `escape_html` injective on that
//!    domain: two different inputs can never escape to the same output.
//!
//! The bounds are what the model checker can explore exhaustively in a
//! few minutes. Longer inputs are covered statistically by the proptest
//! suite (`tests/test_proptest.rs`) and the `fuzz_html_escape` target.

#[cfg(kani)]
mod kani_proofs {
    use metadata_gen::utils::{escape_html, unescape_html};

    /// Longest input of the totality proof, in chars.
    const TOTALITY_LEN: usize = 4;

    /// Longest input of the round-trip proof, in chars.
    const ROUNDTRIP_LEN: usize = 3;

    /// A symbolic string of any length up to `N` chars, built char by
    /// char so the model checker never has to reason about UTF-8
    /// validation: every `char` is a valid scalar value, so every
    /// string this returns is valid UTF-8, and every valid string of up
    /// to `N` chars is one of them. With `ascii`, every char is ASCII.
    fn any_string<const N: usize>(ascii: bool) -> String {
        let len: usize = kani::any();
        kani::assume(len <= N);
        let mut s = String::with_capacity(4 * N);
        for i in 0..N {
            if i < len {
                let c: char = kani::any();
                kani::assume(!ascii || c.is_ascii());
                s.push(c);
            }
        }
        s
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(26)]
    fn check_escape_html_totality() {
        let s = any_string::<TOTALITY_LEN>(false);
        let escaped = escape_html(&s);
        assert!(!escaped
            .bytes()
            .any(|b| matches!(b, b'<' | b'>' | b'"' | b'\'')));
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    #[kani::unwind(26)]
    fn check_html_escape_ascii_roundtrip() {
        let s = any_string::<ROUNDTRIP_LEN>(true);
        assert_eq!(unescape_html(&escape_html(&s)), s);
    }
}
