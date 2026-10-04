#!/usr/bin/env python3
"""Export only the declared source surface; never credentials or raw agent homes."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess

from control import MAX_FILE, atomic_json, now, read_file, read_json, reference_documents, safe_text
import decision_feed

SCRIPT_SUFFIXES = {".py", ".css", ".js", ".sh", ".txt", ".json", ".md"}
FIXED_PATHS = ("codex-rs/features/src/lib.rs", "docs/corbanu-product-spec.md")


def source_paths(repo, config):
    paths = set()
    for root in ("docs/plans", "docs/sprints"):
        paths.update(p for p in (repo / root).rglob("*.md"))
        paths.add(repo / root / "check.py")
    paths.update(p for p in (repo / "scripts/initiative_control").iterdir() if p.is_file() and p.suffix in SCRIPT_SUFFIXES)
    paths.update(repo / t["path"] for t in config["human_tests"])
    for relative in reference_documents(config):
        path = repo / relative
        safe_text(read_file(path, repo), MAX_FILE)
        paths.add(path)
    paths.update(repo / p for p in FIXED_PATHS)
    return paths


def committed_files(repo, commit, config):
    """The same declared surface read from one commit; working-copy edits never publish."""
    listing = subprocess.check_output(["git", "-C", str(repo), "ls-tree", "-r", "-z", "--full-tree", commit])
    tree = {}
    for record in listing.split(b"\0"):
        if record:
            meta, path = record.split(b"\t", 1)
            mode, kind, oid = meta.decode().split()
            tree[path.decode()] = (mode, kind, oid)
    relatives = {p for p in tree if p.startswith(("docs/plans/", "docs/sprints/")) and p.endswith(".md")}
    relatives.update(("docs/plans/check.py", "docs/sprints/check.py", *FIXED_PATHS))
    relatives.update(p for p in tree if PurePosixPath(p).parent.as_posix() == "scripts/initiative_control"
                     and PurePosixPath(p).suffix in SCRIPT_SUFFIXES)
    relatives.update(t["path"] for t in config["human_tests"])
    references = set(reference_documents(config))
    relatives.update(references)
    for relative in relatives:
        if relative not in tree:
            raise ValueError(f"declared source file is not committed: {relative}")
        mode, kind, _ = tree[relative]
        if kind != "blob" or mode not in {"100644", "100755"}:
            raise ValueError("source is not a regular committed file")
    ordered = sorted(relatives)
    batch = subprocess.run(["git", "-C", str(repo), "cat-file", "--batch"], check=True, capture_output=True,
                           input=b"".join(tree[p][2].encode() + b"\n" for p in ordered)).stdout
    texts, offset = {}, 0
    for relative in ordered:
        header_end = batch.index(b"\n", offset)
        _, kind, size = batch[offset:header_end].decode().split()
        if kind != "blob" or int(size) > MAX_FILE:
            raise ValueError("source exceeds publication size limit")
        data = batch[header_end + 1:header_end + 1 + int(size)]
        offset = header_end + 2 + int(size)
        # Same universal-newline text that read_file() yields for a checkout.
        text = data.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")
        if relative in references:
            safe_text(text, MAX_FILE)
        texts[relative] = text
    return texts


def worktree_files(repo, config):
    return {p.relative_to(repo).as_posix(): read_file(p, repo) for p in sorted(source_paths(repo, config))}


def identity(repo):
    git = lambda *args: subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()
    return git("rev-parse", "HEAD"), git("rev-parse", "--abbrev-ref", "HEAD")


def verify_source(repo, manifest, expected_branch=None, config=None):
    commit, branch = identity(repo)
    if expected_branch and branch != expected_branch:
        raise ValueError("dashboard source is not the declared manager branch")
    if (commit, branch) != (manifest["commit"], manifest["branch"]):
        raise ValueError("dashboard source revision changed during synchronization; retry")
    if manifest.get("checkout") != str(repo.resolve()):
        raise ValueError("dashboard source checkout differs from the collected checkout")
    if manifest.get("mode") == "committed":
        # Commit contents are immutable; recheck inventory and bytes from it.
        texts = committed_files(repo, commit, config)
        if set(texts) != set(manifest["files"]):
            raise ValueError("dashboard source inventory/configuration changed during synchronization; retry")
        for relative, expected in manifest["files"].items():
            if hashlib.sha256(texts[relative].encode()).hexdigest() != expected:
                raise ValueError("dashboard source contents changed during synchronization; retry")
        return
    for relative, expected in manifest["files"].items():
        if hashlib.sha256(read_file(repo / relative, repo).encode()).hexdigest() != expected:
            raise ValueError("dashboard source contents changed during synchronization; retry")


def export(repo, state, destination, expected_branch=None, committed=False):
    commit, branch = identity(repo)
    if expected_branch and branch != expected_branch:
        raise ValueError("dashboard source is not the declared manager branch")
    config = read_json(state / "control.json", state)
    collected_at = now()
    if destination:
        feed_bytes, feed_pin = decision_feed.capture(state, collected_at)
    paths = None if committed else source_paths(repo, config)
    texts = committed_files(repo, commit, config) if committed else worktree_files(repo, config)
    hashes = {}
    for relative, text in sorted(texts.items()):
        hashes[relative] = hashlib.sha256(text.encode()).hexdigest()
        if destination:
            target = destination / "source" / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(text, encoding="utf-8")
    label = "Manager's declared receiving checkout" if expected_branch else "Manager's declared local planning checkout"
    manifest = {
        "collected_at": collected_at, "commit": commit, "branch": branch,
        "checkout": str(repo.resolve()),
        "label": label + (" (committed)" if committed else ""),
        "note": ("Declared source files are read from this commit and pinned by hashes; uncommitted checkout changes are excluded."
                 if committed else "Declared source files are pinned by hashes and may include uncommitted changes.")
                + " Source identity does not prove deployment, enrollment or delivery.",
        "files": hashes,
        "tree_digest": hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest(),
    }
    if committed:
        manifest["mode"] = "committed"
    # Local metadata-only export has no immutable feed artifact: preserve its
    # existing publication path with explicit unknown decisions, not a false pin.
    if destination:
        manifest["decision_feed"] = feed_pin
    verify_source(repo, manifest, expected_branch, config)
    if (not committed and source_paths(repo, config) != paths) or read_json(state / "control.json", state) != config:
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
    parser.add_argument("--committed", action="store_true",
                        help="Read the declared files from the HEAD commit, excluding uncommitted checkout changes")
    parser.add_argument("--verify", type=Path, help="Verify an existing export against the current source; no writes")
    args = parser.parse_args()
    if args.verify:
        manifest = read_json(args.verify, args.verify.parent)
        current_config = read_json(args.state / "control.json", args.state)
        exported_config = read_json(args.verify.parent / "control.json", args.verify.parent)
        if current_config != exported_config:
            raise ValueError("dashboard source inventory/configuration changed during synchronization; retry")
        if args.committed != (manifest.get("mode") == "committed"):
            raise ValueError("dashboard export mode differs from the requested verification mode")
        verify_source(args.repo.resolve(), manifest, args.expected_branch, current_config)
        if not args.committed:
            inventory = {p.relative_to(args.repo.resolve()).as_posix() for p in source_paths(args.repo.resolve(), current_config)}
            if inventory != set(manifest["files"]):
                raise ValueError("dashboard source inventory/configuration changed during synchronization; retry")
        decision_feed.verify_input(args.state, manifest, manifest["collected_at"])
        print("Source checkout/revision/content match the collected export")
        raise SystemExit(0)
    result = export(args.repo.resolve(), args.state.resolve(), args.destination, args.expected_branch, args.committed)
    print(f"Exported {len(result['files'])} allowlisted files; tree {result['tree_digest']}")
