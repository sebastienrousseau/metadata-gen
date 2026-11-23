// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Sustained transactions per second for `extract_metadata` on a 1 KiB
//! document (#49). Criterion reports `elem/s`; one element is one
//! complete extraction, so the figure is the TPS a single thread
//! sustains.

use criterion::{
    criterion_group, criterion_main, Criterion, Throughput,
};
use metadata_gen::extract_metadata;
use std::hint::black_box;

/// A realistic 1 KiB post header: a dozen fields, nested author, list.
fn one_kib_post() -> String {
    let mut doc = String::from(
        "---\ntitle: Sustained throughput\ndate: 2026-01-02\n\
         author:\n  name: Ada Lovelace\n  url: https://example.com/ada\n\
         tags: [rust, metadata, seo, front-matter]\n",
    );
    let mut i = 0;
    while doc.len() < 1000 {
        doc.push_str(&format!("field_{i}: value number {i}\n"));
        i += 1;
    }
    doc.push_str("---\n# Body\n\nText.\n");
    doc
}

fn stress_tps(c: &mut Criterion) {
    let doc = one_kib_post();
    let mut group = c.benchmark_group("tps");
    group.throughput(Throughput::Elements(1));
    group.bench_function("extract_metadata_1KiB", |b| {
        b.iter(|| extract_metadata(black_box(&doc)))
    });
    group.finish();
}

criterion_group!(benches, stress_tps);
criterion_main!(benches);
