#!/usr/bin/env python3
"""Read-only traceability gate; evidence truth/independence still need review."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys


def require(condition, message):
    if not condition:
        raise ValueError(message)


def text(value, label):
    require(isinstance(value, str) and bool(value.strip()), f"missing {label}")


def sha(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            checksum.update(block)
    return checksum.hexdigest()


def artifact(reference, root):
    require(isinstance(reference, dict), "missing artifact reference")
    text(reference.get("path"), "artifact path")
    relative = Path(reference["path"])
    require(not relative.is_absolute(), "artifact path must be relative")
    path = (root / relative).resolve()
    require(path.is_relative_to(root.resolve()), "artifact escapes record directory")
    require(path.is_file(), f"missing artifact: {relative}")
    require(sha(reference.get("sha256")), f"invalid artifact hash: {relative}")
    require(digest(path) == reference["sha256"], f"artifact hash mismatch: {relative}")


def approval(value, root, implementer):
    require(isinstance(value, dict), "missing recorded product-authority approval")
    text(value.get("by"), "approval owner")
    require(value["by"] != implementer, "implementer cannot approve own exclusion or budget extension")
    text(value.get("reason"), "approval reason")
    artifact(value.get("artifact"), root)


def indexed(cases):
    require(isinstance(cases, list) and cases, "cases must be a nonempty list")
    result = {}
    for case in cases:
        require(isinstance(case, dict), "invalid case")
        text(case.get("id"), "case ID")
        require(case["id"] not in result, f"duplicate case: {case['id']}")
        result[case["id"]] = case
    return result


def read_json(path):
    # Duplicate JSON keys must not silently discard a case list or disposition.
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, f"duplicate JSON key: {key}")
            result[key] = value
        return result

    value = json.loads(path.read_text(), object_pairs_hook=unique)
    require(isinstance(value, dict), "record must be an object")
    return value


def execution(case, root, implementer, designer, design_hash, candidate_hash):
    run = case.get("execution")
    require(isinstance(run, dict), "independent execution required")
    for field in ("agent", "run_id", "model", "machine", "profile", "launcher"):
        text(run.get(field), f"execution {field}")
    require(run["agent"] not in {implementer, designer},
            "executor must differ from implementer and designer")
    for field in ("fresh_context", "code_blind", "results_blind"):
        require(run.get(field) is True, f"execution must attest {field}")
    for field in ("packet", "access_record", "actions"):
        artifact(run.get(field), root)
    reference = run.get("isolation_record")
    artifact(reference, root)
    isolation = read_json(root / reference["path"])
    require(isolation.get("enforcement") == "os-enforced",
            "execution requires OS-enforced isolation")
    for field, expected in (("agent", run["agent"]), ("run_id", run["run_id"]),
                            ("case_id", case["id"]), ("design_sha256", design_hash),
                            ("candidate_sha256", candidate_hash)):
        require(isolation.get(field) == expected, f"isolation binding mismatch: {field}")
    for field in ("policy", "tool_inventory", "probe_evidence"):
        artifact(isolation.get(field), root)
    # These are auditable claims, never a substitute for inspecting actual probes.
    for field in ("source_denied", "history_denied", "symlink_escape_denied",
                  "credentials_denied", "cross_run_ipc_denied", "network_restricted",
                  "package_readonly", "children_confined", "packet_readable",
                  "candidate_launchable", "actual_input_available"):
        require(isolation.get(field) is True, f"isolation prerequisite missing: {field}")
    return run["agent"]


def check(design_path, results_path, candidate_path):
    design, results = read_json(design_path), read_json(results_path)
    source_root, result_root = design_path.parent, results_path.parent
    require(type(results.get("schema_version")) is int and results["schema_version"] == 2,
            "schema 2 required; legacy evidence cannot qualify isolated execution")
    for field in ("feature", "designer"):
        text(design.get(field), field)
    for field in ("fresh_context", "code_blind", "results_blind"):
        require(design.get(field) is True, f"design must attest {field}")
    require(design.get("isolation") in {"restricted", "instruction-only"}, "unknown isolation")
    for field in ("brief", "proposal", "access_record"):
        artifact(design.get(field), source_root)
    screenshots = design.get("screenshots")
    require(isinstance(screenshots, list) and screenshots, "screenshots required")
    for screenshot in screenshots:
        artifact(screenshot, source_root)
    proposals = indexed(design.get("cases"))
    for case in proposals.values():
        require(case.get("priority") in {"blocker", "advisory"}, "unknown priority")
        text(case.get("starting_conditions"), "starting conditions")
        for field in ("actions", "expected"):
            values = case.get(field)
            require(isinstance(values, list) and values, f"missing {field}")
            for value in values:
                text(value, field)
    require(results.get("design_sha256") == digest(design_path), "frozen design hash mismatch")
    text(results.get("implementer"), "implementer")
    require(results["implementer"] != design["designer"], "designer cannot be implementer")
    candidate = results.get("candidate")
    require(isinstance(candidate, dict), "candidate required")
    for field in ("version", "source", "platform"):
        text(candidate.get(field), f"candidate {field}")
    actual_hash = digest(candidate_path)
    require(candidate.get("binary_sha256") == actual_hash, "delivered candidate hash mismatch")
    artifact(candidate.get("package_manifest"), result_root)
    outcomes = indexed(results.get("cases"))
    require(proposals.keys() == outcomes.keys(), "missing or unproposed case IDs")
    executors = set()
    for case in outcomes.values():
        text(case.get("summary"), "case outcome summary")
        disposition = case.get("disposition")
        if disposition == "out_of_scope":
            approval(case.get("approval"), result_root, results["implementer"])
            continue
        require(disposition == "passed", f"unresolved {case['id']}: {disposition}")
        require(case.get("candidate_sha256") == actual_hash, f"stale case: {case['id']}")
        executors.add(execution(case, result_root, results["implementer"],
                                design["designer"], digest(design_path), actual_hash))
        require(case.get("method") in {"tmux", "native-ui", "automated", "manual"}, "unknown method")
        evidence = case.get("evidence")
        require(isinstance(evidence, list) and evidence, "passed case needs execution evidence")
        for reference in evidence:
            artifact(reference, result_root)
    review = results.get("evidence_check")
    require(isinstance(review, dict), "independent evidence check required")
    text(review.get("agent"), "evidence-check agent")
    require(review["agent"] != results["implementer"], "implementer cannot self-approve evidence")
    require(review["agent"] not in executors, "executor cannot self-approve evidence")
    require(review.get("verdict") == "pass", "evidence check did not pass")
    artifact(review.get("artifact"), result_root)
    budget = results.get("review_budget")
    require(isinstance(budget, dict), "review budget required")
    require(type(budget.get("used")) is int and budget["used"] >= 2, "count design and evidence passes")
    require(type(budget.get("limit")) is int and budget["limit"] > 0, "invalid review limit")
    artifact(budget.get("ledger"), result_root)
    if budget["used"] > budget["limit"] or budget["limit"] > 5:
        approval(budget.get("extension"), result_root, results["implementer"])
    return len(proposals)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("design", type=Path)
    parser.add_argument("results", type=Path)
    parser.add_argument("candidate", type=Path, help="exact packaged executable being handed off")
    args = parser.parse_args()
    try:
        count = check(args.design, args.results, args.candidate)
    except (OSError, ValueError, TypeError, KeyError) as error:
        print(f"BLOCKED: {error}", file=sys.stderr)
        return 1
    print(f"Traceability gate passed: {count} cases; independent evidence review recorded. Human acceptance is separate.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
