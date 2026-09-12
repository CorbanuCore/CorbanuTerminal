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
import decision_feed


def source_paths(repo, config):
    paths = set()
    for root in ("docs/plans", "docs/sprints"):
        paths.update(p for p in (repo / root).rglob("*.md"))
        paths.add(repo / root / "check.py")
    paths.update(p for p in (repo / "scripts/initiative_control").iterdir() if p.is_file() and p.suffix in {".py", ".css", ".js", ".sh", ".txt", ".json", ".md"})
    paths.update(repo / t["path"] for t in config["human_tests"])
    paths.add(repo / "codex-rs/features/src/lib.rs")
    paths.add(repo / "docs/corbanu-product-spec.md")
    return paths


def identity(repo):
    git = lambda *args: subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()
    return git("rev-parse", "HEAD"), git("rev-parse", "--abbrev-ref", "HEAD")


def verify_source(repo, manifest, expected_branch=None):
    commit, branch = identity(repo)
    if expected_branch and branch != expected_branch:
        raise ValueError("dashboard source is not the declared manager branch")
    if (commit, branch) != (manifest["commit"], manifest["branch"]):
        raise ValueError("dashboard source revision changed during synchronization; retry")
    if manifest.get("checkout") != str(repo.resolve()):
        raise ValueError("dashboard source checkout differs from the collected checkout")
    for relative, expected in manifest["files"].items():
        if hashlib.sha256(read_file(repo / relative, repo).encode()).hexdigest() != expected:
            raise ValueError("dashboard source contents changed during synchronization; retry")


def export(repo, state, destination, expected_branch=None):
    commit, branch = identity(repo)
    if expected_branch and branch != expected_branch:
        raise ValueError("dashboard source is not the declared manager branch")
    config = read_json(state / "control.json", state)
    collected_at = now()
    if destination:
        feed_bytes, feed_pin = decision_feed.capture(state, collected_at)
    paths = source_paths(repo, config)
    hashes = {}
    for path in sorted(paths):
        text = read_file(path, repo)
        relative = path.relative_to(repo).as_posix()
        hashes[relative] = hashlib.sha256(text.encode()).hexdigest()
        if destination:
            target = destination / "source" / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(text, encoding="utf-8")
    manifest = {
        "collected_at": collected_at, "commit": commit, "branch": branch,
        "checkout": str(repo.resolve()),
        "label": "Manager's declared receiving checkout" if expected_branch else "Manager's declared local planning checkout",
        "note": "Declared source files are pinned by hashes and may include uncommitted changes. Source identity does not prove deployment, enrollment or delivery.",
        "files": hashes,
        "tree_digest": hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest(),
    }
    # Local metadata-only export has no immutable feed artifact: preserve its
    # existing publication path with explicit unknown decisions, not a false pin.
    if destination:
        manifest["decision_feed"] = feed_pin
    verify_source(repo, manifest, expected_branch)
    if source_paths(repo, config) != paths or read_json(state / "control.json", state) != config:
        raise ValueError("dashboard source inventory/configuration changed during collection; retry")
    decision_feed.verify_input(state, manifest, collected_at)
    if destination:
        # Fixed snapshot belongs to this export, never to mutable remote state.
        target = destination / "source" / decision_feed.ARTIFACT
        with target.open("xb") as stream:
            stream.write(feed_bytes)
        decision_feed.read_snapshot(destination / "source", manifest, collected_at)
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
    parser.add_argument("--expected-branch")
    parser.add_argument("--verify", type=Path, help="Verify an existing export against the current source; no writes")
    args = parser.parse_args()
    if args.verify:
        manifest = read_json(args.verify, args.verify.parent)
        verify_source(args.repo.resolve(), manifest, args.expected_branch)
        current_config = read_json(args.state / "control.json", args.state)
        exported_config = read_json(args.verify.parent / "control.json", args.verify.parent)
        inventory = {p.relative_to(args.repo.resolve()).as_posix() for p in source_paths(args.repo.resolve(), current_config)}
        if current_config != exported_config or inventory != set(manifest["files"]):
            raise ValueError("dashboard source inventory/configuration changed during synchronization; retry")
        decision_feed.verify_input(args.state, manifest, manifest["collected_at"])
        print("Source checkout/revision/content match the collected export")
        raise SystemExit(0)
    result = export(args.repo.resolve(), args.state.resolve(), args.destination, args.expected_branch)
    print(f"Exported {len(result['files'])} allowlisted files; tree {result['tree_digest']}")
