#!/usr/bin/env python3
"""Validate Corbanu sprint lifecycle, plan linkage, and execution readiness."""

import argparse
import json
import os
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
REPO_ROOT = ROOT.parents[1]
CURRENT_STATUSES = {"draft", "ready", "in_progress", "blocked"}
ARCHIVE_STATUSES = {"completed", "cancelled"}
EXECUTABLE_STATUSES = {"ready", "in_progress", "blocked"}
REQUIRED_KEYS = (
    "sprint_id",
    "title",
    "status",
    "plan_file",
    "plan_feature",
    "execution_order",
    "owner",
    "worktree",
    "branch",
    "base_commit",
    "depends_on",
    "created",
    "updated",
)
REQUIRED_SECTIONS = (
    "Execution mandate",
    "Plan linkage",
    "Code boundaries",
    "Preconditions",
    "Done",
    "Remaining",
    "Verification",
    "Exit evidence",
)
MAX_CURRENT_LINES = 100


def scalar(value):
    value = value.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {'"', "'"}:
        return value[1:-1]
    return value


def parse_front_matter(path):
    text = path.read_text(encoding="utf-8")
    match = re.match(r"\A---\r?\n(.*?)\r?\n---(?:\r?\n|\Z)", text, re.DOTALL)
    if match is None:
        return text, {}, ""
    front = match.group(1)
    values = {}
    for key, value in re.findall(r"^([a-z][a-z0-9_-]*):[ \t]*(.*?)\s*$", front, re.MULTILINE):
        values[key] = scalar(value)
    return text, values, front


def section_body(text, heading):
    match = re.search(
        rf"^## {re.escape(heading)}\s*$\r?\n(.*?)(?=^## |\Z)",
        text,
        re.MULTILINE | re.DOTALL,
    )
    return match.group(1) if match else ""


def plan_status(path):
    _, values, _ = parse_front_matter(path)
    return values.get("status")


def plan_worktrees(front):
    records = []
    current = None
    in_worktrees = False
    for line in front.splitlines():
        if line == "implementation_worktrees:":
            in_worktrees = True
            continue
        if not in_worktrees:
            continue
        if line and not line[0].isspace():
            break
        path_match = re.match(r"^[ \t]+-[ \t]+path:[ \t]*(.*?)\s*$", line)
        if path_match:
            current = {"path": scalar(path_match.group(1))}
            records.append(current)
            continue
        value_match = re.match(
            r"^[ \t]+(branch|base_commit):[ \t]*(.*?)\s*$",
            line,
        )
        if value_match and current is not None:
            current[value_match.group(1)] = scalar(value_match.group(2))
    return records


def dependency_ids(value):
    if not value or value.lower() == "none":
        return []
    return [item.strip() for item in value.split(",") if item.strip()]


def sprint_files(root):
    records = []
    for lifecycle in ("current", "archive"):
        directory = root / lifecycle
        if not directory.is_dir():
            continue
        for path in sorted(directory.rglob("*.md")):
            if path.name == "index.md":
                continue
            records.append((lifecycle, path))
    return records


