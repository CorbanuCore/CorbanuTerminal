"""Check the frozen design crosswalk. JSON manifests; no conformance claim."""

import argparse
import json
from pathlib import Path

from security_level_adversarial import cases
from security_level_evidence import (
    ADAPTERS,
    CATALOG,
    STATUSES,
    EvidenceError,
    candidate_identity,
    checked_artifact,
    digest,
    indexed,
    load_catalog,
    load_json,
    local_path,
    read_bytes,
    require,
    validate_run,
    write_json,
)

KINDS = ("automated", "adversarial", "tui")


def required_assertions(root, subject, kind):
    """Freeze coverage keys; one unrelated passing assertion is not sufficient."""
    catalog = load_catalog(root)
    adapters = indexed(load_json(root / ADAPTERS)["fixtures"], "adapters")
    if kind == "adapters":
        return set(adapters[subject]["expected"]) | set(
            adapters[subject]["contract_tests"]
        )
    if kind == "ingresses":
        return {
            "host-origin",
            "source-kind",
            "unknown-denied",
            "benign-control",
            "forced-classifier-miss",
            "all-sinks",
        }
    if kind == "tui":
        return {"success", "cancel-failure", "recovery", "resume"}
    control = indexed(catalog["controls"], "controls")[subject]
    if kind == "automated":
        return {control["behavior"]} | {
            test
            for adapter in control["adapters"]
            for test in adapters[adapter]["contract_tests"]
        }
    return {
        f"{case['id']}/{dimension}"
        for case in cases(catalog)
        if case["attack"] in control["attacks"]
        for dimension in ("task_integrity", "policy", "confidentiality")
    }


def template(root):
    catalog = load_catalog(root)

    def pending():
        return {"status": "pending", "evidence": None}

    return {
        "schema_version": 1,
        "candidate": None,
        "catalog_sha256": digest(read_bytes(root / CATALOG)),
        "standards_review_date": catalog["standards_review_date"],
        "controls": [
            {**row, "results": {kind: pending() for kind in KINDS}}
            for row in catalog["controls"]
        ],
        "ingresses": [
            {
                "id": row["id"],
                "owner": row["owner"],
                "support": "pending",
                "result": pending(),
            }
            for row in catalog["ingresses"]
        ],
        "adapters": [
            {"id": row["id"], "owners": row["owners"], "result": pending()}
            for row in load_json(root / ADAPTERS)["fixtures"]
        ],
    }


def check_result(
    entry,
    *,
    subject,
    kind,
    evidence_root,
    candidate,
    catalog_sha256,
    not_before,
    planning,
    required,
):
    require(
        isinstance(entry, dict) and set(entry) == {"status", "evidence"},
        "invalid result entry",
    )
    status = entry["status"]
    require(status in STATUSES, "invalid result status")
    if status != "passed":
        if entry["evidence"] is not None:
            checked_artifact(evidence_root, entry["evidence"])
        return status
    require(not planning, "planning manifest cannot contain product passes")
    checked_artifact(evidence_root, entry["evidence"])
    report = load_json(local_path(evidence_root, entry["evidence"]["path"]))
    validate_run(report, candidate, catalog_sha256, not_before)
    require(
        report.get("status") == "passed"
        and report.get("subject") == subject
        and report.get("kind") == kind,
        "failed or mismatched evidence verdict",
    )
    assertions = report.get("assertions")
    require(
        isinstance(assertions, dict)
        and required <= assertions.keys()
        and all(value == "passed" for value in assertions.values()),
        "missing or failed evidence assertions",
    )
    require(
        isinstance(report.get("artifacts"), list) and bool(report["artifacts"]),
        "pass requires captured proof, not a self-reported verdict",
    )
    # Artifact paths are relative to the manifest root, including nested reports.
    for artifact in report["artifacts"]:
        require(bool(checked_artifact(evidence_root, artifact)), "empty proof artifact")
    if kind == "tui":
        require(
            report.get("actual_keys_sent") is True
            and report.get("live_repository") in {"tensorcash", "isometricgame"},
            "TUI evidence requires actual keys and live repository",
        )
    return "passed"


