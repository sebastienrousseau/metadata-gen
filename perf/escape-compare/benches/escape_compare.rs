// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `metadata_gen::utils::escape_html` against `v_htmlescape` on 64 KiB.
//!
//! Two inputs bracket real use: prose with no character to escape (the
//! common case for titles and descriptions) and markup-dense text with
//! seven escapable bytes in every 27. `v_htmlescape` also escapes `/`,
//! which neither input contains, so both crates write the same output
//! here; the comparison is of throughput.

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::hint::black_box;

const SIZE: usize = 64 << 10;

/// `unit` repeated to exactly `SIZE` bytes. The repeat count comes from
/// the unit's own length: a hand-counted divisor once fell 2 KiB short
/// and the slice below panicked. Both units are ASCII, so any cut is a
/// char boundary.
fn fill(unit: &str) -> String {
    unit.repeat(SIZE / unit.len() + 1)[..SIZE].to_string()
}

fn inputs() -> [(&'static str, String); 2] {
    [
        (
            "clean",
            fill("The quick brown fox jumps over the lazy dog. "),
        ),
        ("dense", fill("a <b> & \"c\" 'd' plain text ")),
    ]
}

fn compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("escape_64KiB");
    for (name, input) in inputs() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("metadata_gen", name),
            &input,
            |b, s| {
                b.iter(|| {
                    metadata_gen::utils::escape_html(black_box(s))
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("v_htmlescape", name),
            &input,
            |b, s| {
                b.iter(|| {
                    let mut out =
                        String::with_capacity(s.len() + s.len() / 8);
                    v_htmlescape::escape_string(black_box(s), &mut out);
                    out
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, compare);
criterion_main!(benches);
