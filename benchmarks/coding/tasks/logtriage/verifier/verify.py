from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


LOGS_COMPLEX = [
    '{"ts":"2026-07-02T10:02:00Z","level":"err","service":"api","id":"e2","message":"db down"}',
    'ts=2026-07-02T10:00:00Z level=info service=api id=e1 msg="started"',
    "2026-07-02T10:01:00Z | WARN | worker | id=w1 | queue deep",
    "2026-07-02T10:01:30Z | fatal | worker | id=w2 | job crashed",
    '{"ts":"2026-07-02T10:02:00Z","level":"error","service":"api","id":"e2","message":"duplicate later"}',
    'ts=2026-07-02T10:03:00Z level=DEBUG service=api msg="cache hit"',
    "not a valid log line",
    '{"ts":"2026-07-02T10:04:00Z","level":"mystery","service":"api","message":"bad level"}',
    '{"ts":"2026-07-02T10:05:00Z","level":"info","service":"api"}',
]


class HiddenContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        repo = Path(__file__).resolve().parent
        sys.path.insert(0, str(repo / "src"))
        global parse_logs, query_window, summarize_logs
        from logtriage import parse_logs as pl
        from logtriage import query_window as qw
        from logtriage import summarize_logs as sl

        parse_logs = pl
        query_window = qw
        summarize_logs = sl

    def test_parse_mixed_formats_sorting_and_aliases(self) -> None:
        parsed = parse_logs(LOGS_COMPLEX)
        self.assertEqual([e["id"] for e in parsed["entries"][:4]], ["e1", "w1", "w2", "e2"])
        self.assertEqual([e["severity"] for e in parsed["entries"][:4]], ["INFO", "WARNING", "CRITICAL", "ERROR"])
        self.assertEqual(parsed["duplicate_count"], 1)

    def test_diagnostics_are_stable_and_nonfatal(self) -> None:
        parsed = parse_logs(LOGS_COMPLEX)
        codes = [d["code"] for d in parsed["diagnostics"]]
        self.assertIn("duplicate_log", codes)
        self.assertIn("malformed_line", codes)
        self.assertIn("unknown_severity", codes)
        self.assertIn("missing_field", codes)

    def test_query_window_is_start_inclusive_end_exclusive(self) -> None:
        rows = query_window(LOGS_COMPLEX, "2026-07-02T10:01:00Z", "2026-07-02T10:02:00Z")
        self.assertEqual([row["id"] for row in rows], ["w1", "w2"])

    def test_query_filters_normalize_severity_and_service(self) -> None:
        rows = query_window(LOGS_COMPLEX, "2026-07-02T10:00:00Z", "2026-07-02T10:03:00Z", severity="err", service="api")
        self.assertEqual([row["message"] for row in rows], ["db down"])

    def test_summary_rollups_top_messages_and_bounds(self) -> None:
        lines = [
            'ts=2026-07-02T10:00:00Z level=error service=api id=a1 msg="failed"',
            'ts=2026-07-02T10:00:01Z level=error service=api id=a2 msg="failed"',
            'ts=2026-07-02T10:00:02Z level=warning service=worker id=w1 msg="slow"',
        ]
        summary = summarize_logs(lines)
        self.assertEqual(summary["total"], 3)
        self.assertEqual(summary["by_severity"], {"ERROR": 2, "WARNING": 1})
        self.assertEqual(summary["by_service"], {"api": 2, "worker": 1})
        self.assertEqual(summary["top_messages"], [{"message": "failed", "count": 2}, {"message": "slow", "count": 1}])
        self.assertEqual(summary["first_ts"], "2026-07-02T10:00:00Z")
        self.assertEqual(summary["last_ts"], "2026-07-02T10:00:02Z")

    def test_cli_free_contract_via_import_only(self) -> None:
        repo = Path(__file__).resolve().parent
        env = os.environ.copy()
        env["PYTHONPATH"] = str(repo / "src")
        code = "from logtriage import summarize_logs; print(summarize_logs(['2026-07-02T00:00:00Z | INFO | api | ok'])['total'])"
        out = subprocess.check_output([sys.executable, "-c", code], cwd=repo, env=env, text=True)
        self.assertEqual(out.strip(), "1")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", help="Benchmark repo copy to verify")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    if not (repo / "src" / "logtriage").exists():
        print(json.dumps({"ok": False, "error": f"not a logtriage repo: {repo}"}))
        return 2
    target = repo / ".hidden_verify.py"
    target.write_text(Path(__file__).read_text(encoding="utf-8"), encoding="utf-8")
    try:
        proc = subprocess.run(
            [sys.executable, str(target), "--internal-run"],
            cwd=repo,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=60,
        )
    finally:
        target.unlink(missing_ok=True)
    print(proc.stdout, end="")
    return proc.returncode


def internal_run() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(HiddenContractTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    summary = {"ok": result.wasSuccessful(), "tests_run": result.testsRun, "failures": len(result.failures), "errors": len(result.errors)}
    print("HIDDEN_VERIFIER_SUMMARY", json.dumps(summary, sort_keys=True))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    if "--internal-run" in sys.argv:
        raise SystemExit(internal_run())
    raise SystemExit(main())
