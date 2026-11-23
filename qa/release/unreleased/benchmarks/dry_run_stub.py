#!/usr/bin/env python3
"""No-model stand-in for a harness binary in dry-run configs.

make_run_configs.py --dry-run puts this script in place of each agent binary.
It prints one JSON line describing the argv it received (the prompt only as a
length and digest), whether the per-run state directories exist, and the
version of the real harness binary named by BENCH_DRY_RUN_TARGET, then exits 0.
It makes no model or provider call and prints no environment values except the
paths of the isolated state directories.

For the website-builder dry run, set BENCH_DRY_RUN_WEBSITE_MODEL to the frozen
model. The stub then writes the route evidence run_pair.py expects (the
Corbanu request dump, or Claude Code's JSON fields on stdout) so the verifier
and capture steps run. In the Claude Code lane (CLAUDE_CONFIG_DIR set) it never
starts a target binary: the real Claude Code must not run outside a
keychain-denying sandbox.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys

STATE_VARS = (
    "CODEX_HOME",
    "HERMES_HOME",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "XDG_STATE_HOME",
    "XDG_CACHE_HOME",
    "KILO_CONFIG",
    "CLAUDE_CONFIG_DIR",
)


def describe(arg: str) -> str:
    if len(arg) <= 200 and "\n" not in arg:
        return arg
    digest = hashlib.sha256(arg.encode()).hexdigest()
    return f"<text len={len(arg)} sha256={digest}>"


def main() -> int:
    stdin_text = "" if sys.stdin is None or sys.stdin.isatty() else sys.stdin.read()
    target = os.environ.get("BENCH_DRY_RUN_TARGET", "")
    website_model = os.environ.get("BENCH_DRY_RUN_WEBSITE_MODEL", "")
    claude_lane = "CLAUDE_CONFIG_DIR" in os.environ
    if claude_lane:
        target = ""
    version = None
    if target:
        completed = subprocess.run(
            [target, "--version"],
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
        version = (completed.stdout or completed.stderr).strip().splitlines()[
            :8
        ] or None
    record = {
        "dry_run": True,
        "argv": [describe(arg) for arg in sys.argv[1:]],
        "stdin": describe(stdin_text) if stdin_text else None,
        "cwd": os.getcwd(),
        "state": {
            name: {"path": os.environ[name], "exists": os.path.exists(os.environ[name])}
            for name in STATE_VARS
            if name in os.environ
        },
        "target": target or None,
        "target_version": version,
    }
    if website_model:
        dump = os.environ.get("CORBANU_DUMP_ANTHROPIC_REQUEST")
        if dump and not claude_lane:
            with open(dump, "w", encoding="utf-8") as handle:
                json.dump({"model": website_model, "dry_run": True}, handle)
        record["modelUsage"] = {website_model: {}}
        record["total_cost_usd"] = 0
    print(json.dumps(record, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
