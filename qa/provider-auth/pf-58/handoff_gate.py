#!/usr/bin/env python3
"""Run current-candidate TMUX suites and fail closed on checklist coverage gaps."""

import argparse
from html import escape
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import subprocess
import time
import xml.etree.ElementTree as ET

from handoff_preflight import digest


SUITES = {
    "provider-management": "test(suite::provider_management::tmux_)",
    "reauthentication": "test(suite::provider_management::reauthentication::tmux_)",
    "provider-convergence": "test(suite::provider_convergence::tmux_)",
    "multi-provider": "test(suite::multi_provider_onboarding::tmux_)",
    "claude-auth": "test(suite::claude_auth::tmux_)",
    "stream": "test(suite::output_text_stream::tmux_)",
    "security": "test(suite::security_profiles::tmux_)",
    "memory": "test(suite::memory_human_fixture::tmux_) | test(suite::memory_stage_one_policy::tmux_)",
}


class Checklist(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "input" and "data-check" in attrs:
            self.ids.append(attrs["data-check"])


def coverage(html, matrix):
    parser = Checklist()
    parser.feed(html)
    if len(parser.ids) != len(set(parser.ids)) or set(parser.ids) != set(matrix):
        raise ValueError("humanTest.html checklist and coverage matrix disagree")
    for item in matrix.values():
        if not item["suites"] or any(
            s not in {*SUITES, "packaged-runtime"} for s in item["suites"]
        ):
            raise ValueError("unknown or missing suite")
    return parser.ids


def junit_summary(path):
    cases = ET.parse(path).findall(".//testcase")
    failed = [
        c.attrib["name"]
        for c in cases
        if c.find("failure") is not None or c.find("error") is not None
    ]
    skipped = [c.attrib["name"] for c in cases if c.find("skipped") is not None]
    return {
        "passed": bool(cases) and not failed and not skipped,
        "count": len(cases),
        "failed": failed,
        "skipped": skipped,
    }


def assessment(matrix, results, runtime_ok):
    return {
        key: {
            "ready": runtime_ok
            and all(results.get(s, {}).get("passed", False) for s in item["suites"])
            and not item["remaining"],
            "suites": item["suites"],
            "remaining": item["remaining"],
        }
        for key, item in matrix.items()
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--runtime-report", type=Path, required=True)
    parser.add_argument("--tools-report", type=Path, required=True)
    parser.add_argument(
        "--suite",
        action="append",
        choices=tuple(SUITES),
        help="Diagnostic subset only; unexecuted groups still block handoff",
    )
    args = parser.parse_args()
    candidate = args.candidate.resolve(strict=True)
    args.evidence.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).parent
    matrix = json.loads((root / "checklist_coverage.json").read_text())
    ids = coverage((args.repo / "humanTest.html").read_text(), matrix)
    sha = digest(candidate)
    runtime = json.loads(args.runtime_report.read_text())
    packaged = json.loads(args.tools_report.read_text())
    runtime_sha = next(
        c["detail"]["sha256"] for c in runtime["checks"] if c["check"] == "codex"
    )
    if runtime_sha != sha or packaged.get("sha256") != sha:
        raise ValueError("reports belong to a different candidate")
    results = {"packaged-runtime": packaged}
    report = {
        "candidate": str(candidate),
        "sha256": sha,
        "host_sha256": digest(candidate.with_name("codex-code-mode-host")),
        "coverage_sha256": digest(root / "checklist_coverage.json"),
        "checklist_sha256": digest(args.repo / "humanTest.html"),
        "human_acceptance": False,
        "ready": False,
        "suites": results,
    }
    env = dict(
        os.environ,
        CORBANU_TMUX_REQUIRED="1",
        CORBANU_TEST_NO_NATIVE_KEYRING="1",
        CARGO_BIN_EXE_codex=str(candidate),
        CARGO_BIN_EXE_corbanu=str(candidate),
        CARGO_BIN_EXE_pfterminal=str(candidate),
        RUST_MIN_STACK="8388608",
    )
    target = Path(env["CARGO_TARGET_DIR"])
    try:
        for suite, expression in SUITES.items():
            if args.suite and suite not in args.suite:
                continue
            started = time.time()
            env["CORBANU_TMUX_ARTIFACT_DIR"] = str(args.evidence / suite / "tmux")
            env["CORBANU_SECURITY_UI_EVIDENCE"] = str(
                args.evidence / suite / "security"
            )
            with (args.evidence / f"{suite}.log").open("x") as log:
                code = subprocess.run(
                    [
                        "just",
                        "test",
                        "-p",
                        "codex-tui",
                        "--test",
                        "all",
                        "-E",
                        expression,
                        "--retries",
                        "0",
                        "--test-threads",
                        "1",
                    ],
                    cwd=args.repo,
                    env=env,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                ).returncode
            junit = target / "nextest/local/junit.xml"
            if junit.exists() and junit.stat().st_mtime >= started:
                raw = junit.read_bytes()
                saved = args.evidence / f"{suite}.xml"
                saved.write_bytes(raw)
                results[suite] = junit_summary(saved)
            else:
                results[suite] = {
                    "passed": False,
                    "reason": "missing fresh JUnit report",
                }
            results[suite]["passed"] &= code == 0
            results[suite]["exit_code"] = code
            print(suite, "PASS" if results[suite]["passed"] else "BLOCKED", flush=True)
            (args.evidence / "partial.json").write_text(
                json.dumps(report, indent=2) + "\n"
            )
    finally:
        # A passing report for another host executable must never bless this package.
        runtime_host = next(
            c["detail"]["sha256"]
            for c in runtime["checks"]
            if c["check"] == "codex-code-mode-host"
        )
        runtime_ok = (
            runtime.get("passed", False)
            and runtime_host == report["host_sha256"]
            and packaged.get("host_sha256") == report["host_sha256"]
        )
        wallet = candidate.with_name("pfterminal-walletd")
        wallet_check = next(
            (c for c in runtime["checks"] if c["check"] == "pfterminal-walletd"), {}
        )
        runtime_ok = (
            runtime_ok
            and wallet.is_file()
            and wallet_check.get("passed", False)
            and wallet_check.get("detail", {}).get("sha256") == digest(wallet)
        )
        report["checks"] = assessment(matrix, results, runtime_ok)
        report["ready"] = (
            all(c["ready"] for c in report["checks"].values())
            and digest(candidate) == sha
        )
        (args.evidence / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        rows = "".join(
            f"<tr><td>{i + 1}. {escape(key)}</td><td>{'Automated baseline passed' if report['checks'][key]['ready'] else 'BLOCKED / incomplete'}</td>"
            f"<td>{escape('; '.join(matrix[key]['remaining']) or 'See suite results; all must pass.')}</td></tr>"
            for i, key in enumerate(ids)
        )
        (args.evidence / "report.html").write_text(
            '<!doctype html><meta charset="utf-8"><title>Candidate readiness</title>'
            "<style>body{font:17px system-ui;max-width:1100px;margin:40px auto;background:#141820;color:#eee}td,th{padding:14px;border:1px solid #58616b}table{border-collapse:collapse}h1{color:#ffcc77}</style>"
            "<h1>Human handoff: "
            + ("READY" if report["ready"] else "BLOCKED")
            + "</h1><p>Exact candidate SHA-256: "
            + sha
            + "</p><p>Automated evidence is not human acceptance. Historical results do not qualify this build.</p><table>"
            "<tr><th>Human check</th><th>Machine readiness</th><th>Outstanding evidence</th></tr>"
            + rows
            + "</table>"
        )
    raise SystemExit(0 if report["ready"] else 1)


if __name__ == "__main__":
    main()
