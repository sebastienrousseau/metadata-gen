// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Prints, in WAVE notation, what the component must return for one
//! input, computed natively. `check.py` compares it with the output of
//! `wasmtime run --invoke` on the component.
//!
//! Usage: `expect <extract|extract-meta-tags|escape-html> <input-file>`

use metadata_gen_component::core;
use std::fmt::Write as _;

fn wave_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn wave_records(pairs: &[(String, String)], k: &str, v: &str) -> String {
    let items: Vec<String> = pairs
        .iter()
        .map(|(a, b)| {
            format!("{{{k}: {}, {v}: {}}}", wave_string(a), wave_string(b))
        })
        .collect();
    format!("[{}]", items.join(", "))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, func, path] = args.as_slice() else {
        eprintln!("usage: expect <function> <input-file>");
        std::process::exit(2);
    };
    let input = std::fs::read_to_string(path).expect("readable input file");
    let out = match func.as_str() {
        "extract" => match core::extract(&input) {
            Ok(entries) => {
                format!("ok({})", wave_records(&entries, "key", "value"))
            }
            Err(message) => format!("err({})", wave_string(&message)),
        },
        "extract-meta-tags" => wave_records(
            &core::extract_meta_tags(&input),
            "name",
            "content",
        ),
        "escape-html" => wave_string(&core::escape_html(&input)),
        other => {
            eprintln!("unknown function {other}");
            std::process::exit(2);
        }
    };
    println!("{out}");
}
