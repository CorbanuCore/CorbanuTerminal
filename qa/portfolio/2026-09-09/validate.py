#!/usr/bin/env python3
"""Read-only checks for this fixed draft portfolio; complements lifecycle checkers."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
plans = sorted(ROOT.glob("docs/plans/proposed/portfolio-*.md"))
sprints = sorted(ROOT.glob("docs/sprints/current/portfolio-*/*.md"))
extras = [
    ROOT / "docs/plans/portfolio-2026-09-09.md",
    ROOT / "docs/plans/proposed/index.md",
    ROOT / "docs/sprints/index.md",
    ROOT / "mkdocs.yml",
    ROOT / "qa/portfolio/2026-09-09/review-scope.md",
    Path(__file__).resolve(),
]
files = plans + sprints + extras
errors = []
spec = (ROOT / "docs/corbanu-product-spec.md").read_text(encoding="utf-8")
nav = (ROOT / "mkdocs.yml").read_text(encoding="utf-8")
ids = set()


def field(body, name):
    match = re.search(r"^" + re.escape(name) + r":\s*(.*?)\s*$", body, re.M)
    return match.group(1).strip('"') if match else None


for path in plans + sprints:
    body = path.read_text(encoding="utf-8")
    if field(body, "status") != "draft":
        errors.append(f"{path}: not draft")
    nav_path = str(path.relative_to(ROOT / "docs"))
    if ": " + nav_path not in nav:
        errors.append(f"{path}: absent from navigation")
    if "/Downloads/" in body or ".m4a" in body or "artifacts/transcription/" in body:
        errors.append(f"{path}: private source path in public document")
for path in plans:
    body = path.read_text(encoding="utf-8")
    heading = re.search(r'^  heading: "(.*)"$', body, re.M)
    excerpt = re.search(r'^  requirement_excerpt: "(.*)"$', body, re.M)
    if heading is None or not re.search(r"^#+ " + re.escape(heading.group(1)) + r"$", spec, re.M):
        errors.append(f"{path}: missing exact spec heading")
    if excerpt is None or excerpt.group(1) not in spec:
        errors.append(f"{path}: nonverbatim spec excerpt")
    if "implementation_worktrees: []" not in body:
        errors.append(f"{path}: unexpected allocation")
for path in sprints:
    body = path.read_text(encoding="utf-8")
    sid = field(body, "sprint_id")
    if not sid or sid in ids:
        errors.append(f"{path}: missing/duplicate sprint id")
    ids.add(sid or "")
    if len(body.splitlines()) > 100:
        errors.append(f"{path}: over 100 lines")
    for key in ("worktree", "branch", "base_commit", "write_scope", "parallel_lane", "integration_gate"):
        if field(body, key) != "UNALLOCATED":
            errors.append(f"{path}: unexpected {key} reservation")
    plan = ROOT / (field(body, "plan_file") or "missing-plan")
    relative = "../../" + str(path.relative_to(ROOT)).removeprefix("docs/")
    if not plan.is_file() or relative not in plan.read_text(encoding="utf-8"):
        errors.append(f"{path}: missing exact plan backlink")
for path in files:
    body = path.read_text(encoding="utf-8")
    if path.name == "portfolio-2026-09-09.md":
        if "/Downloads/" in body or ".m4a" in body or "artifacts/transcription/" in body:
            errors.append("portfolio overview contains a private source path")
    for link in re.findall(r"\[[^\]\n]+\]\(([^)\n]+)\)", body):
        target = link.split("#", 1)[0]
        if not target or "://" in target or target.startswith("mailto:"):
            continue
        if not (path.parent / target).resolve().exists():
            errors.append(f"{path}: broken link {link}")
if len(plans) != 16 or len(sprints) != 50:
    errors.append("incorrect plan/sprint count")
if {sid[:5] for sid in ids} != {f"PF-{n}" for n in range(60, 76)}:
    errors.append("unexpected feature IDs")
manifest = [
    {"path": str(path.relative_to(ROOT)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
     "bytes": path.stat().st_size}
    for path in sorted(files)
]
print(json.dumps({
    "plans": len(plans), "sprints": len(sprints),
    "max_sprint_lines": max((len(p.read_text(encoding="utf-8").splitlines()) for p in sprints), default=0),
    "scope_files": len(files), "errors": errors, "manifest": manifest,
}, indent=2))
raise SystemExit(bool(errors))
