#!/usr/bin/env python3
from __future__ import annotations

import argparse
import filecmp
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


TASK_ROOT = Path(__file__).resolve().parents[1]
PRISTINE_TESTS = TASK_ROOT / "tests_pristine"
BUG_PROBES = [
    "BugProbeTests.test_probe_bug1_refresh_returns_json_ready_body",
    "BugProbeTests.test_probe_bug2_permission_revoke_invalidates_cache_before_next_check",
    "BugProbeTests.test_probe_bug3_router_static_route_precedes_dynamic_param",
    "BugProbeTests.test_probe_bug4_context_resets_between_requests",
    "BugProbeTests.test_probe_bug5_rate_limit_key_uses_authenticated_user_not_token",
]


def run(cmd: list[str], cwd: Path, env: dict[str, str], timeout: int = 120) -> dict[str, Any]:
    try:
        proc = subprocess.run(
            cmd,
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout,
        )
        return {"returncode": proc.returncode, "stdout": proc.stdout, "timed_out": False}
    except subprocess.TimeoutExpired as exc:
        stdout = (exc.stdout or "") if isinstance(exc.stdout, str) else ""
        return {"returncode": 124, "stdout": stdout, "timed_out": True}


def compare_tests(workspace: Path) -> dict[str, Any]:
    tests = workspace / "tests"
    modified: list[str] = []
    missing: list[str] = []
    extra: list[str] = []
    pristine_files = {p.relative_to(PRISTINE_TESTS) for p in PRISTINE_TESTS.rglob("*.py")}
    workspace_files = {p.relative_to(tests) for p in tests.rglob("*.py")} if tests.exists() else set()
    for rel in sorted(pristine_files):
        candidate = tests / rel
        pristine = PRISTINE_TESTS / rel
        if not candidate.exists():
            missing.append(str(rel))
            continue
        if not filecmp.cmp(pristine, candidate, shallow=False):
            modified.append(str(rel))
    for rel in sorted(workspace_files - pristine_files):
        extra.append(str(rel))
    return {
        "modified": modified,
        "missing": missing,
        "extra": extra,
        "test_modified": bool(modified or extra or ("test_visible_integration.py" in missing)),
    }


def restore_pristine_tests(workspace: Path) -> None:
    tests = workspace / "tests"
    if tests.exists():
        shutil.rmtree(tests)
    shutil.copytree(PRISTINE_TESTS, tests)


def parse_unittest(stdout: str, returncode: int) -> dict[str, Any]:
    total = 0
    failures = 0
    errors = 0
    match = re.search(r"Ran (\d+) tests?", stdout)
    if match:
        total = int(match.group(1))
    fail_match = re.search(r"FAILED \(([^)]*)\)", stdout)
    if fail_match:
        for part in fail_match.group(1).split(","):
            if "=" not in part:
                continue
            key, value = part.strip().split("=", 1)
            if key == "failures":
                failures = int(value)
            elif key == "errors":
                errors = int(value)
    ok = returncode == 0 and "OK" in stdout
    passed = total if ok else max(total - failures - errors, 0)
    return {"ok": ok, "passed": passed, "total": total, "failures": failures, "errors": errors}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("workspace")
    args = parser.parse_args()
    workspace = Path(args.workspace).resolve()
    if not (workspace / "src" / "apigate").exists():
        summary = {
            "ok": False,
            "error": f"not an apigate workspace: {workspace}",
            "tests_passed": 0,
            "tests_total": 34,
            "bug_probes_passed": 0,
            "bug_probes_total": 5,
            "test_modified": True,
        }
        print("DEBUGTYPE2_VERIFIER_SUMMARY", json.dumps(summary, sort_keys=True))
        return 2

    diff = compare_tests(workspace)
    restore_pristine_tests(workspace)
    env = os.environ.copy()
    env["PYTHONPATH"] = str(workspace / "src")
    full = run([sys.executable, "-m", "unittest", "discover", "-s", "tests", "-v"], workspace, env, timeout=180)
    full_summary = parse_unittest(full["stdout"], int(full["returncode"]))

    bug_results: list[dict[str, Any]] = []
    for probe in BUG_PROBES:
        result = run([sys.executable, "tests/test_bug_probes.py", probe], workspace, env, timeout=60)
        parsed = parse_unittest(result["stdout"], int(result["returncode"]))
        bug_results.append({"probe": probe, "ok": parsed["ok"], "stdout_tail": result["stdout"].splitlines()[-8:]})
    bug_fixed = sum(1 for item in bug_results if item["ok"])
    ok = bool(full_summary["ok"] and bug_fixed == len(BUG_PROBES) and not diff["test_modified"])
    summary = {
        "ok": ok,
        "tests_passed": full_summary["passed"],
        "tests_total": full_summary["total"],
        "bug_probes_passed": bug_fixed,
        "bug_probes_total": len(BUG_PROBES),
        "test_modified": diff["test_modified"],
        "test_diff": diff,
        "full_suite": full_summary,
        "bug_results": bug_results,
    }
    print(full["stdout"], end="")
    print("DEBUGTYPE2_VERIFIER_SUMMARY", json.dumps(summary, sort_keys=True))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
