#!/usr/bin/env python3
"""Local, credential-free MCP used by the packaged TUI handoff smoke."""

import json
import sys


for line in sys.stdin:
    request = json.loads(line)
    if "id" not in request:
        continue
    method = request["method"]
    if method == "initialize":
        result = {
            "protocolVersion": request["params"]["protocolVersion"],
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "handoff", "version": "1"},
        }
    elif method == "tools/list":
        result = {
            "tools": [
                {
                    "name": "echo",
                    "description": "Return the synthetic handoff marker",
                    "inputSchema": {"type": "object", "properties": {}},
                }
            ]
        }
    elif method == "tools/call":
        result = {"content": [{"type": "text", "text": "MCP_HANDOFF_OK"}]}
    else:
        result = {}
    print(
        json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}),
        flush=True,
    )
