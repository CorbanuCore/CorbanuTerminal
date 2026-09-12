#!/usr/bin/env python3
"""Verify incremental feature-delivery evidence; never authorize a release."""
import argparse
import hashlib
import json
from pathlib import Path
import re

from control import read_file, read_json

BOUNDARIES = {"discovery", "commands", "tools", "api", "jobs", "resume", "migrations"}
MERGE_CASES = {"off_visibility", "off_execution", "compatibility"}
ENABLE_CASES = MERGE_CASES | {"on_success", "on_failure", "on_cancel", "disable_after_use", "restart_resume", "auth_recovery"}


def digest(repo, paths):
    if not paths or len(paths) != len(set(paths)):
        raise ValueError("candidate file list must be nonempty and unique")
    values = {}
    for name in sorted(paths):
        if Path(name).is_absolute() or ".." in Path(name).parts:
            raise ValueError("candidate paths must be repository relative")
        values[name] = hashlib.sha256(read_file(repo / name, repo).encode()).hexdigest()
    return hashlib.sha256(json.dumps(values, sort_keys=True).encode()).hexdigest()


def check(repo, contract, phase):
    errors = []
    if set(contract.get("boundaries", {})) != BOUNDARIES or not all(isinstance(v, str) and len(v.strip()) >= 12 for v in contract.get("boundaries", {}).values()):
        errors.append("Every entry boundary needs a concrete gating or non-applicability explanation")
    actual = digest(repo, contract["candidate_files"])
    if actual != contract.get("candidate_tree_sha256"):
        errors.append("candidate tree differs from reviewed evidence")
    registry = "codex-rs/features/src/lib.rs"
    if contract.get("mode") == "feature-flagged":
        if registry not in contract["candidate_files"]:
            errors.append("native flag registry must be part of candidate identity")
        source = read_file(repo / registry, repo)
        key = re.escape(contract.get("flag", ""))
        blocks = re.findall(r"FeatureSpec\s*\{(.*?)\n    \}", source, re.S)
        matching = [b for b in blocks if re.search(rf'key:\s*"{key}"\s*,', b)]
        if len(matching) != 1 or not re.search(r"default_enabled:\s*false\s*,", matching[0]) or not re.search(r"stage:\s*Stage::UnderDevelopment\s*,", matching[0]):
            errors.append("feature must have one UnderDevelopment, literal default-false native flag")
    elif contract.get("mode") != "internal-only" or not contract.get("internal_boundary"):
        errors.append("use feature-flagged mode or justify an internal-only non-runtime boundary")
    cases = contract.get("tests", {})
    for name in (ENABLE_CASES if phase == "enable" else MERGE_CASES):
        test = cases.get(name, {})
        if test.get("result") != "pass" or test.get("candidate_tree_sha256") != actual:
            errors.append(f"{name}: missing passing evidence for exact candidate")
            continue
        try:
            artifact = hashlib.sha256(read_file(repo / test["artifact"], repo).encode()).hexdigest()
            if artifact != test.get("artifact_sha256"):
                errors.append(f"{name}: evidence artifact changed")
        except (KeyError, ValueError, OSError):
            errors.append(f"{name}: missing/invalid evidence artifact")
    if phase == "enable":
        acceptance = contract.get("human_acceptance", {})
        if not acceptance.get("tester") or acceptance.get("result") != "pass" or acceptance.get("candidate_tree_sha256") != actual:
            errors.append("named human acceptance for exact candidate is missing")
        if not contract.get("enablement_decision"):
            errors.append("explicit product enablement decision is missing")
    return errors


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--contract", type=Path, required=True)
    parser.add_argument("--phase", choices=("merge", "enable"), required=True)
    args = parser.parse_args()
    errors = check(args.repo.resolve(), read_json(args.contract, args.repo), args.phase)
    print(json.dumps({"ok": not errors, "errors": errors, "release_authorized": False}, indent=2))
    raise SystemExit(bool(errors))
