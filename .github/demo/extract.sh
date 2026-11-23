#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Used by .github/demo.tape: sends a file to the MCP example server's
# extract_metadata tool and prints the flat map it returns.
set -euo pipefail
python3 -I -c 'import json,sys; print(json.dumps({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"extract_metadata","arguments":{"content":open(sys.argv[1]).read()}}}))' "$1" \
  | ./target/debug/examples/mcp_server \
  | python3 -I -c 'import json,sys; print(json.dumps(json.load(sys.stdin)["result"]["structuredContent"]["metadata"], indent=2, sort_keys=True))'
