"""Narrow postflight/failure reporting for the existing private dashboard sync."""
import argparse
from pathlib import Path

from control import atomic_json, locked, now, read_json


def check(root, expected_commit=None, expected_digest=None):
    with locked(root / ".sync.lock"):
        source = read_json(root / "state/source.json", root)
        published = read_json(root / "site/current/manifest.json", root)
        health = read_json(root / "site/health.json", root)
        if ((expected_commit and source["commit"] != expected_commit)
                or (expected_digest and source["tree_digest"] != expected_digest)):
            raise ValueError("server source differs from this sync's expected export")
        if (not health.get("ok") or published["source"] != source
                or health["collected_at"] != source["collected_at"]
                or health["generation"] != (root / "site/current").resolve().name):
            raise ValueError("published generation does not match the collected source")
        print(f"Verified publication: {source['branch']} @ {source['commit']} · {source['tree_digest']} · {source['collected_at']}")


def failed(root):
    with locked(root / ".sync.lock"):
        failure = {
            "ok": False, "attempted_at": now(),
            "error": "Source synchronization failed; last good snapshot retained. Manager attention required.",
        }
        atomic_json(root / "state/sync-failure.json", failure)
        atomic_json(root / "site/health.json", failure)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "failed"])
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--expected-commit")
    parser.add_argument("--expected-digest")
    args = parser.parse_args()
    if args.root.resolve().name != "corbanu-control":
        parser.error("requires the dedicated corbanu-control directory")
    if args.command == "check":
        check(args.root.resolve(), args.expected_commit, args.expected_digest)
    else:
        failed(args.root.resolve())
