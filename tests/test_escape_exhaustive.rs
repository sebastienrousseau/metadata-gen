// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Exhaustive checks of the HTML escape pair over small domains (#55).
//!
//! Every input in each domain is tried, so within it these are proofs by
//! enumeration, the same claim the Kani harnesses in `tests/kani/` make
//! symbolically at smaller bounds:
//!
//! - every Unicode scalar value as a one-char string: `escape_html`
//!   returns, leaves no raw `<`, `>`, `"` or `'`, and `unescape_html`
//!   gives the char back;
//! - every ASCII string of up to three chars (2 113 665 strings): the
//!   same, which also makes `escape_html` injective on that domain.

use metadata_gen::utils::{escape_html, unescape_html};

fn check(s: &str) {
    let escaped = escape_html(s);
    assert!(
        !escaped.contains(['<', '>', '"', '\'']),
        "raw markup left in {escaped:?} from {s:?}"
    );
    assert_eq!(unescape_html(&escaped), s, "round trip of {s:?}");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "a million inputs; the unit tests cover Miri"
)]
fn every_char_round_trips() {
    let mut buf = [0u8; 4];
    for c in (0..=u32::from(char::MAX)).filter_map(char::from_u32) {
        check(c.encode_utf8(&mut buf));
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "two million inputs; the unit tests cover Miri"
)]
fn every_ascii_string_up_to_three_chars_round_trips() {
    let ascii = || (0u8..0x80).map(char::from);
    let mut s = String::with_capacity(3);
    check("");
    for a in ascii() {
        for b in ascii() {
            for c in ascii() {
                s.clear();
                s.extend([a, b, c]);
                check(&s);
            }
            s.clear();
            s.extend([a, b]);
            check(&s);
        }
        s.clear();
        s.push(a);
        check(&s);
    }
}
