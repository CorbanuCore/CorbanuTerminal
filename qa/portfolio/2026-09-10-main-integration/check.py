#!/usr/bin/env python3
"""Fixed-baseline publication checks, not a waiver of global sprint errors."""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def run_json(path, *args):
    result = subprocess.run(
        [sys.executable, str(ROOT / path), *args],
        cwd=ROOT, text=True, capture_output=True, check=False,
    )
    if result.returncode not in (0, 1):
        raise RuntimeError(result.stderr or result.stdout)
    return result.returncode, json.loads(result.stdout)


baseline = json.loads((HERE / "baseline.json").read_text())
portfolio_exit, portfolio = run_json("qa/portfolio/2026-09-09/validate.py")
sprint_exit, sprints = run_json("docs/sprints/check.py", "--json")
errors = list(portfolio["errors"])
if portfolio_exit:
    errors.append("fixed portfolio validator failed")
if sorted(sprints["errors"]) != sorted(baseline["errors"]):
    errors.append("global sprint errors differ from the seven inherited errors")
if sprint_exit != 1 or len(sprints["errors"]) != 7:
    errors.append("unexpected global checker status; rebaseline deliberately")
if sprints["current_count"] != baseline["current_count"] + 50:
    errors.append("unexpected current sprint inventory")
if sprints["archive_count"] != baseline["archive_count"]:
    errors.append("archive inventory changed")
reserved = [
    {key: row[key] for key in ("sprint_id", "status", "owner", "plan_file")}
    for row in sprints["sprints"] if row["status"] in ("in_progress", "blocked")
]
if reserved != baseline["reserved"]:
    errors.append("reserved sprint inventory changed")
new_ids = {
    row["sprint_id"] for row in sprints["sprints"]
    if row["path"].startswith("docs/sprints/current/portfolio-")
}
if new_ids & set(baseline["existing_sprint_ids"]):
    errors.append("portfolio sprint IDs collide with inherited IDs")

# The old validator checks its fixed historical scope. Check links in all new
# publication Markdown too, without treating private QA receipts as site pages.
files = [ROOT / "docs/plans/scrum-2026-09-10.md", *HERE.glob("*.md")]
for path in files:
    body = path.read_text()
    for link in re.findall(r"\[[^\]\n]+\]\(([^)\n]+)\)", body):
        target = link.split("#", 1)[0]
        if target and "://" not in target and not target.startswith("mailto:"):
            if not (path.parent / target).resolve().is_file():
                errors.append(f"{path.relative_to(ROOT)}: missing link {link}")
    if "/Downloads/" in body or ".m4a" in body:
        errors.append(f"{path.relative_to(ROOT)}: private recording location")

print(json.dumps({
    "ok": not errors,
    "baseline": baseline["base_commit"],
    "portfolio_plans": portfolio["plans"],
    "portfolio_sprints": portfolio["sprints"],
    "max_sprint_lines": portfolio["max_sprint_lines"],
    "current_count": sprints["current_count"],
    "archive_count": sprints["archive_count"],
    "inherited_global_errors": len(sprints["errors"]),
    "global_sprint_checker_ok": sprints["ok"],
    "reserved_sprints": [row["sprint_id"] for row in reserved],
    "errors": errors,
}, indent=2))
raise SystemExit(bool(errors))
