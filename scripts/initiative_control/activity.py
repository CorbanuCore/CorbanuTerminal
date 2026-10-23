"""Presentation only: distinguish reported execution from sprint lifecycle."""
from datetime import datetime

LABELS = {
    "working": ("Working", "working"),
    "awaiting_review": ("Awaiting manager review", "awaiting_review"),
    "blocked": ("Blocked", "blocked"),
    "finished": ("Assignment returned", "awaiting_review"),
    "failed": ("Run failed — manager follow-up", "failed"),
    "cancelled": ("Cancelled — manager follow-up", "awaiting_review"),
}
ACTIVE = {"working", "awaiting_review", "blocked"}


def observed(run):
    return datetime.fromisoformat(run["updated_at"].replace("Z", "+00:00"))


def latest_reports(reports):
    latest = {}
    for report in sorted(reports, key=observed):
        group = latest.setdefault(report["run_id"], [])
        if group and observed(report) > observed(group[0]):
            group.clear()
        if report not in group:
            group.append(report)
    return sorted((r for group in latest.values() for r in group),
                  key=lambda r: (observed(r), r["run_id"]), reverse=True)


def presentation(reports):
    reports = latest_reports(reports)
    if not reports:
        return "Activity unknown — no report", "unknown", None
    if len(reports) != len({r["run_id"] for r in reports}):
        return "Conflicting reports — manager check", "awaiting_review", min(reports, key=observed)["updated_at"]
    # A newer returned assignment must not hide another unresolved run.
    current = [r for r in reports if r["status"] in ACTIVE] or reports[:1]
    states = {r["status"] for r in current}
    seen = min(current, key=observed)["updated_at"]
    if len(states) > 1:
        return "Mixed activity — see run details", "awaiting_review", seen
    label, tone = LABELS[current[0]["status"]]
    return label, tone, seen
