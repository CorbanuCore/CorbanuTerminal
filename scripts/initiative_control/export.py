#!/usr/bin/env python3
"""Export only the declared source surface; never credentials or raw agent homes."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

from control import atomic_json, now, read_file, read_json


def export(repo, state, destination):
    paths = set()
    for root in ("docs/plans", "docs/sprints"):
        paths.update(p for p in (repo / root).rglob("*.md"))
        paths.add(repo / root / "check.py")
    paths.update(p for p in (repo / "scripts/initiative_control").iterdir() if p.is_file() and p.suffix in {".py", ".css", ".js", ".txt", ".json", ".md"})
    config = read_json(state / "control.json", state)
    paths.update(repo / t["path"] for t in config["human_tests"])
    paths.add(repo / "codex-rs/features/src/lib.rs")
    paths.add(repo / "docs/corbanu-product-spec.md")
    hashes = {}
    for path in sorted(paths):
        text = read_file(path, repo)
        relative = path.relative_to(repo).as_posix()
        hashes[relative] = hashlib.sha256(text.encode()).hexdigest()
        if destination:
            target = destination / "source" / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(text, encoding="utf-8")
    git = lambda *args: subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()
    manifest = {
        "collected_at": now(), "commit": git("rev-parse", "HEAD"),
        "branch": git("rev-parse", "--abbrev-ref", "HEAD"),
        "label": "Manager's declared local planning checkout",
        "note": "Declared source files are pinned by hashes and may include uncommitted changes. Source identity does not prove deployment, enrollment or delivery.",
        "files": hashes,
        "tree_digest": hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest(),
    }
    atomic_json(state / "source.json", manifest)
    if destination:
        atomic_json(destination / "state/source.json", manifest)
        atomic_json(destination / "state/control.json", config)
        for path in (state / "events").glob("*.json"):
            target = destination / "state/events" / path.name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
        # Only completed exports bearing this tool's manifest, under this exact
        # destination parent. Keep three local copies; remote rollback is separate.
        copies = sorted((p for p in destination.parent.iterdir()
                         if re.fullmatch(r"export\.[A-Za-z0-9]{6}", p.name)
                         and p.is_dir() and not p.is_symlink()
                         and (p / "state/source.json").is_file()
                         and (p / "source/scripts/initiative_control/export.py").is_file()),
                        key=lambda p: p.stat().st_mtime, reverse=True)
        for old in copies[3:]:
            if old != destination:
                shutil.rmtree(old)
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--destination", type=Path)
    args = parser.parse_args()
    result = export(args.repo.resolve(), args.state.resolve(), args.destination)
    print(f"Exported {len(result['files'])} allowlisted files; tree {result['tree_digest']}")
