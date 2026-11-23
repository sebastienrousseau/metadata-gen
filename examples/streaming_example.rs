// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Walking front matter as events without building a map (issue #50).
//!
//! A synthetic sitemap manifest of 100 000 URLs in JSON front matter is
//! walked once, counting entries, and once more stopping at the first
//! entry. JSON is used because YAML parsing runs under noyalib's strict
//! budget, which caps a sequence at 1 024 items. Run with
//! `cargo run --example streaming_example`.

use metadata_gen::metadata::events::for_each_event_with_limits;
use metadata_gen::{Event, MetadataError, ParseLimits};
use std::fmt::Write as _;
use std::ops::ControlFlow;

fn main() -> Result<(), MetadataError> {
    let mut doc = String::from("{\"urls\": [");
    for i in 0..100_000 {
        if i > 0 {
            doc.push(',');
        }
        let _ = write!(doc, "\"https://example.com/page/{i}\"");
    }
    doc.push_str("]}\n# Sitemap\n");

    // The default block limit is 1 MiB; this manifest is about 3.3 MiB.
    let limits =
        ParseLimits::default().max_block_bytes(8 * 1024 * 1024);
    let mut urls = 0_usize;
    let _ = for_each_event_with_limits(&doc, &limits, |event| {
        if matches!(event, Event::Scalar(_)) {
            urls += 1;
        }
        ControlFlow::Continue(())
    })?;
    println!("{} bytes of front matter, {urls} URLs", doc.len());

    let mut first = None;
    let flow =
        for_each_event_with_limits(
            &doc,
            &limits,
            |event| match event {
                Event::Scalar(s) => {
                    first = Some(s.to_text());
                    ControlFlow::Break(())
                }
                _ => ControlFlow::Continue(()),
            },
        )?;
    println!(
        "stopped early: {}, first URL: {first:?}",
        flow.is_break()
    );
    Ok(())
}
