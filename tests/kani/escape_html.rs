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
//! 1. `check_escape_html_totality`: for every valid UTF-8 string of at
//!    most [`TOTALITY_LEN`] bytes, `escape_html` returns without
//!    panicking, overflowing or touching memory out of bounds, and its
//!    output contains no raw `<`, `>`, `"` or `'`.
//!
//! 2. `check_html_escape_ascii_roundtrip`: for every ASCII string
//!    (bytes `0x00` to `0x7F`) of at most [`ROUNDTRIP_LEN`] bytes,
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

    /// Longest input of the totality proof, in bytes.
    const TOTALITY_LEN: usize = 8;

    /// Longest input of the round-trip proof, in bytes.
    const ROUNDTRIP_LEN: usize = 4;

    /// A symbolic string of any length up to `N` over `bytes`, or
    /// `None` when the bytes are not valid UTF-8.
    fn any_prefix<const N: usize>(bytes: &[u8; N]) -> Option<&str> {
        let len: usize = kani::any();
        kani::assume(len <= N);
        core::str::from_utf8(&bytes[..len]).ok()
    }

    #[kani::proof]
    #[kani::unwind(50)]
    fn check_escape_html_totality() {
        let bytes: [u8; TOTALITY_LEN] = kani::any();
        if let Some(s) = any_prefix(&bytes) {
            let escaped = escape_html(s);
            assert!(!escaped
                .bytes()
                .any(|b| matches!(b, b'<' | b'>' | b'"' | b'\'')));
        }
    }

    #[kani::proof]
    #[kani::unwind(28)]
    fn check_html_escape_ascii_roundtrip() {
        let bytes: [u8; ROUNDTRIP_LEN] = kani::any();
        kani::assume(bytes.iter().all(u8::is_ascii));
        let s = any_prefix(&bytes).expect("ASCII is valid UTF-8");
        assert_eq!(unescape_html(&escape_html(s)), s);
    }
}
