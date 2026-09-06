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

/// A YAML front-matter document of roughly `bytes` bytes: a fixed header
/// plus as many `key_N: value N` lines as fit, then a short body.
fn yaml_document(bytes: usize) -> String {
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

/// Throughput at 1 KB, 10 KB and 1 MB (#49). The point is the slope: a
/// regression that only shows at scale is invisible on the 200-byte
/// fixtures above.
fn benchmark_throughput(c: &mut Criterion) {
    let sizes = [1 << 10, 10 << 10, 1 << 20];

    let mut group = c.benchmark_group("throughput/extract_metadata");
    for &size in &sizes {
        let doc = yaml_document(size);
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &doc,
            |b, d| b.iter(|| extract_metadata(black_box(d))),
        );
    }
    group.finish();

    let mut group = c.benchmark_group("throughput/extract_meta_tags");
    for &size in &sizes {
        let doc = html_document(size);
        group.throughput(Throughput::Bytes(doc.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &doc,
            |b, d| b.iter(|| extract_meta_tags(black_box(d))),
        );
    }
    group.finish();

    let mut group = c.benchmark_group("throughput/escape_html");
    for &size in &sizes {
        let text: String =
            "a <b> & \"c\" 'd' plain text ".repeat(size / 28 + 1);
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &text,
            |b, d| b.iter(|| escape_html(black_box(d))),
        );
    }
    group.finish();
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
