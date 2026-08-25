#!/usr/bin/env python3
"""Validate Corbanu Terminal plan lifecycle and active-plan metadata."""

import argparse
import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
LIFECYCLE_STATUS = {
    "proposed": "draft",
    "active": "active",
    "completed": "completed",
    "cancelled": "cancelled",
}
ACTIVE_LIMIT = 2
REQUIRED_ACTIVE_KEYS = (
    "title",
    "status",
    "change_class",
    "priority",
    "owner",
    "activation_authority",
    "activation_basis",
    "target_release",
    "deadline",
    "created",
    "updated",
)
REQUIRED_ACTIVE_SECTIONS = (
    "Activation record",
    "User pain",
    "Product intent and ideal flow",
    "Product linkage",
    "Scope",
    "Invariants",
    "Ownership and implementation worktrees",
    "Useful code references",
    "Sprint execution map",
    "Acceptance flows",
    "Implementation sequence",
    "Automated evidence",
    "True-TUI evidence",
    "Live-repository applicability",
    "Human acceptance",
    "Documentation",
    "Dependencies, decisions, and blockers",
    "Release linkage",
    "Completion",
)


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


def nested_value(front, parent, key):
    block = re.search(
        rf"^{re.escape(parent)}:\s*$((?:\r?\n[ \t]+.*)*)",
        front,
        re.MULTILINE,
    )
    if block is None:
        return None
    value = re.search(
        rf"^[ \t]+(?:-[ \t]+)?{re.escape(key)}:[ \t]*(.*?)\s*$",
        block.group(1),
        re.MULTILINE,
    )
    return scalar(value.group(1)) if value is not None else None


def check_plan_root(root=ROOT):
    errors = []
    records = []
    for directory, expected_status in LIFECYCLE_STATUS.items():
        lifecycle = root / directory
        if not lifecycle.is_dir():
            errors.append(f"missing lifecycle directory: {lifecycle}")
            continue
        for path in sorted(lifecycle.glob("*.md")):
            if path.name in {"README.md", "index.md"}:
                continue
            text, front_values, front = parse_front_matter(path)
            relative = path.relative_to(root).as_posix()
            status = front_values.get("status")
            records.append(
                {
                    "path": relative,
                    "directory": directory,
                    "status": status,
                    "title": front_values.get("title"),
                    "priority": front_values.get("priority"),
                    "owner": front_values.get("owner"),
                }
            )
            if not front:
                errors.append(f"{relative}: missing YAML front matter")
                continue
            if status != expected_status:
                errors.append(
                    f"{relative}: status {status!r} does not match directory "
                    f"{directory!r} ({expected_status!r} required)"
                )
            if directory == "active":
                errors.extend(check_active(path, text, front_values, front, root))

    active = [record for record in records if record["directory"] == "active"]
    if len(active) > ACTIVE_LIMIT:
        errors.append(
            f"active-plan limit exceeded: found {len(active)}, maximum is {ACTIVE_LIMIT}"
        )
    return {
        "ok": not errors,
        "active_limit": ACTIVE_LIMIT,
        "active_count": len(active),
        "available_slots": max(0, ACTIVE_LIMIT - len(active)),
        "plans": records,
        "errors": errors,
    }


def check_active(path, text, values, front, root):
    relative = path.relative_to(root).as_posix()
    errors = []
    for key in REQUIRED_ACTIVE_KEYS:
        value = values.get(key, "")
        if not value or "<" in value or ">" in value:
            errors.append(f"{relative}: active plan requires concrete {key!r}")

    if values.get("change_class") != "product-initiative":
        errors.append(f"{relative}: active plan must be a product-initiative")
    if values.get("priority") not in {"P0", "P1", "P2"}:
        errors.append(f"{relative}: priority must be P0, P1, or P2")

    product_file = nested_value(front, "product_spec", "file")
    heading = nested_value(front, "product_spec", "heading")
    excerpt = nested_value(front, "product_spec", "requirement_excerpt")
    if product_file != "docs/corbanu-product-spec.md":
        errors.append(f"{relative}: product_spec.file must name the canonical specification")
    if not heading or "<" in heading:
        errors.append(f"{relative}: product_spec.heading must be concrete")
    if not excerpt or "<" in excerpt:
        errors.append(f"{relative}: product_spec.requirement_excerpt must be concrete")

    worktree_path = nested_value(front, "implementation_worktrees", "path")
    branch = nested_value(front, "implementation_worktrees", "branch")
    base_commit = nested_value(front, "implementation_worktrees", "base_commit")
    if not worktree_path or "<" in worktree_path:
        errors.append(f"{relative}: an exact implementation worktree path is required")
    if not branch or "<" in branch:
        errors.append(f"{relative}: an implementation branch is required")
    if not base_commit or re.fullmatch(r"[0-9a-f]{40}", base_commit) is None:
        errors.append(f"{relative}: base_commit must be a 40-character lowercase Git hash")

    sections = set(re.findall(r"^## (.+?)\s*$", text, re.MULTILINE))
    for section in REQUIRED_ACTIVE_SECTIONS:
        if section not in sections:
            errors.append(f"{relative}: missing required section {section!r}")
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit the complete result as JSON")
    args = parser.parse_args()
    result = check_plan_root()
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(
            f"plans: active {result['active_count']}/{result['active_limit']}; "
            f"available slots {result['available_slots']}"
        )
        for error in result["errors"]:
            print(f"error: {error}", file=sys.stderr)
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
