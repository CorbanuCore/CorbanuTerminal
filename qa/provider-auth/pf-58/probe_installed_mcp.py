#!/usr/bin/env python3
"""Initialize explicitly selected installed stdio MCPs without logging payloads."""

import argparse
import json
import os
from pathlib import Path
import select
import subprocess
import time


def exchange(process, message):
    process.stdin.write(json.dumps(message) + "\n")
    process.stdin.flush()
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if not select.select([process.stdout], [], [], 1)[0]:
            continue
        line = process.stdout.readline()
        if not line:
            raise RuntimeError("server exited before response")
        response = json.loads(line)
        if response.get("id") == message["id"]:
            if "error" in response:
                raise RuntimeError("server rejected request")
            return response["result"]
    raise TimeoutError("MCP response timed out")


def probe(config):
    outcomes = []
    for name, server in json.loads(config.read_text())["mcpServers"].items():
        cwd = Path(server.get("cwd", "."))
        if not cwd.is_absolute():
            cwd = config.parent / cwd
        env = dict(os.environ)
        env.update(server.get("env", {}))
        process = None
        try:
            process = subprocess.Popen(
                [server["command"], *server.get("args", [])],
                cwd=cwd,
                env=env,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                bufsize=1,
            )
            initialized = exchange(
                process,
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2025-03-26",
                        "capabilities": {},
                        "clientInfo": {"name": "candidate-preflight", "version": "1"},
                    },
                },
            )
            process.stdin.write(
                '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
            )
            process.stdin.flush()
            result = exchange(
                process,
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
            )
            tools = result.get("tools", [])
            outcomes.append(
                {
                    "server": name,
                    "passed": bool(initialized.get("serverInfo")) and bool(tools),
                    "tool_count": len(tools),
                }
            )
        except (OSError, ValueError, KeyError, RuntimeError, TimeoutError):
            outcomes.append(
                {
                    "server": name,
                    "passed": False,
                    "error": "initialize or tools/list failed",
                }
            )
        finally:
            if process is not None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
    return outcomes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mcp-config", type=Path, action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    results = [item for config in args.mcp_config for item in probe(config)]
    report = {
        "passed": bool(results) and all(item["passed"] for item in results),
        "servers": results,
    }
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))
    raise SystemExit(0 if report["passed"] else 1)


if __name__ == "__main__":
    main()
