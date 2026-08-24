from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import unittest
from pathlib import Path


class HiddenContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        repo = Path(__file__).resolve().parent
        sys.path.insert(0, str(repo / "src"))
        global sliding_window, token_bucket
        from rategate import sliding_window as sw
        from rategate import token_bucket as tb

        sliding_window = sw
        token_bucket = tb

    def test_token_bucket_refill_capacity_and_multi_key(self) -> None:
        result = token_bucket(
            [
                {"id": "a", "key": "u1", "ts": 0, "cost": 2},
                {"id": "b", "key": "u1", "ts": 0, "cost": 2},
                {"id": "c", "key": "u2", "ts": 0, "cost": 3},
                {"id": "d", "key": "u1", "ts": 2, "cost": 2},
            ],
            rate=1,
            capacity=3,
        )
        self.assertEqual([d["allowed"] for d in result["decisions"]], [True, False, True, True])
        self.assertEqual(result["decisions"][-1]["tokens_remaining"], 1.0)

    def test_token_bucket_rejected_requests_do_not_consume(self) -> None:
        result = token_bucket(
            [
                {"id": "a", "key": "u", "ts": 0, "cost": 2},
                {"id": "b", "key": "u", "ts": 0, "cost": 2},
                {"id": "c", "key": "u", "ts": 1, "cost": 1},
            ],
            rate=1,
            capacity=2,
        )
        self.assertEqual([d["allowed"] for d in result["decisions"]], [True, False, True])
        self.assertEqual(result["decisions"][2]["tokens_remaining"], 0.0)

    def test_sliding_window_half_open_boundary(self) -> None:
        result = sliding_window(
            [
                {"id": "a", "key": "ip", "ts": 0},
                {"id": "b", "key": "ip", "ts": 10},
                {"id": "c", "key": "ip", "ts": 10.001},
            ],
            limit=1,
            window_seconds=10,
        )
        self.assertEqual([d["allowed"] for d in result["decisions"]], [True, True, False])
        self.assertEqual([d["window_count"] for d in result["decisions"]], [1, 1, 1])

    def test_iso_timestamps_sort_before_input_order_ties(self) -> None:
        result = sliding_window(
            [
                {"id": "late", "key": "k", "ts": "2026-07-02T00:00:10Z"},
                {"id": "early", "key": "k", "ts": "2026-07-02T00:00:00Z"},
                {"id": "tie", "key": "k", "ts": "2026-07-02T00:00:10Z"},
            ],
            limit=10,
            window_seconds=60,
        )
        self.assertEqual([d["id"] for d in result["decisions"]], ["early", "late", "tie"])

    def test_invalid_requests_are_diagnostics_and_rejections(self) -> None:
        result = token_bucket(
            [
                {"id": "missing-key", "ts": 0},
                {"id": "bad-cost", "key": "u", "ts": 0, "cost": -1},
                {"id": "bad-ts", "key": "u", "ts": "not-time"},
            ],
            rate=1,
            capacity=2,
        )
        self.assertEqual([d["allowed"] for d in result["decisions"]], [False, False, False])
        codes = [d["code"] for d in result["diagnostics"]]
        self.assertIn("missing_key", codes)
        self.assertIn("invalid_cost", codes)
        self.assertIn("invalid_timestamp", codes)

    def test_import_contract_in_subprocess(self) -> None:
        repo = Path(__file__).resolve().parent
        env = os.environ.copy()
        env["PYTHONPATH"] = str(repo / "src")
        code = "from rategate import token_bucket; print(token_bucket([{'key':'x','ts':0}], 1, 1)['decisions'][0]['allowed'])"
        out = subprocess.check_output([sys.executable, "-c", code], cwd=repo, env=env, text=True)
        self.assertEqual(out.strip(), "True")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", help="Benchmark repo copy to verify")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    if not (repo / "src" / "rategate").exists():
        print(json.dumps({"ok": False, "error": f"not a rategate repo: {repo}"}))
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
