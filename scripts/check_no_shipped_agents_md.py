#!/usr/bin/env python3
"""Fail when the repository ships AGENTS.md content.

Corbanu Terminal must not ship a default AGENTS.md: any AGENTS.md inside the
source tree is auto-injected into every agent session run under the checkout
(project-doc discovery walks up to the git root), silently steering agents and
inflating context. Development policy lives in docs/development-policy.md,
docs/rust-development-policy.md, and docs/crate-notes/; the Telegram identity
lives in [telegram].identity_instructions.

Benchmark task fixtures are the only exception: their AGENTS.md files are
candidate-visible workspace content copied into isolated benchmark workspaces.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
# Benchmark task fixtures are candidate-visible workspace content copied into
# isolated benchmark workspaces; they are not shipped and not auto-injected.
ALLOWED_PREFIXES = ("benchmarks/",)
BANNED_NAMES = {"AGENTS.md", "AGENTS.override.md", "AGENTS.md.template"}


def tracked_files() -> list[str]:
    output = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "ls-files"],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout
    return output.splitlines()


def main() -> int:
    offenders = [
        path
        for path in tracked_files()
        if Path(path).name in BANNED_NAMES
        and not path.startswith(ALLOWED_PREFIXES)
    ]
    if offenders:
        print("shipped AGENTS.md content is not allowed:", file=sys.stderr)
        for path in offenders:
            print(f"  {path}", file=sys.stderr)
        print(
            "move policy to docs/development-policy.md or a skill; "
            "see scripts/check_no_shipped_agents_md.py",
            file=sys.stderr,
        )
        return 1
    print("no shipped AGENTS.md content found")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
