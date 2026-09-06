// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `escape_html` / `unescape_html` as a pair: escaping never produces a
//! raw `<`, `>`, `&`, `"` or `'`, and unescaping the escaped form gives
//! the input back. The reverse direction (unescape then escape) is not
//! an identity and is deliberately not asserted.
#![no_main]

use libfuzzer_sys::fuzz_target;
use metadata_gen::utils::{escape_html, unescape_html};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    let escaped = escape_html(text);
    assert!(!escaped.contains(['<', '>', '"', '\'']));
    assert!(!escaped.contains("& "));
    assert_eq!(unescape_html(&escaped), text, "escape/unescape is not an identity");
});
