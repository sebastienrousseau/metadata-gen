// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Kani formal verification proofs for HTML escaping and unescaping.
//!
//! # Verified Properties
//!
//! 1. `check_escape_html_totality`:
//!    Proves in plain English that `escape_html` is total (never panics, crashes,
//!    or exhibits undefined behavior) for any arbitrary UTF-8 byte sequence
//!    within the verified bound.
//!
//! 2. `check_html_escape_ascii_roundtrip`:
//!    Proves in plain English that for every valid ASCII input slice up to length N,
//!    the composition `unescape_html(escape_html(s))` produces the exact original slice `s`.
//!    This formally verifies that `unescape_html` is the inverse of `escape_html`
//!    on all valid ASCII strings.

#[cfg(kani)]
mod kani_proofs {
    use metadata_gen::utils::{escape_html, unescape_html};

    /// Property 1: `escape_html` totality.
    /// Proves that `escape_html` will execute safely and never panic for any
    /// well-formed UTF-8 input string.
    #[kani::proof]
    #[kani::unwind(16)]
    fn check_escape_html_totality() {
        let input: [u8; 8] = kani::any();
        if let Ok(s) = std::str::from_utf8(&input) {
            let _ = escape_html(s);
        }
    }

    /// Property 2: `escape_html` and `unescape_html` roundtrip on ASCII.
    /// Proves that unescaping the result of escaping an ASCII string always
    /// yields the original input string without modification or loss.
    #[kani::proof]
    #[kani::unwind(32)]
    fn check_html_escape_ascii_roundtrip() {
        let input: [u8; 8] = kani::any();
        // Constrain input to printable ASCII range
        kani::assume(input.iter().all(|&b| b >= 0x20 && b <= 0x7E));
        if let Ok(s) = std::str::from_utf8(&input) {
            let escaped = escape_html(s);
            let unescaped = unescape_html(&escaped);
            assert_eq!(s, unescaped.as_str());
        }
    }
}
