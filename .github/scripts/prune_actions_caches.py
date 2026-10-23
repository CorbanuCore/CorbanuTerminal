#!/usr/bin/env python3

"""Keep only the newest copy of each run-scoped Actions cache on main (#636).

Build caches such as sccache end their key with the run ID (optionally followed
by an attempt number), so every main run saves a fresh multi-GB copy and the
copies fill the repository's 10 GB limit. Restores pick the newest entry by key
prefix, so older copies are dead weight. This script groups main's run-scoped
entries by family (the key without the run ID and any hash segments right
before it) and deletes all but the newest entry of each family. Keys without a
run ID (cargo-home, apt, ...) are saved only on a key miss and are left alone.
"""

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime


MAIN_REF = "refs/heads/main"
# `-<run id>` or `-<run id>-<attempt>` at the end of the key. Run IDs have 11
# digits now; 10+ keeps short all-digit hashes (rust-cache) from matching.
RUN_ID_SUFFIX = re.compile(r"-\d{10,12}(?:-\d{1,3})?$")
# Lockfile, toolchain or commit hashes right before the run ID; restores fall
# back across them by prefix, so they don't make a separate family.
HASH_SUFFIX = re.compile(r"-[0-9a-f]{16,64}$")


@dataclass(frozen=True)
class CacheEntry:
    id: int
    key: str
    ref: str
    size_in_bytes: int
    created_at: str


def family(key: str) -> str | None:
    """Return the family of a run-scoped key, or None for other keys."""
    stripped = RUN_ID_SUFFIX.sub("", key)
    if stripped == key:
        return None
    while True:
        shorter = HASH_SUFFIX.sub("", stripped)
        if shorter == stripped:
            return stripped
        stripped = shorter


def created(entry: CacheEntry) -> datetime:
    return datetime.fromisoformat(entry.created_at.replace("Z", "+00:00"))


def plan_deletions(entries: list[CacheEntry]) -> list[CacheEntry]:
    families: dict[str, list[CacheEntry]] = defaultdict(list)
    for entry in entries:
        name = family(entry.key)
        if entry.ref == MAIN_REF and name is not None:
            families[name].append(entry)
    deletions = []
    for members in families.values():
        members.sort(key=lambda entry: (created(entry), entry.id), reverse=True)
        deletions.extend(members[1:])
    return sorted(deletions, key=lambda entry: entry.key)


def gh_api(*args: str) -> str:
    return subprocess.run(
        ["gh", "api", *args], check=True, capture_output=True, text=True
    ).stdout


def list_main_caches(repo: str) -> list[CacheEntry]:
    output = gh_api(
        "--paginate",
        f"repos/{repo}/actions/caches?ref={MAIN_REF}&per_page=100",
        "--jq",
        ".actions_caches[]",
    )
    return [
        CacheEntry(
            id=item["id"],
            key=item["key"],
            ref=item["ref"],
            size_in_bytes=item["size_in_bytes"],
            created_at=item["created_at"],
        )
        for item in map(json.loads, output.splitlines())
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, help="owner/name")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    deletions = plan_deletions(list_main_caches(args.repo))
    freed = 0
    failed = 0
    for entry in deletions:
        verb = "would delete" if args.dry_run else "delete"
        print(f"{verb} {entry.key} ({entry.size_in_bytes / 1e6:.0f} MB)")
        if args.dry_run:
            freed += entry.size_in_bytes
            continue
        try:
            gh_api("--method", "DELETE", f"repos/{args.repo}/actions/caches/{entry.id}")
            freed += entry.size_in_bytes
        except subprocess.CalledProcessError as error:
            # 404: a concurrent prune run deleted it first. Anything else
            # (e.g. a 403 token policy) fails the job after the other deletes.
            if "HTTP 404" not in error.stderr:
                failed += 1
            print(f"  failed: {error.stderr.strip()}", file=sys.stderr)
    print(f"{freed / 1e9:.2f} GB freed; {failed} deletes failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
