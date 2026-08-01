#!/usr/bin/env python3
"""Reject untested PF Terminal product-contract regressions between manifests."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


@dataclass(frozen=True)
class Difference:
    id: str
    category: str
    message: str
    baseline: Any = None
    candidate: Any = None

    def as_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "category": self.category,
            "message": self.message,
            "baseline": self.baseline,
            "candidate": self.candidate,
        }


def keyed(items: Iterable[dict[str, Any]], field: str) -> dict[str, dict[str, Any]]:
    return {item[field]: item for item in items}


def missing_values(
    differences: list[Difference], category: str, baseline: Iterable[str], candidate: Iterable[str]
) -> None:
    candidate_set = set(candidate)
    for value in sorted(set(baseline) - candidate_set):
        differences.append(
            Difference(
                id=f"{category}:missing:{value}",
                category=category,
                message=f"released {category} entry is missing",
                baseline=value,
            )
        )


def compare_keyed_entries(
    differences: list[Difference],
    category: str,
    baseline: Iterable[dict[str, Any]],
    candidate: Iterable[dict[str, Any]],
    key: str,
    compatibility_fields: Iterable[str],
) -> None:
    baseline_by_key = keyed(baseline, key)
    candidate_by_key = keyed(candidate, key)
    missing_values(differences, category, baseline_by_key, candidate_by_key)
    for name in sorted(baseline_by_key.keys() & candidate_by_key.keys()):
        before = baseline_by_key[name]
        after = candidate_by_key[name]
        for field in compatibility_fields:
            if before.get(field) != after.get(field):
                differences.append(
                    Difference(
                        id=f"{category}:changed:{name}:{field}",
                        category=category,
                        message=f"released {category} field changed",
                        baseline=before.get(field),
                        candidate=after.get(field),
                    )
                )


def compare_migrations(
    differences: list[Difference], baseline: list[dict[str, str]], candidate: list[dict[str, str]]
) -> None:
    baseline_by_path = keyed(baseline, "path")
    candidate_by_path = keyed(candidate, "path")
    missing_values(differences, "migration", baseline_by_path, candidate_by_path)
    for path in sorted(baseline_by_path.keys() & candidate_by_path.keys()):
        before = baseline_by_path[path]["sha256"]
        after = candidate_by_path[path]["sha256"]
        if before != after:
            differences.append(
                Difference(
                    id=f"migration:changed:{path}",
                    category="migration",
                    message="released migration content changed",
                    baseline=before,
                    candidate=after,
                )
            )


def compare_integrations(
    differences: list[Difference], baseline: dict[str, Any], candidate: dict[str, Any]
) -> None:
    missing_values(differences, "integration", baseline, candidate)
    for name in sorted(baseline.keys() & candidate.keys()):
        before = baseline[name]
        after = candidate[name]
        command = after.get("slash_command")
        if command is None or not command.get("dispatch_bindings"):
            differences.append(
                Difference(
                    id=f"integration:unbound:{name}",
                    category="integration",
                    message="protected integration has no slash-command dispatch binding",
                    baseline=before.get("slash_command"),
                    candidate=command,
                )
            )
        missing_values(
            differences,
            f"integration-path:{name}",
            before.get("implementation_paths", []),
            after.get("implementation_paths", []),
        )


def compare_manifests(baseline: dict[str, Any], candidate: dict[str, Any]) -> list[Difference]:
    differences: list[Difference] = []
    before_entries = baseline["entry_points"]
    after_entries = candidate["entry_points"]
    compare_keyed_entries(
        differences,
        "binary",
        before_entries["binaries"],
        after_entries["binaries"],
        "name",
        ("crate",),
    )
    compare_keyed_entries(
        differences,
        "cli",
        before_entries["cli_subcommands"],
        after_entries["cli_subcommands"],
        "command",
        ("variant", "description", "hidden"),
    )
    compare_keyed_entries(
        differences,
        "slash",
        before_entries["tui_slash_commands"],
        after_entries["tui_slash_commands"],
        "command",
        (
            "variant",
            "aliases",
            "description",
            "supports_inline_args",
            "available_during_task",
            "available_in_side_conversation",
        ),
    )
    missing_values(
        differences,
        "config",
        baseline["configuration"]["property_paths"],
        candidate["configuration"]["property_paths"],
    )
    compare_migrations(
        differences,
        baseline["persistence"]["state_migrations"],
        candidate["persistence"]["state_migrations"],
    )
    compare_keyed_entries(
        differences,
        "model",
        baseline["model_catalog"],
        candidate["model_catalog"],
        "slug",
        (
            "input_modalities",
            "context_window",
            "max_context_window",
            "default_reasoning_level",
            "supported_reasoning_levels",
            "service_tiers",
            "available_in_plans",
            "visibility",
            "supported_in_api",
        ),
    )
    missing_values(differences, "app-server", baseline["app_server_methods"], candidate["app_server_methods"])
    for platform, before_paths in baseline["platform_artifacts"].items():
        missing_values(
            differences,
            f"platform:{platform}",
            before_paths,
            candidate["platform_artifacts"].get(platform, []),
        )
    compare_integrations(
        differences, baseline["protected_integrations"], candidate["protected_integrations"]
    )
    return differences


def apply_allowlist(
    differences: list[Difference], allowlist: dict[str, Any] | None, candidate_paths: set[str]
) -> tuple[list[Difference], list[dict[str, Any]], list[str]]:
    entries = (allowlist or {}).get("differences", [])
    by_id = {entry.get("id"): entry for entry in entries}
    accepted: list[dict[str, Any]] = []
    invalid: list[str] = []
    unresolved: list[Difference] = []
    difference_ids = {difference.id for difference in differences}
    for entry_id in sorted(set(by_id) - difference_ids):
        invalid.append(f"allowlist entry does not match a difference: {entry_id}")
    for difference in differences:
        entry = by_id.get(difference.id)
        if entry is None:
            unresolved.append(difference)
            continue
        tests = entry.get("acceptance_tests")
        if not isinstance(tests, list) or not tests:
            invalid.append(f"allowlist entry has no acceptance tests: {difference.id}")
            unresolved.append(difference)
            continue
        missing_tests = sorted(set(tests) - candidate_paths)
        if missing_tests:
            invalid.append(f"allowlist tests do not exist for {difference.id}: {missing_tests}")
            unresolved.append(difference)
            continue
        accepted.append({"difference": difference.as_dict(), "acceptance_tests": sorted(set(tests))})
    return unresolved, accepted, invalid


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--allowlist", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    baseline = load_json(args.baseline)
    candidate = load_json(args.candidate)
    allowlist = load_json(args.allowlist) if args.allowlist else None
    differences = compare_manifests(baseline, candidate)
    unresolved, accepted, invalid = apply_allowlist(
        differences, allowlist, set(candidate.get("source_paths", []))
    )
    report = {
        "schema_version": 1,
        "baseline_source": baseline["source"],
        "candidate_source": candidate["source"],
        "difference_count": len(differences),
        "accepted_difference_count": len(accepted),
        "unresolved_difference_count": len(unresolved),
        "invalid_allowlist_entry_count": len(invalid),
        "accepted_differences": accepted,
        "unresolved_differences": [difference.as_dict() for difference in unresolved],
        "invalid_allowlist_entries": invalid,
    }
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    if unresolved or invalid:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
