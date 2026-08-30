#!/usr/bin/env python3
"""Fail when the PF-34-S04 narrative and lane hash ledger diverge."""

import hashlib
from pathlib import Path


HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[3]
EVIDENCE = (HERE / "evidence.md").read_text(encoding="utf-8")
EXPECTED_CANDIDATE = "a75efecc0a37d5544e123ad19d57867cac360a68"
EXPECTED_REVIEW_PACKET = "3813e9783ddbf09fb9e2bdbb16fa9600adeb62b58fcd09385bf6328089bc3389"
NARRATED_FILES = {
    "codex-rs/content-security/src/contract.rs",
    "codex-rs/content-security/src/contract_tests.rs",
}


def main() -> None:
    ledger: dict[str, str] = {}
    for line in (HERE / "lane-files.sha256").read_text(encoding="utf-8").splitlines():
        digest, relative = line.split("  ", maxsplit=1)
        actual = hashlib.sha256((REPOSITORY / relative).read_bytes()).hexdigest()
        if actual != digest:
            raise SystemExit(f"PF-34-S04 hash mismatch: {relative}")
        ledger[relative] = digest

    for relative in NARRATED_FILES:
        if ledger[relative] not in EVIDENCE:
            raise SystemExit(f"PF-34-S04 evidence omits current hash: {relative}")
    for required in (EXPECTED_CANDIDATE, EXPECTED_REVIEW_PACKET, "14 passed"):
        if required not in EVIDENCE:
            raise SystemExit(f"PF-34-S04 evidence omits current identity: {required}")
    print(f"PF-34-S04 evidence: {len(ledger)} hashes and narrative identities agree")


if __name__ == "__main__":
    main()
