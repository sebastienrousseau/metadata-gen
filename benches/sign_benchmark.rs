// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ML-DSA-65 sign and verify over a typical front-matter block
//! (feature `pqc-sign`). Run with
//! `cargo bench --bench sign_benchmark --features pqc-sign`.

use criterion::{criterion_group, criterion_main, Criterion};
use metadata_gen::extract_metadata;
use metadata_gen::sign::{ml_dsa::Keypair, signing_key_from_seed};
use std::hint::black_box;

const PAGE: &str = "---
title: Benchmark Test Page
description: A test page for benchmarking signatures
keywords: benchmark, metadata, signature
date: 2026-11-23
author:
  name: Ada
  url: https://example.com/
---
# Body
";

fn bench_sign_verify(c: &mut Criterion) {
    let key = signing_key_from_seed(&[5u8; 32]);
    let public = key.verifying_key();
    let metadata = extract_metadata(PAGE).expect("valid front matter");
    let signature = metadata.sign(&key).expect("signs");

    let mut group = c.benchmark_group("ml_dsa_65");
    group.bench_function("sign", |b| {
        b.iter(|| black_box(&metadata).sign(black_box(&key)))
    });
    group.bench_function("verify", |b| {
        b.iter(|| {
            black_box(&metadata)
                .verify(black_box(&public), black_box(&signature))
        })
    });
    group.finish();
}

criterion_group!(benches, bench_sign_verify);
criterion_main!(benches);
