// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A minimal Model Context Protocol server over stdio: one JSON-RPC 2.0
//! message per line in, one per line out. Shared by
//! `examples/mcp_server.rs` (the binary) and `tests/test_mcp.rs` (which
//! drives it in process).

use metadata_gen::metatags::extract_meta_tags;
use metadata_gen::{escape_html, extract_metadata, MetadataError};
use serde_json::{json, Map, Value};

/// The MCP protocol revision this server implements.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// Handles one line of input. Returns the line to write back, or `None`
/// for a notification, which gets no reply.
pub fn handle_line(line: &str) -> Option<String> {
    let reply = match serde_json::from_str::<Value>(line) {
        Ok(request) => handle(&request)?,
        Err(e) => rpc_error(
            &Value::Null,
            -32700,
            &format!("parse error: {e}"),
        ),
    };
    Some(reply.to_string())
}

fn handle(request: &Value) -> Option<Value> {
    let id = request.get("id")?.clone();
    let method =
        request.get("method").and_then(Value::as_str).unwrap_or("");
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => Ok(initialize()),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => call_tool(&params),
        other => Err((-32601, format!("method not found: {other}"))),
    };
    Some(match result {
        Ok(result) => {
            json!({ "jsonrpc": "2.0", "id": id, "result": result })
        }
        Err((code, message)) => rpc_error(&id, code, &message),
    })
}

fn rpc_error(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn initialize() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "metadata-gen", "version": env!("CARGO_PKG_VERSION") },
    })
}

fn string_input(name: &str, description: &str) -> Value {
    json!({
        "type": "object",
        "properties": { name: { "type": "string", "description": description } },
        "required": [name],
    })
}

/// The advertised tools, with JSON Schemas for their arguments.
pub fn tools() -> Value {
    json!([
        {
            "name": "extract_metadata",
            "description": "Parse YAML (---), TOML (+++) or JSON front matter at the top of a document into a flat map with dotted keys (author.name).",
            "inputSchema": string_input("content", "The whole document, front matter first."),
        },
        {
            "name": "extract_meta_tags",
            "description": "List every <meta> tag in an HTML document as name and content.",
            "inputSchema": string_input("html", "An HTML document or fragment."),
        },
        {
            "name": "escape_html",
            "description": "Escape &, <, >, \" and ' for HTML text and attributes.",
            "inputSchema": string_input("text", "Text to escape."),
        },
    ])
}

fn argument<'a>(
    params: &'a Value,
    name: &str,
) -> Result<&'a str, (i64, String)> {
    params
        .get("arguments")
        .and_then(|a| a.get(name))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            (-32602, format!("missing string argument `{name}`"))
        })
}

fn call_tool(params: &Value) -> Result<Value, (i64, String)> {
    let tool = params.get("name").and_then(Value::as_str).unwrap_or("");
    match tool {
        "extract_metadata" => Ok(extract(argument(params, "content")?)),
        "extract_meta_tags" => {
            let tags = extract_meta_tags(argument(params, "html")?)
                .unwrap_or_default();
            let list: Vec<Value> = tags
                .into_iter()
                .map(
                    |t| json!({ "name": t.name, "content": t.content }),
                )
                .collect();
            Ok(success(json!({ "tags": list })))
        }
        "escape_html" => Ok(success(
            json!({ "escaped": escape_html(argument(params, "text")?) }),
        )),
        other => Err((-32602, format!("unknown tool: {other}"))),
    }
}

fn extract(content: &str) -> Value {
    match extract_metadata(content) {
        Ok(metadata) => {
            let map: Map<String, Value> = metadata
                .into_inner()
                .into_iter()
                .map(|(k, v)| (k, Value::String(v)))
                .collect();
            success(json!({ "metadata": map }))
        }
        Err(e) => tool_error(&e),
    }
}

/// A successful tool result: the JSON as text for the model, and the
/// same value as `structuredContent` for clients that read it.
fn success(value: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": value.to_string() }],
        "structuredContent": value,
        "isError": false,
    })
}

/// A failed tool call, reported in the result (as MCP asks, so the model
/// sees it) with the `MetadataError` kind, message and byte span.
fn tool_error(error: &MetadataError) -> Value {
    let span = match error {
        MetadataError::Parse { span: Some(s), .. } => {
            json!([s.start, s.end])
        }
        _ => Value::Null,
    };
    let detail = json!({ "kind": error_kind(error), "message": error.to_string(), "span": span });
    json!({
        "content": [{ "type": "text", "text": error.to_string() }],
        "structuredContent": { "error": detail },
        "isError": true,
    })
}

fn error_kind(error: &MetadataError) -> &'static str {
    match error {
        MetadataError::Parse { .. } => "parse",
        MetadataError::ExtractionError { .. } => "extraction",
        MetadataError::UnsupportedFormatError(_) => {
            "unsupported_format"
        }
        MetadataError::MissingFieldError(_) => "missing_field",
        MetadataError::DateParseError(_) => "date_parse",
        _ => "other",
    }
}
