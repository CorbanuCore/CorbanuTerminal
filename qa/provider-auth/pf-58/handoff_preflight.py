#!/usr/bin/env python3
"""Read-only package/launcher prerequisite gate. Never read credential values."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def inspect(candidate, runtime_path, configs):
    checks = []

    def record(name, ok, detail):
        checks.append({"check": name, "passed": bool(ok), "detail": detail})

    # Match the released package-local canonical/legacy daemon lookup.
    canonical_wallet = candidate.with_name("corbanu-walletd")
    legacy_wallet = candidate.with_name("pfterminal-walletd")
    wallet = (
        canonical_wallet
        if canonical_wallet.is_file() or not legacy_wallet.is_file()
        else legacy_wallet
    )
    for binary in [
        candidate,
        candidate.with_name("codex-code-mode-host"),
        wallet,
    ]:
        ok = binary.is_file() and os.access(binary, os.X_OK)
        record(
            binary.name,
            ok,
            {"path": str(binary), "sha256": digest(binary) if ok else None},
        )
    if checks[0]["passed"]:
        try:
            version = subprocess.run(
                [str(candidate), "--version"],
                capture_output=True,
                text=True,
                timeout=15,
                check=True,
            ).stdout.strip()
            record("candidate starts", True, version)
        except (OSError, subprocess.SubprocessError):
            record("candidate starts", False, "version command failed")
    for file in configs:
        # Explicitly supplied installed configurations only. Cached but disabled
        # plugins must not be mistaken for the launcher's effective server set.
        try:
            config = json.loads(file.read_text())
            servers = config["mcpServers"]
            if not isinstance(servers, dict) or not servers:
                raise ValueError("missing server map")
            for name, server in servers.items():
                if not isinstance(server, dict):
                    raise ValueError("invalid server configuration")
                command = server.get("command")
                if not isinstance(command, str) or not command:
                    record(
                        name,
                        False,
                        "not a supported local command; separately validate remote MCP",
                    )
                    continue
                cwd = Path(server.get("cwd", "."))
                if not cwd.is_absolute():
                    cwd = file.parent / cwd
                resolved = (
                    str(cwd / command)
                    if "/" in command and not Path(command).is_absolute()
                    else shutil.which(command, path=runtime_path)
                )
                ok = cwd.is_dir() and resolved and os.access(resolved, os.X_OK)
                # Do not serialize args, env, headers, tokens or command output.
                record(
                    name,
                    ok,
                    {
                        "command": command,
                        "resolved": resolved,
                        "cwd_exists": cwd.is_dir(),
                    },
                )
        except (OSError, ValueError, KeyError, TypeError):
            record(
                str(file), False, "invalid or unreadable installed MCP configuration"
            )
    return {"passed": all(c["passed"] for c in checks), "checks": checks}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--mcp-config", type=Path, action="append", default=[])
    parser.add_argument(
        "--launch-pid",
        type=int,
        help="Linux: inspect actual launch PATH, not SSH/build PATH",
    )
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    runtime_path = os.environ.get("PATH", "")
    if args.launch_pid:
        # Only PATH is retained; never print or write the process environment.
        entries = Path(f"/proc/{args.launch_pid}/environ").read_bytes().split(b"\0")
        runtime_path = next(e[5:].decode() for e in entries if e.startswith(b"PATH="))
    report = inspect(args.candidate.absolute(), runtime_path, args.mcp_config)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(
        "PASS"
        if report["passed"]
        else "BLOCKED: package or launcher dependencies are missing"
    )
    raise SystemExit(0 if report["passed"] else 1)


if __name__ == "__main__":
    main()
