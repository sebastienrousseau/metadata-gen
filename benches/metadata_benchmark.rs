// benches/metadata_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use metadata_gen::{
    extract_and_prepare_metadata,
    metadata::{extract_metadata, process_metadata, Metadata},
    metatags::generate_metatags,
    utils::{escape_html, unescape_html},
};
use std::collections::HashMap;

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
    metadata.insert("title".to_string(), "Benchmark Test Page".to_string());
    metadata.insert("description".to_string(), "A test page for benchmarking metadata processing".to_string());
    metadata.insert("date".to_string(), "2023-05-25".to_string());
    let metadata = Metadata::new(metadata);

    c.bench_function("process_metadata", |b| {
        b.iter(|| process_metadata(black_box(&metadata)))
    });
}

fn benchmark_generate_metatags(c: &mut Criterion) {
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), "Benchmark Test Page".to_string());
    metadata.insert("description".to_string(), "A test page for benchmarking meta tag generation".to_string());
    metadata.insert("og:title".to_string(), "OG Benchmark Test Page".to_string());
    metadata.insert("twitter:card".to_string(), "summary".to_string());

    c.bench_function("generate_metatags", |b| {
        b.iter(|| generate_metatags(black_box(&metadata)))
    });
}

fn benchmark_escape_html(c: &mut Criterion) {
    let input = r#"<script>alert("XSS");</script> & "quotes" & 'apostrophes'"#;

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

criterion_group!(
    benches,
    benchmark_extract_and_prepare_metadata,
    benchmark_extract_metadata,
    benchmark_process_metadata,
    benchmark_generate_metatags,
    benchmark_escape_html,
    benchmark_unescape_html
);
criterion_main!(benches);
