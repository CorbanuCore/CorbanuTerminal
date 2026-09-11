#!/usr/bin/env python3
"""Read-only, fixed-baseline proof for the September 10 ledger repair."""

import importlib.util
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MAP = json.loads(Path(__file__).with_name("identity-map.json").read_text())
BASE = MAP["base_commit"]
RECORDS = {row["old_path"]: row for row in MAP["records"]}
SPEC = importlib.util.spec_from_file_location("sprints", ROOT / "docs/sprints/check.py")
assert SPEC and SPEC.loader
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)
errors = []


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True)


def original(path):
    return git("show", f"{BASE}:{path}")


def require(condition, message):
    if not condition:
        errors.append(message)


def canonical(text, row):
    for old, new in row["replacements"].items():
        text = text.replace(old, new)
    return text


def front_without_updated(text):
    front = text.split("---", 2)[1]
    return re.sub(r"^updated: .*\n", "", front, flags=re.M)


result = checker.check_sprints(ROOT / "docs/sprints", ROOT)
errors.extend(result["errors"])
require((result["current_count"], result["archive_count"]) == (112, 121),
        "repair inventory changed")
reserved = {r["sprint_id"] for r in result["sprints"]
            if r["lifecycle"] == "current" and r["status"] in checker.RESERVED_STATUSES}
require(reserved == {"PF-27-S04", "PF-35-S01"}, "reserved sprint set changed")

# Compare every original sprint, not just the records that produced errors.
paths = git("ls-tree", "-r", "--name-only", BASE, "--",
            "docs/sprints/current", "docs/sprints/archive").splitlines()
checked = 0
for path in paths:
    if not path.endswith(".md") or path.endswith("/index.md"):
        continue
    checked += 1
    row = RECORDS.get(path)
    target = ROOT / (row["new_path"] if row else path)
    require(target.is_file(), f"missing sprint: {target}")
    if not target.is_file():
        continue
    before, after = original(path), target.read_text()
    if row is None:
        require(before == after, f"unrelated sprint changed: {path}")
        continue
    require(not (ROOT / path).exists(), f"legacy duplicate path remains: {path}")
    require(front_without_updated(canonical(before, row)) == front_without_updated(after),
            f"metadata beyond identity/update date changed: {path}")
    before_boxes = re.findall(r"^- \[[ xX]\].*$", canonical(before, row), re.M)
    after_boxes = re.findall(r"^- \[[ xX]\].*$", after, re.M)
    require(before_boxes == after_boxes, f"checkbox evidence changed: {path}")
    _, fields, _ = checker.parse_front_matter(target)
    require(fields["sprint_id"] == row["new_id"], f"wrong canonical ID: {path}")
    require(fields["plan_file"] == MAP["plan_file"], f"owning plan changed: {path}")

# Worktree authorizations in the owning plan remain byte-for-byte equivalent.
plan = MAP["plan_file"]
require(front_without_updated(original(plan)) ==
        front_without_updated((ROOT / plan).read_text()),
        "active-plan authority/allocation frontmatter changed")

# Historical releases retain labels and claims; only two link destinations move.
release_links = {
    "qa/release/0.1.40/RELEASE.md": ("pf-45-s02-agent-profile-scope.md",
                                   "pf-78-s02-agent-profile-scope.md"),
    "qa/release/0.1.41/RELEASE.md": ("pf-42-s02-relink-recovery.md",
                                   "pf-77-s02-relink-recovery.md"),
}
for path, (old, new) in release_links.items():
    require(original(path).replace(old, new) == (ROOT / path).read_text(),
            f"release evidence changed beyond link repair: {path}")

allowed = set(RECORDS) | {r["new_path"] for r in RECORDS.values()} | set(release_links) | {
    "docs/corbanu-product-spec.md", plan, "docs/plans/portfolio-2026-09-09.md",
    "docs/plans/scrum-2026-09-10.md", "docs/sprints/index.md",
    "docs/sprints/current/p0-security-levels/index.md",
    "docs/sprints/identity-reconciliation-2026-09-10.md",
    "docs/sprints/tests/test_check.py", "mkdocs.yml",
}
changed = set(git("diff", "--no-renames", "--name-only", BASE, "--").splitlines())
changed.update(git("ls-files", "--others", "--exclude-standard").splitlines())
for path in changed:
    require(path in allowed or path.startswith("qa/planning-ledger/2026-09-10/"),
            f"outside fixed repair scope: {path}")

# Check local Markdown links in changed docs and the five renamed sprint records.
for path in sorted(changed):
    source = ROOT / path
    if not source.is_file() or not path.endswith(".md"):
        continue
    for link in re.findall(r"\[[^\]\n]+\]\(([^)\n]+)\)", source.read_text()):
        target = link.split("#", 1)[0]
        if not target or "://" in target or target.startswith("mailto:"):
            continue
        require((source.parent / target).resolve().exists(),
                f"broken local link: {path}: {link}")

print(json.dumps({
    "ok": not errors, "base_commit": BASE, "errors": errors,
    "original_sprints_checked": checked, "renumbered_sprints": len(RECORDS),
    "current": result["current_count"], "archived": result["archive_count"],
    "reserved_sprints": sorted(reserved),
}, indent=2))
raise SystemExit(1 if errors else 0)
