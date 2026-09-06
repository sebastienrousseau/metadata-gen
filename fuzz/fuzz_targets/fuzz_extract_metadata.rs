// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Front matter in all three shapes, through the full pipeline.
//!
//! The property is totality: any input either yields metadata or a
//! `MetadataError`; nothing panics, nothing hangs. The processing step is
//! run too so date normalisation and slug derivation see hostile input.
#![no_main]

use libfuzzer_sys::fuzz_target;
use metadata_gen::{extract_and_prepare_metadata, extract_metadata, process_metadata};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    if let Ok(meta) = extract_metadata(text) {
        let _ = process_metadata(&meta);
    }
    let _ = extract_and_prepare_metadata(text);
});
