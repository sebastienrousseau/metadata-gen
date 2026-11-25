// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Fixed workloads for the instruction-count gate (#49).
//!
//! `scripts/instruction_compare.py` runs this binary under Valgrind's
//! callgrind once per workload, on the base commit's library and on the
//! change's, and compares the instructions executed. Instruction counts
//! do not depend on how busy the runner is, so the gate can be tight
//! where wall-clock timings on a shared runner cannot.
//!
//! Usage: `instructions <workload>` runs one workload a fixed number of
//! times on fixed input; `instructions --list` prints the workload names.
//! An unknown name exits with status 2, which the comparison reads as
//! "not in this build" (a workload the base commit does not have).
//! `cargo bench` (which passes `--bench`) runs nothing.

use metadata_gen::{
    extract_and_prepare_metadata,
    metadata::{extract_metadata, process_metadata, Metadata},
    metatags::{extract_meta_tags, generate_metatags},
    utils::{escape_html, unescape_html},
    MetadataMap,
};
use std::hint::black_box;
use std::process::ExitCode;

/// A workload: its name and the function that runs it once.
type Workload = (&'static str, fn());

const WORKLOADS: [Workload; 9] = [
    ("escape_html_64k", escape_64k),
    ("unescape_html_64k", unescape_64k),
    ("extract_yaml_1k", || extract(&yaml_header(1 << 10))),
    ("extract_yaml_32k", || extract(&yaml_header(32 << 10))),
    ("extract_toml_1k", || extract(&toml_header())),
    ("extract_json_1k", || extract(&json_header())),
    ("extract_and_prepare_1k", extract_and_prepare_1k),
    ("extract_meta_tags_64k", meta_tags_64k),
    ("process_and_generate", process_and_generate),
];

/// How many times each workload's inner loop runs: enough that the
/// process start-up the count also includes is a small share of it.
const REPEAT: usize = 20;

fn main() -> ExitCode {
    // `cargo bench` runs every bench target with `--bench` (after any
    // arguments it forwards); the Criterion steps reach this one too,
    // and it has nothing to time.
    if std::env::args().any(|a| a == "--bench") {
        return ExitCode::SUCCESS;
    }
    let Some(arg) = std::env::args().nth(1) else {
        eprintln!("usage: instructions <workload> | --list");
        return ExitCode::from(2);
    };
    if arg == "--list" {
        for (name, _) in WORKLOADS {
            println!("{name}");
        }
        return ExitCode::SUCCESS;
    }
    match WORKLOADS.iter().find(|(name, _)| *name == arg) {
        Some((_, run)) => {
            run();
            ExitCode::SUCCESS
        }
        None => ExitCode::from(2),
    }
}

/// `escape_html` input: seven escapable bytes in every 27.
fn escapable(bytes: usize) -> String {
    "a <b> & \"c\" 'd' plain text ".repeat(bytes / 28 + 1)
}

fn escape_64k() {
    let text = escapable(64 << 10);
    for _ in 0..REPEAT {
        drop(black_box(escape_html(black_box(&text))));
    }
}

fn unescape_64k() {
    let text = escape_html(&escapable(64 << 10));
    for _ in 0..REPEAT {
        drop(black_box(unescape_html(black_box(&text))));
    }
}

/// A YAML front-matter block of about `bytes` bytes, then a short body.
fn yaml_header(bytes: usize) -> String {
    let mut doc =
        String::from("---\ntitle: Instructions\ndate: 2024-01-02\n");
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

fn toml_header() -> String {
    let mut doc = String::from("+++\ntitle = \"Instructions\"\n");
    for i in 0..24 {
        doc.push_str(&format!(
            "key_{i} = \"value number {i} with some text\"\n"
        ));
    }
    doc.push_str("+++\n# Body\n");
    doc
}

fn json_header() -> String {
    let fields: Vec<String> = (0..24)
        .map(|i| {
            format!("\"key_{i}\": \"value number {i} with some text\"")
        })
        .collect();
    format!(
        "{{\"title\": \"Instructions\", {}}}\n# Body\n",
        fields.join(", ")
    )
}

fn extract(doc: &str) {
    for _ in 0..REPEAT {
        drop(black_box(extract_metadata(black_box(doc)).unwrap()));
    }
}

fn extract_and_prepare_1k() {
    let doc = yaml_header(1 << 10);
    for _ in 0..REPEAT {
        drop(black_box(
            extract_and_prepare_metadata(black_box(&doc)).unwrap(),
        ));
    }
}

fn meta_tags_64k() {
    let mut html = String::from("<html><head>");
    let mut i = 0;
    while html.len() + 80 < 64 << 10 {
        html.push_str(&format!(
            "<meta name=\"field{i}\" content=\"value &amp; number {i}\">"
        ));
        i += 1;
    }
    html.push_str("</head><body></body></html>");
    for _ in 0..REPEAT {
        drop(black_box(extract_meta_tags(black_box(&html))));
    }
}

fn process_and_generate() {
    let mut map = MetadataMap::new();
    for (key, value) in [
        ("title", "Instructions"),
        ("description", "Fixed input for the instruction-count gate"),
        ("date", "2023-05-25"),
        ("keywords", "a, b, c"),
        ("og:title", "Instructions"),
        ("twitter:card", "summary"),
        ("author", "Ada"),
    ] {
        map.insert(key.to_string(), value.to_string());
    }
    let metadata = Metadata::new(map.clone());
    for _ in 0..REPEAT {
        drop(black_box(
            process_metadata(black_box(&metadata)).unwrap(),
        ));
        drop(black_box(generate_metatags(black_box(&map))));
    }
}
