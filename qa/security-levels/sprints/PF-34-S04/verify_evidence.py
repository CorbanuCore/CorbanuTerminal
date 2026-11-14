#!/usr/bin/env python3
"""Fail when the PF-34-S04 narrative and final-tree hash ledger diverge."""

import hashlib
from pathlib import Path, PurePosixPath


HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[3]
EVIDENCE = (HERE / "evidence.md").read_text(encoding="utf-8")
REVIEWED_INTEGRATION_COMMIT = "279ce48a9e8d3b28ab518ff184aae770d7462d2f"
INTEGRATION_REVIEW_PACKET = "5ebbb39bbea56a3cc69549f6239e7346e627584d5b261e4dee556d87c5c1c8f4"
NARRATED_FILES = {
    "Contract SHA-256": "codex-rs/content-security/src/contract.rs",
    "Contract tests SHA-256": "codex-rs/content-security/src/contract_tests.rs",
}


def main() -> None:
    ledger: dict[str, str] = {}
    for line in (HERE / "lane-files.sha256").read_text(encoding="utf-8").splitlines():
        if "  " not in line:
            raise SystemExit("PF-34-S04 ledger contains a malformed line")
        digest, relative = line.split("  ", maxsplit=1)
        if len(digest) != 64 or any(
            character not in "0123456789abcdef" for character in digest
        ):
            raise SystemExit(f"PF-34-S04 ledger digest is malformed: {digest}")
        if relative in ledger:
            raise SystemExit(f"PF-34-S04 ledger repeats a path: {relative}")
        relative_path = PurePosixPath(relative)
        if relative_path.is_absolute() or ".." in relative_path.parts:
            raise SystemExit(f"PF-34-S04 ledger path is unsafe: {relative}")
        path = REPOSITORY.joinpath(*relative_path.parts)
        if path.is_symlink() or not path.resolve().is_relative_to(REPOSITORY.resolve()):
            raise SystemExit(f"PF-34-S04 ledger path escapes the repository: {relative}")
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != digest:
            raise SystemExit(f"PF-34-S04 hash mismatch: {relative}")
        ledger[relative] = digest

    for label, relative in NARRATED_FILES.items():
        digest = ledger.get(relative)
        if digest is None:
            raise SystemExit(f"PF-34-S04 ledger omits narrated file: {relative}")
        if f"- {label}: `{digest}`" not in EVIDENCE:
            raise SystemExit(f"PF-34-S04 evidence omits current hash: {relative}")
    for required in (
        f"Registered integration checkpoint: `{REVIEWED_INTEGRATION_COMMIT}`",
        f"Integration review packet SHA-256: `{INTEGRATION_REVIEW_PACKET}`",
        "14 passed, 0 failed",
        "21 passed, 0 failed",
    ):
        if required not in EVIDENCE:
            raise SystemExit(f"PF-34-S04 evidence omits current identity: {required}")
    print(f"PF-34-S04 evidence: {len(ledger)} hashes and narrative identities agree")


if __name__ == "__main__":
    main()
