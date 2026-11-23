// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! An MCP (Model Context Protocol) server over stdio that exposes
//! `extract_metadata`, `extract_meta_tags` and `escape_html` as tools.
//!
//! Build it with `cargo build --release --example mcp_server`, then
//! register `target/release/examples/mcp_server` with an MCP client;
//! see docs/MCP.md. Each line on stdin is one JSON-RPC message; each
//! reply is one line on stdout.

use std::io::{self, BufRead, Write};

#[path = "mcp/server.rs"]
mod server;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = server::handle_line(&line) {
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}
