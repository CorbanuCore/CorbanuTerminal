#!/usr/bin/env python3
"""#1856: helpers for .github/workflows/windows-integration-tests.yml.

Usage:
  windows_integration_known_failures.py filter|summary <cargo package>
      The nextest filter excluding the package's known Windows failures
      (.github/windows-integration-known-failures.txt), or the job summary.
  windows_integration_known_failures.py packages < changed-paths (NUL- or newline-separated)
      `packages=<JSON array>`: the packages whose tests a PR changing these
      paths runs.
"""

import json
import re
import sys
from pathlib import Path
from typing import NamedTuple

LIST = Path(__file__).resolve().parents[1] / "windows-integration-known-failures.txt"
LINE = re.compile(r"^(?P<package>[a-z0-9-]+) (?P<test>[A-Za-z0-9_:]+) #(?P<issue>[0-9]+)$")


# Cargo package -> its directory. A change to one of the job's own files runs
# every package.
PACKAGES = {"codex-core": "codex-rs/core/", "codex-app-server": "codex-rs/app-server/"}
JOB_FILES = (
    ".github/workflows/windows-integration-tests.yml",
    ".github/windows-integration-known-failures.txt",
    ".github/scripts/windows_integration_known_failures.py",
    "codex-rs/.config/nextest.toml",
)


class KnownFailure(NamedTuple):
    package: str
    test: str
    issue: str


def load(path: Path = LIST) -> list[KnownFailure]:
    entries = []
    for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        match = LINE.match(line)
        if match is None:
            raise ValueError(f"{path.name}:{number}: expected '<package> <test> #<issue>': {raw!r}")
        entries.append(KnownFailure(**match.groupdict()))
    return entries


def nextest_filter(entries: list[KnownFailure], package: str) -> str:
    """A nextest filterset that runs every test except the package's known failures."""
    tests = [entry.test for entry in entries if entry.package == package]
    if not tests:
        return "all()"
    return "not (" + " | ".join(f"test(={test})" for test in tests) + ")"


def summary(entries: list[KnownFailure], package: str) -> str:
    excluded = [entry for entry in entries if entry.package == package]
    lines = [
        f"### {package}: integration tests (`tests/*.rs`) on native Windows (#1856)",
        "",
        "Non-blocking: blocking-ci.yml does not call this workflow, so `CI required`"
        " neither waits for nor depends on it.",
    ]
    if excluded:
        lines += [
            "",
            f"{len(excluded)} known Windows failures were excluded "
            "(`.github/windows-integration-known-failures.txt`):",
            "",
        ]
        lines += [f"- `{entry.test}` (#{entry.issue})" for entry in excluded]
    return "\n".join(lines) + "\n"


def changed_packages(paths: list[str]) -> list[str]:
    if any(path in JOB_FILES for path in paths):
        return list(PACKAGES)
    return [
        package
        for package, directory in PACKAGES.items()
        if any(path.startswith(directory) for path in paths)
    ]


def main(argv: list[str]) -> int:
    if len(argv) == 2 and argv[1] == "packages":
        # `git diff --name-only -z` output (newline-separated also works).
        paths = [path for path in re.split(r"[\0\n]", sys.stdin.read()) if path]
        sys.stdout.write(f"packages={json.dumps(changed_packages(paths))}\n")
        return 0
    if len(argv) != 3 or argv[1] not in ("filter", "summary"):
        print(__doc__, file=sys.stderr)
        return 2
    entries = load()
    if argv[1] == "filter":
        # No trailing newline: on Windows it would be "\r\n", and `$(...)`
        # strips only the "\n".
        sys.stdout.write(nextest_filter(entries, argv[2]))
    else:
        sys.stdout.write(summary(entries, argv[2]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
