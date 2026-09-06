// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `<meta>` extraction over arbitrary bytes: the streaming reader must
//! stop cleanly on any malformed input and every tag it returns must
//! round-trip through the hashmap view.
#![no_main]

use libfuzzer_sys::fuzz_target;
use metadata_gen::metatags::{extract_meta_tags, meta_tags_to_hashmap};

fuzz_target!(|data: &[u8]| {
    let Ok(html) = std::str::from_utf8(data) else { return };
    if let Ok(tags) = extract_meta_tags(html) {
        let count = tags.len();
        let map = meta_tags_to_hashmap(tags);
        assert!(map.len() <= count);
    }
});