def check(root, path, *, candidate=None, not_before=None, planning=False):
    expected = template(root)
    manifest = load_json(path)
    require(
        set(manifest) == set(expected) and manifest["schema_version"] == 1,
        "unsupported crosswalk schema",
    )
    for key in ("catalog_sha256", "standards_review_date"):
        require(manifest[key] == expected[key], "stale crosswalk contract")
    if planning:
        require(manifest["candidate"] is None, "planning must not claim a candidate")
    else:
        require(
            candidate_identity(manifest["candidate"]) == candidate_identity(candidate),
            "crosswalk candidate does not match selected candidate",
        )
        require(
            not_before is not None, "qualification requires an evidence freshness floor"
        )
    statuses = []

    def result(entry, subject, kind):
        statuses.append(
            check_result(
                entry,
                subject=subject,
                kind=kind,
                evidence_root=path.parent,
                candidate=candidate,
                catalog_sha256=expected["catalog_sha256"],
                not_before=not_before,
                planning=planning,
                required=required_assertions(root, subject, kind),
            )
        )

    for group in ("controls", "ingresses", "adapters"):
        rows = indexed(manifest[group], group)
        expected_rows = indexed(expected[group], group)
        require(rows.keys() == expected_rows.keys(), f"missing/extra {group}")
        for key, baseline in expected_rows.items():
            row = dict(rows[key])
            if group == "controls":
                results = row.pop("results", None)
                baseline = {k: v for k, v in baseline.items() if k != "results"}
                require(
                    isinstance(results, dict) and set(results) == set(KINDS),
                    "control requires automated, adversarial and true-TUI evidence",
                )
                for kind in KINDS:
                    result(results[kind], key, kind)
            else:
                entry = row.pop("result", None)
                baseline = {k: v for k, v in baseline.items() if k != "result"}
                if group == "ingresses":
                    support = row.pop("support", None)
                    baseline.pop("support")
                    require(
                        support in {"pending", "supported", "denied", "unavailable"},
                        "invalid ingress support",
                    )
                    if entry and entry.get("status") == "passed":
                        require(
                            support in {"supported", "denied"},
                            "unavailable ingress cannot pass",
                        )
                    if key == "unknown":
                        require(
                            support != "supported",
                            "unknown origin cannot be a supported authority",
                        )
                result(entry, key, group)
            require(row == baseline, "crosswalk changes frozen control or ownership")
    status = (
        "failed"
        if "failed" in statuses
        else "unavailable"
        if "unavailable" in statuses
        else "pending"
        if "pending" in statuses
        else "passed"
    )
    return {
        "schema_version": 1,
        "phase": "planning" if planning else "qualification",
        "status": status,
        "candidate": candidate,
        "checked_results": len(statuses),
        "qualification": "pending" if planning else status,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--template", type=Path, help="write a new pending JSON manifest"
    )
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--check-plan", action="store_true")
    parser.add_argument("--candidate", type=Path)
    parser.add_argument("--source-commit")
    parser.add_argument("--platform", choices=["linux", "macos", "windows"])
    parser.add_argument("--not-before")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    try:
        if args.template:
            require(
                not any(
                    [
                        args.manifest,
                        args.check_plan,
                        args.candidate,
                        args.source_commit,
                        args.platform,
                        args.not_before,
                    ]
                ),
                "template takes no qualification inputs",
            )
            write_json(args.template, template(root))
            print("crosswalk template created; qualification PENDING")
            return 0
        require(args.manifest is not None, "--manifest required")
        candidate = None
        if args.check_plan:
            require(
                not any(
                    [args.candidate, args.source_commit, args.platform, args.not_before]
                ),
                "planning takes no qualification inputs",
            )
        else:
            require(
                all(
                    [args.candidate, args.source_commit, args.platform, args.not_before]
                ),
                "qualification requires candidate and freshness inputs",
            )
            from security_level_compat import sha256_file

            candidate = {
                "source_commit": args.source_commit,
                "binary_sha256": sha256_file(args.candidate),
                "platform": args.platform,
            }
        report = check(
            root,
            args.manifest,
            candidate=candidate,
            not_before=args.not_before,
            planning=args.check_plan,
        )
        print(json.dumps(report, sort_keys=True))
        return 0 if args.check_plan or report["status"] == "passed" else 1
    except (EvidenceError, OSError, ValueError, KeyError, TypeError):
        print("security-level-standards-check: invalid or incomplete evidence")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
