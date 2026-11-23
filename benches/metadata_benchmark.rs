// benches/metadata_benchmark.rs

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use metadata_gen::{
    extract_and_prepare_metadata,
    metadata::{extract_metadata, process_metadata, Metadata},
    metatags::{extract_meta_tags, generate_metatags},
    utils::{escape_html, unescape_html},
};
use std::collections::HashMap;
use std::hint::black_box;

fn benchmark_extract_and_prepare_metadata(c: &mut Criterion) {
    let content = r#"---
title: Benchmark Test Page
description: A test page for benchmarking metadata extraction and processing
keywords: benchmark, metadata, extraction, processing
date: 2023-05-25
author: John Doe
---
# Benchmark Content
This is a test file for benchmarking metadata extraction and processing."#;

    c.bench_function("extract_and_prepare_metadata", |b| {
        b.iter(|| extract_and_prepare_metadata(black_box(content)))
    });
}

fn benchmark_extract_metadata(c: &mut Criterion) {
    let content = r#"---
title: Benchmark Test Page
description: A test page for benchmarking metadata extraction
keywords: benchmark, metadata, extraction
---
# Benchmark Content"#;

    c.bench_function("extract_metadata", |b| {
        b.iter(|| extract_metadata(black_box(content)))
    });
}

fn benchmark_process_metadata(c: &mut Criterion) {
    let mut metadata = HashMap::new();
    metadata
        .insert("title".to_string(), "Benchmark Test Page".to_string());
    metadata.insert(
        "description".to_string(),
        "A test page for benchmarking metadata processing".to_string(),
    );
    metadata.insert("date".to_string(), "2023-05-25".to_string());
    let metadata = Metadata::new(metadata);

    c.bench_function("process_metadata", |b| {
        b.iter(|| process_metadata(black_box(&metadata)))
    });
}

fn benchmark_generate_metatags(c: &mut Criterion) {
    let mut metadata = HashMap::new();
    metadata
        .insert("title".to_string(), "Benchmark Test Page".to_string());
    metadata.insert(
        "description".to_string(),
        "A test page for benchmarking meta tag generation".to_string(),
    );
    metadata.insert(
        "og:title".to_string(),
        "OG Benchmark Test Page".to_string(),
    );
    metadata.insert("twitter:card".to_string(), "summary".to_string());

    c.bench_function("generate_metatags", |b| {
        b.iter(|| generate_metatags(black_box(&metadata)))
    });
}

fn benchmark_escape_html(c: &mut Criterion) {
    let input =
        r#"<script>alert("XSS");</script> & "quotes" & 'apostrophes'"#;

    c.bench_function("escape_html", |b| {
        b.iter(|| escape_html(black_box(input)))
    });
}

fn benchmark_unescape_html(c: &mut Criterion) {
    let input = r#"&lt;script&gt;alert(&quot;XSS&quot;);&lt;/script&gt; &amp; &quot;quotes&quot; &amp; &#x27;apostrophes&#x27;"#;

    c.bench_function("unescape_html", |b| {
        b.iter(|| unescape_html(black_box(input)))
    });
}

/// A YAML front-matter block of roughly `bytes` bytes: a fixed header
/// plus as many `key_N: value N` lines as fit, then a short body.
fn yaml_block_document(bytes: usize) -> String {
    let mut doc = String::with_capacity(bytes + 64);
    doc.push_str("---\ntitle: Throughput\ndate: 2024-01-02\n");
    let mut i = 0;
    while doc.len() + 40 < bytes {
        doc.push_str(&format!(
            "key_{i}: value number {i} with some text\n"
        ));
        i += 1;
    }
    doc.push_str("---\n# Body\n");
    doc
}

