#!/usr/bin/env python3
"""Scan the release worktree for high-confidence credential material."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MAX_TEXT_BYTES = 5 * 1024 * 1024


PATTERNS = {
    "private_key": re.compile(
        rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----.*?"
        rb"-----END (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
        re.DOTALL,
    ),
    "openai_key": re.compile(
        rb"(?:sk-proj-[A-Za-z0-9_-]{20,}|sk-[A-Za-z0-9_]{20,})"
    ),
    "anthropic_key": re.compile(rb"sk-ant-[A-Za-z0-9_-]{20,}"),
    "github_token": re.compile(rb"gh[pousr]_[A-Za-z0-9]{20,}"),
    "aws_access_key": re.compile(rb"AKIA[0-9A-Z]{16}"),
    "slack_token": re.compile(rb"xox[baprs]-[A-Za-z0-9-]{20,}"),
    "google_api_key": re.compile(rb"AIza[0-9A-Za-z_-]{35}"),
}

# Exact fingerprints of committed inert fixtures used by redaction, identity,
# and credential-broker tests. A new value in the same file is still a finding.
ALLOWED_FIXTURE_MATCHES: set[tuple[str, str, str]] = {
    (
        "codex-rs/agent-identity/src/lib.rs",
        "private_key",
        "58c4f9f9bee0e0f8369340de944595ffcdb50d9fbb4f01ce3ef1945155fcbfdb",
    ),
    (
        "codex-rs/login/src/auth/agent_identity.rs",
        "private_key",
        "58c4f9f9bee0e0f8369340de944595ffcdb50d9fbb4f01ce3ef1945155fcbfdb",
    ),
    (
        "codex-rs/login/src/auth/auth_tests.rs",
        "private_key",
        "58c4f9f9bee0e0f8369340de944595ffcdb50d9fbb4f01ce3ef1945155fcbfdb",
    ),
    (
        "codex-rs/memories/write/src/phase1.rs",
        "openai_key",
        "82be8a4d9cdebab78235e6d0618fdea34065fcb6f498073283ff4ec7376e551a",
    ),
    (
        "codex-rs/network-proxy/src/credential_broker_tests.rs",
        "openai_key",
        "eb09e54db52a4ba24556deea3efe50cdcaa7a4833a584064b811baf7ebd1b61a",
    ),
    (
        "codex-rs/network-proxy/src/credential_broker_tests.rs",
        "openai_key",
        "3193831645bd691450f50c575c8f3019bcd83b9be8448321f223ef36671e03c5",
    ),
    (
        "codex-rs/network-proxy/src/credential_broker_tests.rs",
        "github_token",
        "faaa7b43e631a9e9a3242da86d987ec050a9f0db84884dbddce714774b2f761c",
    ),
}


def worktree_paths() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    paths = []
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        relative = Path(raw.decode("utf-8", errors="strict"))
        if relative.parts[:4] == ("qa", "release", "0.1.27", "artifacts"):
            continue
        paths.append(relative)
    return sorted(paths)


def scan() -> dict[str, object]:
    findings: list[dict[str, object]] = []
    fixture_matches: list[dict[str, object]] = []
    scanned = 0
    skipped_binary_or_large = 0
    for relative in worktree_paths():
        path = ROOT / relative
        if not path.is_file():
            continue
        size = path.stat().st_size
        if size > MAX_TEXT_BYTES:
            skipped_binary_or_large += 1
            continue
        data = path.read_bytes()
        if b"\0" in data:
            skipped_binary_or_large += 1
            continue
        scanned += 1
        for kind, pattern in PATTERNS.items():
            for match in pattern.finditer(data):
                fingerprint = hashlib.sha256(match.group()).hexdigest()
                item = {
                    "kind": kind,
                    "path": relative.as_posix(),
                    "line": data.count(b"\n", 0, match.start()) + 1,
                    "sha256": fingerprint,
                }
                identity = (relative.as_posix(), kind, fingerprint)
                if identity in ALLOWED_FIXTURE_MATCHES:
                    fixture_matches.append(item)
                else:
                    findings.append(item)
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout.strip()
    return {
        "head": head,
        "files_scanned": scanned,
        "binary_or_large_files_skipped": skipped_binary_or_large,
        "patterns": sorted(PATTERNS),
        "finding_count": len(findings),
        "findings": findings,
        "reviewed_fixture_match_count": len(fixture_matches),
        "reviewed_fixture_matches": fixture_matches,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fail-on-finding", action="store_true")
    args = parser.parse_args()
    report = scan()
    print(json.dumps(report, indent=2, sort_keys=True))
    if args.fail_on_finding and report["finding_count"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
