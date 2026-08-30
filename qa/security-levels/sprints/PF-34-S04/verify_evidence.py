#!/usr/bin/env python3
"""Fail when the PF-34-S04 candidate narrative and hash ledger diverge.

Run this snapshot guard again at G1. Retire it only when the sprint is archived;
combined-tree evidence may add identities but must retain these lane identities.
"""

import hashlib
from pathlib import Path, PurePosixPath


HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[3]
EVIDENCE = (HERE / "evidence.md").read_text(encoding="utf-8")
EXPECTED_CANDIDATE = "a75efecc0a37d5544e123ad19d57867cac360a68"
EXPECTED_REVIEW_PACKET = "3813e9783ddbf09fb9e2bdbb16fa9600adeb62b58fcd09385bf6328089bc3389"
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
        f"Final Opus-remediated implementation: `{EXPECTED_CANDIDATE}`",
        f"Third full review packet SHA-256: `{EXPECTED_REVIEW_PACKET}`",
        "14 passed, 0 failed",
    ):
        if required not in EVIDENCE:
            raise SystemExit(f"PF-34-S04 evidence omits current identity: {required}")
    print(f"PF-34-S04 evidence: {len(ledger)} hashes and narrative identities agree")


if __name__ == "__main__":
    main()
