#!/usr/bin/env python3
"""Scan benchmark source and artifacts for exact credential values."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def candidate_files(root: Path):
    if root.is_file():
        yield root
        return
    for path in sorted(root.rglob("*")):
        if path.is_file() and not path.is_symlink():
            yield path


def contains_secret(path: Path, needles: list[bytes]) -> tuple[bool, int]:
    overlap = max(map(len, needles)) - 1
    tail = b""
    scanned = 0
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            scanned += len(chunk)
            window = tail + chunk
            if any(needle in window for needle in needles):
                return True, scanned
            tail = window[-overlap:] if overlap > 0 else b""
    return False, scanned


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--path", action="append", type=Path, required=True)
    parser.add_argument("--key-file", action="append", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    key_files = {path.resolve() for path in args.key_file}
    needles = []
    for path in args.key_file:
        value = path.read_bytes().strip()
        if not value:
            raise SystemExit(f"empty key file: {path}")
        needles.append(value)

    output = args.output.resolve() if args.output else None
    hits = []
    files_scanned = 0
    bytes_scanned = 0
    for root in args.path:
        for path in candidate_files(root):
            resolved = path.resolve()
            if resolved in key_files or resolved == output:
                continue
            found, size = contains_secret(path, needles)
            files_scanned += 1
            bytes_scanned += size
            if found:
                hits.append(str(path))

    result = {
        "paths": [str(path) for path in args.path],
        "files_scanned": files_scanned,
        "bytes_scanned": bytes_scanned,
        "exact_key_hits": hits,
        "hit_count": len(hits),
    }
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print("exact-key scan complete; use --output for the JSON report")
    return 0 if not hits else 1


if __name__ == "__main__":
    raise SystemExit(main())
