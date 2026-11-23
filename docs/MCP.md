<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# MCP server example

[`examples/mcp_server.rs`](../examples/mcp_server.rs) is a Model
Context Protocol server that gives an AI agent three tools backed by
this crate. It speaks JSON-RPC 2.0 over stdio, one message per line,
protocol revision `2025-06-18`, and has no dependency beyond the crate
and `serde_json`. It is an example, not a published binary: build it
from a checkout.

```bash
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen
cargo build --release --example mcp_server
# binary: target/release/examples/mcp_server
```

## Register it

Claude Code:

```bash
claude mcp add metadata-gen -- "$PWD/target/release/examples/mcp_server"
```

Claude Desktop and other clients that read an `mcpServers` map
(`claude_desktop_config.json` for Claude Desktop):

```json
{
  "mcpServers": {
    "metadata-gen": {
      "command": "/absolute/path/to/metadata-gen/target/release/examples/mcp_server"
    }
  }
}
```

## Tools

| Tool | Argument | Result (`structuredContent`) |
| :--- | :--- | :--- |
| `extract_metadata` | `content`: the document | `{"metadata": {"title": "...", "author.name": "..."}}` |
| `extract_meta_tags` | `html`: an HTML document | `{"tags": [{"name": "...", "content": "..."}]}` |
| `escape_html` | `text` | `{"escaped": "..."}` |

Each argument schema is `{"type": "object", "properties": {<name>:
{"type": "string"}}, "required": [<name>]}`; `tools/list` returns them
with descriptions. Every result also carries the same JSON as a text
content block, for clients that ignore `structuredContent`.

## Errors

A document the crate rejects is a tool error, reported inside the
result as MCP asks (so the model sees it), not as a JSON-RPC error:

```json
{
  "content": [{ "type": "text", "text": "json front matter failed to parse at bytes 6..7: ..." }],
  "structuredContent": {
    "error": { "kind": "parse", "message": "json front matter failed to parse ...", "span": [6, 7] }
  },
  "isError": true
}
```

`kind` is one of `parse`, `extraction` (no front matter found),
`unsupported_format`, `missing_field`, `date_parse` or `other`;
`span` is the byte range inside the front-matter block when the parser
reported one, otherwise `null`.

Protocol problems are JSON-RPC errors: `-32700` for a line that is not
JSON, `-32601` for an unknown method, `-32602` for an unknown tool or
a missing argument.

## Try it without a client

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"extract_metadata","arguments":{"content":"---\ntitle: Hi\n---\n"}}}' \
  | target/release/examples/mcp_server
```

## Tests

[`tests/test_mcp.rs`](../tests/test_mcp.rs) drives the same handler
in process: `initialize`, the `initialized` notification, `tools/list`,
each tool, a malformed document, and each protocol error. `make
examples` builds and runs every example in CI.