def check_sprints(root=ROOT, repo_root=REPO_ROOT):
    errors = []
    records = []
    by_id = {}
    metadata_by_id = {}

    for required in ("current", "archive"):
        if not (root / required).is_dir():
            errors.append(f"missing sprint lifecycle directory: {root / required}")

    for lifecycle, path in sprint_files(root):
        text, values, front = parse_front_matter(path)
        relative = path.relative_to(repo_root).as_posix()
        sprint_id = values.get("sprint_id", "")
        record = {
            "path": relative,
            "lifecycle": lifecycle,
            "sprint_id": sprint_id,
            "title": values.get("title"),
            "status": values.get("status"),
            "plan_file": values.get("plan_file"),
            "plan_feature": values.get("plan_feature"),
            "execution_order": values.get("execution_order"),
        }
        records.append(record)

        if not front:
            errors.append(f"{relative}: missing YAML front matter")
            continue
        for key in REQUIRED_KEYS:
            if not values.get(key):
                errors.append(f"{relative}: missing required field {key!r}")

        status = values.get("status")
        allowed = CURRENT_STATUSES if lifecycle == "current" else ARCHIVE_STATUSES
        if status not in allowed:
            errors.append(
                f"{relative}: status {status!r} is invalid for {lifecycle}; "
                f"expected one of {sorted(allowed)}"
            )

        feature = values.get("plan_feature", "")
        if re.fullmatch(r"PF-\d{2}", feature) is None:
            errors.append(f"{relative}: plan_feature must be exactly one PF-NN id")
        expected_id = rf"{re.escape(feature)}-S\d{{2}}" if feature else r"PF-\d{2}-S\d{2}"
        if re.fullmatch(expected_id, sprint_id) is None:
            errors.append(f"{relative}: sprint_id must be {feature or 'PF-NN'}-SNN")
        if sprint_id and sprint_id.lower() not in path.stem:
            errors.append(f"{relative}: filename must contain lowercase sprint id {sprint_id.lower()!r}")
        if sprint_id in by_id:
            errors.append(f"{relative}: duplicate sprint_id {sprint_id!r} also used by {by_id[sprint_id]}")
        elif sprint_id:
            by_id[sprint_id] = relative
            metadata_by_id[sprint_id] = record

        try:
            order = int(values.get("execution_order", ""))
            if order <= 0:
                raise ValueError
        except ValueError:
            errors.append(f"{relative}: execution_order must be a positive integer")

        sections = set(re.findall(r"^## (.+?)\s*$", text, re.MULTILINE))
        for section in REQUIRED_SECTIONS:
            if section not in sections:
                errors.append(f"{relative}: missing required section {section!r}")

        if lifecycle == "current" and len(text.splitlines()) > MAX_CURRENT_LINES:
            errors.append(
                f"{relative}: current sprint is {len(text.splitlines())} lines; "
                f"maximum is {MAX_CURRENT_LINES}"
            )

        done = section_body(text, "Done")
        remaining = section_body(text, "Remaining")
        verification = section_body(text, "Verification")
        exit_evidence = section_body(text, "Exit evidence")
        if "- [x]" not in done.lower():
            errors.append(f"{relative}: Done must contain at least one checked item")
        if "- [ ]" in done:
            errors.append(f"{relative}: unchecked work belongs in Remaining, not Done")
        if lifecycle == "current":
            if "- [ ]" not in remaining:
                errors.append(f"{relative}: current sprint Remaining must contain unchecked work")
            if "- [x]" in remaining.lower():
                errors.append(f"{relative}: checked work belongs in Done, not Remaining")
            for heading, body in (("Verification", verification), ("Exit evidence", exit_evidence)):
                if "- [ ]" not in body:
                    errors.append(f"{relative}: {heading} must contain unchecked evidence items")
        elif status == "completed":
            for heading, body in (
                ("Remaining", remaining),
                ("Verification", verification),
                ("Exit evidence", exit_evidence),
            ):
                if "- [ ]" in body:
                    errors.append(f"{relative}: completed sprint has unchecked {heading} items")

        plan_value = values.get("plan_file", "")
        plan_path = repo_root / plan_value
        if not plan_value or Path(plan_value).is_absolute() or ".." in Path(plan_value).parts:
            errors.append(f"{relative}: plan_file must be a repository-relative path")
        elif not plan_path.is_file():
            errors.append(f"{relative}: plan file does not exist: {plan_value}")
        else:
            plan_text = plan_path.read_text(encoding="utf-8")
            _, _, plan_front = parse_front_matter(plan_path)
            if feature and feature not in plan_text:
                errors.append(f"{relative}: linked plan does not define feature {feature}")
            backlink = Path(os.path.relpath(path, plan_path.parent)).as_posix()
            if lifecycle == "current" and backlink not in plan_text:
                errors.append(f"{relative}: linked plan is missing sprint backlink {backlink!r}")
            if status in EXECUTABLE_STATUSES and plan_status(plan_path) != "active":
                errors.append(f"{relative}: {status} sprint requires an active plan")
            if status in EXECUTABLE_STATUSES:
                coordinates = {
                    "path": values.get("worktree", ""),
                    "branch": values.get("branch", ""),
                    "base_commit": values.get("base_commit", ""),
                }
                if coordinates not in plan_worktrees(plan_front):
                    errors.append(
                        f"{relative}: sprint worktree coordinates do not match "
                        "one implementation_worktrees record in the active plan"
                    )

        worktree = values.get("worktree", "")
        branch = values.get("branch", "")
        base_commit = values.get("base_commit", "")
        if status in EXECUTABLE_STATUSES:
            if worktree == "UNALLOCATED" or not Path(worktree).is_absolute():
                errors.append(f"{relative}: {status} sprint requires an exact absolute worktree")
            if branch == "UNALLOCATED":
                errors.append(f"{relative}: {status} sprint requires an exact branch")
            if re.fullmatch(r"[0-9a-f]{40}", base_commit) is None:
                errors.append(f"{relative}: {status} sprint requires a 40-character base commit")

        for dependency in dependency_ids(values.get("depends_on", "")):
            if re.fullmatch(r"PF-\d{2}-S\d{2}", dependency) is None:
                errors.append(f"{relative}: invalid dependency id {dependency!r}")

    known_ids = {record["sprint_id"] for record in records if record["sprint_id"]}
    for lifecycle, path in sprint_files(root):
        _, values, _ = parse_front_matter(path)
        relative = path.relative_to(repo_root).as_posix()
        for dependency in dependency_ids(values.get("depends_on", "")):
            if dependency not in known_ids:
                errors.append(f"{relative}: dependency does not exist: {dependency}")
            elif values.get("status") in EXECUTABLE_STATUSES:
                dependency_record = metadata_by_id[dependency]
                if not (
                    dependency_record["lifecycle"] == "archive"
                    and dependency_record["status"] == "completed"
                ):
                    errors.append(
                        f"{relative}: executable sprint dependency is not completed "
                        f"and archived: {dependency}"
                    )

    order_keys = {}
    for record in records:
        key = (record["plan_file"], record["execution_order"])
        if key in order_keys:
            errors.append(
                f"{record['path']}: duplicate execution_order {record['execution_order']} "
                f"for plan; also used by {order_keys[key]}"
            )
        else:
            order_keys[key] = record["path"]

    in_progress_by_plan = {}
    for record in records:
        if record["lifecycle"] != "current" or record["status"] != "in_progress":
            continue
        plan_file = record["plan_file"]
        if plan_file in in_progress_by_plan:
            errors.append(
                f"{record['path']}: only one sprint may be in_progress per plan; "
                f"also in progress: {in_progress_by_plan[plan_file]}"
            )
        else:
            in_progress_by_plan[plan_file] = record["path"]

    return {
        "ok": not errors,
        "current_count": sum(record["lifecycle"] == "current" for record in records),
        "archive_count": sum(record["lifecycle"] == "archive" for record in records),
        "sprints": records,
        "errors": errors,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit complete JSON")
    args = parser.parse_args()
    result = check_sprints()
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(
            f"sprints: current {result['current_count']}; "
            f"archived {result['archive_count']}"
        )
        for error in result["errors"]:
            print(f"error: {error}", file=sys.stderr)
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
