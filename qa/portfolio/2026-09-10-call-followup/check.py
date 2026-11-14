#!/usr/bin/env python3
"""Planning amendment checks; inherited global failures remain failures."""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
BASE = "6ca801add7eeda9feb9723e7f4935fbf6b29a73b"


def run_json(path, *args):
    result = subprocess.run(
        [sys.executable, str(ROOT / path), *args],
        cwd=ROOT, text=True, capture_output=True, check=False,
    )
    if result.returncode not in (0, 1):
        raise RuntimeError(result.stderr or result.stdout)
    return result.returncode, json.loads(result.stdout)


baseline = json.loads(
    (HERE.parent / "2026-09-10-main-integration/baseline.json").read_text()
)
portfolio_exit, portfolio = run_json(
    "qa/portfolio/2026-09-09/validate.py", "--expected-sprints", "52"
)
sprint_exit, sprints = run_json("docs/sprints/check.py", "--json")
errors = list(portfolio["errors"])
if portfolio_exit:
    errors.append("amended portfolio validation failed")
if sprint_exit != 1 or sorted(sprints["errors"]) != sorted(baseline["errors"]):
    errors.append("global status differs from the seven inherited failures")
if sprints["current_count"] != 112 or sprints["archive_count"] != 121:
    errors.append("unexpected current/archive inventory")
reserved = [
    {key: row[key] for key in ("sprint_id", "status", "owner", "plan_file")}
    for row in sprints["sprints"] if row["status"] in ("in_progress", "blocked")
]
if reserved != baseline["reserved"]:
    errors.append("reserved allocations changed")
new_ids = {
    row["sprint_id"] for row in sprints["sprints"]
    if row["path"].startswith("docs/sprints/current/portfolio-")
}
if new_ids & set(baseline["existing_sprint_ids"]):
    errors.append("portfolio IDs collide with inherited records")

for path in [
    ROOT / "docs/plans/call-followup-2026-09-10.md",
    ROOT / "docs/plans/scrum-2026-09-10.md",
    *HERE.glob("*.md"),
]:
    body = path.read_text()
    for link in re.findall(r"\[[^\]\n]+\]\(([^)\n]+)\)", body):
        target = link.split("#", 1)[0]
        if target and "://" not in target and not target.startswith("mailto:"):
            if not (path.parent / target).resolve().is_file():
                errors.append(f"{path.relative_to(ROOT)}: broken link {link}")
    if any(marker in body for marker in ("/Downloads/", ".m4a", "artifacts/transcription/")):
        errors.append(f"{path.relative_to(ROOT)}: private recording location")

protected = [
    "AGENTS.md", "docs/corbanu-product-spec.md", "docs/plans/index.md",
    "docs/plans/active", "docs/sprints/current/p0-security-levels",
    "docs/sprints/archive", "docs/plans/check.py", "docs/sprints/check.py",
    "codex-rs", ".github", "scripts/initiative_control",
]
unchanged = subprocess.run(
    ["git", "diff", "--quiet", BASE, "--", *protected], cwd=ROOT, check=False
)
if unchanged.returncode:
    errors.append("protected runtime/policy/allocation boundary changed or unavailable")

print(json.dumps({
    "ok": not errors, "source_base": BASE,
    "portfolio_plans": portfolio["plans"], "portfolio_sprints": portfolio["sprints"],
    "max_sprint_lines": portfolio["max_sprint_lines"],
    "current_count": sprints["current_count"], "archive_count": sprints["archive_count"],
    "global_sprint_checker_ok": sprints["ok"],
    "inherited_global_errors": len(sprints["errors"]),
    "reserved_sprints": [row["sprint_id"] for row in reserved],
    "errors": errors,
}, indent=2))
raise SystemExit(bool(errors))
