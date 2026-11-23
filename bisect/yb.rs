use metadata_gen::metadata::extract_metadata;
use std::hint::black_box;
use std::time::Instant;

fn yaml_block_document(bytes: usize) -> String {
    let mut doc = String::with_capacity(bytes + 64);
    doc.push_str("---\ntitle: Throughput\ndate: 2024-01-02\n");
    let mut i = 0;
    while doc.len() + 40 < bytes {
        doc.push_str(&format!("key_{i}: value number {i} with some text\n"));
        i += 1;
    }
    doc.push_str("---\n# Body\n");
    doc
}
fn yaml_document(bytes: usize) -> String {
    let mut doc = yaml_block_document(1 << 10);
    let line = "Body text that the delimiter scan must not read. ---\n";
    while doc.len() + line.len() < bytes {
        doc.push_str(line);
    }
    doc
}
fn time(doc: &str, target_ns: u128) -> f64 {
    // calibrate
    let mut n = 1u64;
    loop {
        let t = Instant::now();
        for _ in 0..n { drop(black_box(extract_metadata(black_box(doc)).unwrap())); }
        if t.elapsed().as_nanos() > target_ns / 10 { break; }
        n *= 2;
    }
    let mut v = Vec::new();
    for _ in 0..15 {
        let t = Instant::now();
        for _ in 0..n { drop(black_box(extract_metadata(black_box(doc)).unwrap())); }
        v.push(t.elapsed().as_nanos() as f64 / n as f64);
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}
fn main() {
    let small = "---\ntitle: Benchmark Test Page\ndescription: A test page for benchmarking metadata extraction\nkeywords: benchmark, metadata, extraction\n---\n# Benchmark Content";
    let cases: Vec<(&str, String)> = vec![
        ("small", small.to_string()),
        ("block1k", yaml_block_document(1 << 10)),
        ("block10k", yaml_block_document(10 << 10)),
        ("block32k", yaml_block_document(32 << 10)),
        ("doc1m", yaml_document(1 << 20)),
    ];
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 4 && args[1] == "count" {
        let doc = &cases.iter().find(|c| c.0 == args[2]).unwrap().1;
        let n: u64 = args[3].parse().unwrap();
        for _ in 0..n { drop(black_box(extract_metadata(black_box(doc)).unwrap())); }
        return;
    }
    let only = args.get(1).cloned();
    for (name, doc) in &cases {
        if let Some(o) = &only { if o != name { continue; } }
        println!("{name} {:.1}", time(doc, 200_000_000));
    }
}
