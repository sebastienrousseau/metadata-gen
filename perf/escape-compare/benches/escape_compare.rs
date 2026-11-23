// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `metadata_gen::utils::escape_html` against `v_htmlescape` on 64 KiB.
//!
//! Two inputs bracket real use: prose with no character to escape (the
//! common case for titles and descriptions) and markup-dense text with
//! one escapable byte in seven. `v_htmlescape` also escapes `/`, so on
//! the dense input it writes slightly more; the comparison is of
//! throughput, not output.

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::hint::black_box;

const SIZE: usize = 64 << 10;

fn inputs() -> [(&'static str, String); 2] {
    let clean = "The quick brown fox jumps over the lazy dog. "
        .repeat(SIZE / 45 + 1);
    let dense = "a <b> & \"c\" 'd' plain text ".repeat(SIZE / 28 + 1);
    [
        ("clean", clean[..SIZE].to_string()),
        ("dense", dense[..SIZE].to_string()),
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