/// An HTML head with as many `<meta>` elements as fit in roughly `bytes`.
fn html_document(bytes: usize) -> String {
    let mut doc = String::with_capacity(bytes + 64);
    doc.push_str("<html><head>");
    let mut i = 0;
    while doc.len() + 80 < bytes {
        doc.push_str(&format!(
            "<meta name=\"field{i}\" content=\"value &amp; number {i}\">"
        ));
        i += 1;
    }
    doc.push_str("</head><body></body></html>");
    doc
}

/// The sizes of the throughput matrix (#49): 1 KiB, 10 KiB, 1 MiB and
/// 10 MiB. The point is the slope: a regression that only shows at
/// scale is invisible on the 200-byte fixtures above.
const SIZES: [usize; 4] = [1 << 10, 10 << 10, 1 << 20, 10 << 20];

/// Block sizes for the parse benchmark. The default `ParseLimits` cap a
/// YAML mapping at 1 024 keys and a block at 1 MiB, so a larger block
/// would measure the budget error, not the parse; 32 KiB is about 800
/// keys.
const BLOCK_SIZES: [usize; 3] = [1 << 10, 10 << 10, 32 << 10];

/// A document of `bytes` bytes as a static-site generator sees one: a
/// 1 KiB YAML header followed by a Markdown body. Extraction should cost
/// the header, not the body (#48).
fn yaml_document(bytes: usize) -> String {
    let mut doc = yaml_block_document(1 << 10);
    let line = "Body text that the delimiter scan must not read. ---\n";
    while doc.len() + line.len() < bytes {
        doc.push_str(line);
    }
    doc
}

/// Fewer samples for the large inputs so the whole matrix stays within
/// the CI budget; Criterion's minimum is 10.
fn sample_size(size: usize) -> usize {
    if size >= 1 << 20 {
        10
    } else {
        100
    }
}

/// Benchmarks `routine` over a document of every size in `sizes`,
/// reporting bytes per second.
fn throughput_group(
    c: &mut Criterion,
    name: &str,
    sizes: &[usize],
    make: fn(usize) -> String,
    routine: fn(&str),
) {
    let mut group = c.benchmark_group(name);
    for &size in sizes {
        let doc = make(size);
        group.sample_size(sample_size(size));
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &doc,
            |b, d| b.iter(|| routine(black_box(d))),
        );
    }
    group.finish();
}

/// Text with seven escapable bytes in every 27 (about one in four).
fn escapable_text(size: usize) -> String {
    "a <b> & \"c\" 'd' plain text ".repeat(size / 28 + 1)
}

/// What [`escapable_text`] escapes to: the unescape input.
fn escaped_text(size: usize) -> String {
    escape_html(&escapable_text(size))
}

fn benchmark_throughput(c: &mut Criterion) {
    // `unwrap` so a document the budget rejects fails the bench run
    // instead of timing the error path.
    throughput_group(
        c,
        "throughput/extract_metadata",
        &SIZES,
        yaml_document,
        |d| drop(black_box(extract_metadata(d).unwrap())),
    );
    throughput_group(
        c,
        "throughput/parse_yaml_block",
        &BLOCK_SIZES,
        yaml_block_document,
        |d| drop(black_box(extract_metadata(d).unwrap())),
    );
    throughput_group(
        c,
        "throughput/extract_meta_tags",
        &SIZES,
        html_document,
        |d| drop(black_box(extract_meta_tags(d))),
    );
    throughput_group(
        c,
        "throughput/escape_html",
        &SIZES,
        escapable_text,
        |d| drop(black_box(escape_html(d))),
    );
    throughput_group(
        c,
        "throughput/unescape_html",
        &SIZES,
        escaped_text,
        |d| drop(black_box(unescape_html(d))),
    );
}

criterion_group!(
    benches,
    benchmark_throughput,
    benchmark_extract_and_prepare_metadata,
    benchmark_extract_metadata,
    benchmark_process_metadata,
    benchmark_generate_metatags,
    benchmark_escape_html,
    benchmark_unescape_html
);
criterion_main!(benches);
