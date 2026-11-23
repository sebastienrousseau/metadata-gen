// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Drives the MCP example server in process, as a client would.

#![cfg(all(feature = "yaml", feature = "json", feature = "html"))]

#[path = "../examples/mcp/server.rs"]
mod server;

use serde_json::{json, Value};

fn call(request: &Value) -> Value {
    let reply =
        server::handle_line(&request.to_string()).expect("a reply");
    serde_json::from_str(&reply).expect("valid JSON")
}

fn tool(name: &str, arguments: Value) -> Value {
    let reply = call(&json!({
        "jsonrpc": "2.0", "id": 7, "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    }));
    assert_eq!(reply["id"], 7);
    reply["result"].clone()
}

#[test]
fn initialize_and_list_tools() {
    let init = call(
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    assert_eq!(
        init["result"]["protocolVersion"],
        server::PROTOCOL_VERSION
    );
    assert_eq!(init["result"]["serverInfo"]["name"], "metadata-gen");

    let note = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    assert!(server::handle_line(&note.to_string()).is_none());

    let list = call(
        &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
    );
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["extract_metadata", "extract_meta_tags", "escape_html"]
    );
    assert_eq!(server::tools().as_array().unwrap().len(), 3);
}

#[test]
fn extract_metadata_tool_returns_the_map() {
    let result = tool(
        "extract_metadata",
        json!({ "content": "---\ntitle: Hi\nauthor:\n  name: Ada\n---\nbody" }),
    );
    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["metadata"]["title"], "Hi");
    assert_eq!(
        result["structuredContent"]["metadata"]["author.name"],
        "Ada"
    );
}

#[test]
fn malformed_front_matter_is_a_structured_tool_error() {
    let result =
        tool("extract_metadata", json!({ "content": "{\"a\": }" }));
    assert_eq!(result["isError"], true);
    let error = &result["structuredContent"]["error"];
    assert_eq!(error["kind"], "parse");
    assert!(error["span"].is_array());
    assert!(error["message"].as_str().unwrap().contains("json"));

    let none =
        tool("extract_metadata", json!({ "content": "plain text" }));
    assert_eq!(
        none["structuredContent"]["error"]["kind"],
        "extraction"
    );
}

#[test]
fn meta_tags_and_escape_tools() {
    let tags = tool(
        "extract_meta_tags",
        json!({ "html": "<meta name=\"description\" content=\"A &amp; B\">" }),
    );
    assert_eq!(
        tags["structuredContent"]["tags"][0]["content"],
        "A & B"
    );
    let escaped = tool("escape_html", json!({ "text": "<b>" }));
    assert_eq!(escaped["structuredContent"]["escaped"], "&lt;b&gt;");
}

#[test]
fn protocol_errors() {
    let unknown =
        call(&json!({ "jsonrpc": "2.0", "id": 3, "method": "nope" }));
    assert_eq!(unknown["error"]["code"], -32601);
    let bad_args = call(
        &json!({ "jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": { "name": "escape_html", "arguments": {} } }),
    );
    assert_eq!(bad_args["error"]["code"], -32602);
    let bad_tool = call(
        &json!({ "jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": { "name": "rm_rf" } }),
    );
    assert_eq!(bad_tool["error"]["code"], -32602);
    let garbage: Value = serde_json::from_str(
        &server::handle_line("{not json").unwrap(),
    )
    .unwrap();
    assert_eq!(garbage["error"]["code"], -32700);
    let ping =
        call(&json!({ "jsonrpc": "2.0", "id": 6, "method": "ping" }));
    assert_eq!(ping["result"], json!({}));
}
