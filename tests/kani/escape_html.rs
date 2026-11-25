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
//! `check_escape_html_totality`: for every one-char string (every
//! Unicode scalar value), `escape_html` returns without panicking,
//! overflowing or touching memory out of bounds. Verified in about a
//! minute.
//!
//! `check_round_trip_*`: for every one-char ASCII string `s`,
//! `escape_html(s)` is `s` itself or, for `&`, `<`, `>`, `"` and `'`,
//! its entity, and `unescape_html` turns that escape back into `s`; so
//! `unescape_html(escape_html(s)) == s`, again with no panic, overflow
//! or out-of-bounds access in either function. The four harnesses split
//! the 128 ASCII chars into classes whose escapes share a length, and
//! between them cover every one. They need per-loop unwind bounds, so
//! they run through `scripts/kani-round-trip.sh`, which says why.
//!
//! # Why the round trip stops at one char
//!
//! Checked directly, `unescape_html(escape_html(s))` did not finish in
//! 20 minutes even for one ASCII char: the escape has a symbolic length,
//! and the entity matching over a heap `String` of symbolic length is
//! more than CBMC can solve. Fixing the escape's length per class is
//! what makes one char tractable. Two chars did not finish either: two
//! symbolic plain bytes ran CBMC out of memory on an 8 GB machine, or
//! past 20 minutes with tighter bounds. Longer strings, and with them
//! injectivity, are instead proved by enumeration in
//! `tests/test_escape_exhaustive.rs` for every Unicode char and every
//! ASCII string of up to three chars, and checked statistically beyond
//! that by the proptest suite and the `fuzz_html_escape` target.

#[cfg(kani)]
mod kani_proofs {
    use metadata_gen::utils::{escape_html, unescape_html};

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

    /// The five bytes `escape_html` rewrites, grouped by the length of
    /// their entity so that every member of a class escapes to a
    /// string of the same length.
    #[derive(Clone, Copy)]
    enum Class {
        /// Any ASCII byte other than `&`, `<`, `>`, `"` and `'`.
        Plain,
        /// `&`, escaped to `&amp;`.
        Amp,
        /// `<` or `>`, escaped to `&lt;` or `&gt;`.
        Angle,
        /// `"` or `'`, escaped to `&quot;` or `&#x27;`.
        Quote,
    }

    /// A symbolic ASCII byte of `class` and the escape it must get.
    /// The escape is written byte by byte at a fixed length, so the
    /// string `unescape_html` reads has a concrete length.
    fn pick(class: Class) -> (u8, [u8; 6], usize) {
        let mut esc = [0u8; 6];
        let (b, ent): (u8, &[u8]) = match class {
            Class::Plain => {
                let b: u8 = kani::any();
                kani::assume(b.is_ascii());
                kani::assume(!matches!(
                    b,
                    b'&' | b'<' | b'>' | b'"' | b'\''
                ));
                esc[0] = b;
                return (b, esc, 1);
            }
            Class::Amp => (b'&', b"&amp;"),
            Class::Angle if kani::any() => (b'<', b"&lt;"),
            Class::Angle => (b'>', b"&gt;"),
            Class::Quote if kani::any() => (b'"', b"&quot;"),
            Class::Quote => (b'\'', b"&#x27;"),
        };
        esc[..ent.len()].copy_from_slice(ent);
        (b, esc, ent.len())
    }

    /// For every byte of `class`: `escape_html` gives the expected
    /// escape, and `unescape_html` turns that escape back into the
    /// byte. Together these are the round trip on that class.
    fn check_round_trip(class: Class) {
        let (b, esc, len) = pick(class);
        let orig = [b];
        // SAFETY: both buffers hold ASCII bytes only, which is UTF-8.
        let s = unsafe { core::str::from_utf8_unchecked(&orig) };
        let t = unsafe { core::str::from_utf8_unchecked(&esc[..len]) };
        assert!(escape_html(s).as_bytes() == t.as_bytes());
        assert!(unescape_html(t).as_bytes() == s.as_bytes());
    }

    #[kani::proof]
    fn check_round_trip_plain() {
        check_round_trip(Class::Plain);
    }

    #[kani::proof]
    fn check_round_trip_amp() {
        check_round_trip(Class::Amp);
    }

    #[kani::proof]
    fn check_round_trip_angle() {
        check_round_trip(Class::Angle);
    }

    #[kani::proof]
    fn check_round_trip_quote() {
        check_round_trip(Class::Quote);
    }
}
