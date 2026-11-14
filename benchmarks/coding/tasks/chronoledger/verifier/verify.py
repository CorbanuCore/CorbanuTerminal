from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


def txn(txn_id: str, ts: str, legs: list[dict], key: str | None = None) -> dict:
    out = {"txn_id": txn_id, "ts": ts, "legs": legs}
    if key is not None:
        out["idempotency_key"] = key
    return out


def leg(account: str, currency: str, side: str, amount: str) -> dict:
    return {"account": account, "currency": currency, "side": side, "amount": amount}


class HiddenContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        repo = Path(__file__).resolve().parent
        sys.path.insert(0, str(repo / "src"))
        global ChronoLedgerEngine
        from chronoledger import ChronoLedgerEngine as engine_cls

        ChronoLedgerEngine = engine_cls

    def make_engine(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        return ChronoLedgerEngine(Path(tmp.name) / "ledger.wal")

    def test_01_multi_currency_multi_leg_balances(self) -> None:
        engine = self.make_engine()
        result = engine.post_transaction(
            txn(
                "multi-1",
                "2026-07-02T10:00:00Z",
                [
                    leg("cash", "USD", "debit", "100.00"),
                    leg("receivable", "USD", "debit", "25.50"),
                    leg("revenue", "USD", "credit", "125.50"),
                    leg("euro_cash", "EUR", "debit", "7.25"),
                    leg("euro_revenue", "EUR", "credit", "7.25"),
                ],
                "key-multi-1",
            )
        )
        self.assertTrue(result["ok"])
        self.assertEqual(engine.balance_as_of("cash", "USD"), "100.00")
        self.assertEqual(engine.balance_as_of("receivable", "USD"), "25.50")
        self.assertEqual(engine.balance_as_of("revenue", "USD"), "-125.50")
        self.assertEqual(engine.balance_as_of("euro_cash", "EUR"), "7.25")

    def test_02_cross_currency_imbalance_rejected_per_currency(self) -> None:
        engine = self.make_engine()
        good = engine.post_transaction(txn("seed", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "10.00"), leg("equity", "USD", "credit", "10.00")]))
        self.assertTrue(good["ok"])
        bad = engine.post_transaction(
            txn(
                "bad-fx",
                "2026-07-02T10:01:00Z",
                [
                    leg("cash", "USD", "debit", "5.00"),
                    leg("equity", "EUR", "credit", "5.00"),
                ],
            )
        )
        self.assertFalse(bad["ok"])
        self.assertEqual(bad["error"], "imbalanced_transaction")
        self.assertEqual(engine.balance_as_of("cash", "USD"), "10.00")
        self.assertEqual(engine.balance_as_of("equity", "EUR"), "0.00")

    def test_03_atomicity_invalid_amount_does_not_write_or_mutate(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            wal = Path(tmp) / "ledger.wal"
            engine = ChronoLedgerEngine(wal)
            before_size = wal.stat().st_size if wal.exists() else 0
            result = engine.post_transaction(txn("bad", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "NaN"), leg("equity", "USD", "credit", "NaN")]))
            after_size = wal.stat().st_size if wal.exists() else 0
            self.assertFalse(result["ok"])
            self.assertEqual(result["error"], "invalid_amount")
            self.assertEqual(before_size, after_size)
            self.assertEqual(engine.balance_as_of("cash", "USD"), "0.00")

    def test_04_idempotency_replay_returns_original_without_double_post(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            wal = Path(tmp) / "ledger.wal"
            engine = ChronoLedgerEngine(wal)
            first = engine.post_transaction(txn("txn-1", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "11.00"), leg("equity", "USD", "credit", "11.00")], "client-a"))
            second = engine.post_transaction(txn("txn-different", "2026-07-02T11:00:00Z", [leg("cash", "USD", "debit", "99.00"), leg("equity", "USD", "credit", "99.00")], "client-a"))
            self.assertTrue(first["ok"])
            self.assertTrue(second["ok"])
            self.assertTrue(second["duplicate"])
            self.assertEqual(second["txn_id"], first["txn_id"])
            self.assertEqual(engine.balance_as_of("cash", "USD"), "11.00")
            restarted = ChronoLedgerEngine(wal)
            third = restarted.post_transaction(txn("third", "2026-07-02T12:00:00Z", [leg("cash", "USD", "debit", "7.00"), leg("equity", "USD", "credit", "7.00")], "client-a"))
            self.assertTrue(third["duplicate"])
            self.assertEqual(restarted.balance_as_of("cash", "USD"), "11.00")

    def test_05_duplicate_txn_id_without_idempotency_rejected(self) -> None:
        engine = self.make_engine()
        self.assertTrue(engine.post_transaction(txn("dup", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "1.00"), leg("equity", "USD", "credit", "1.00")]))["ok"])
        rejected = engine.post_transaction(txn("dup", "2026-07-02T10:01:00Z", [leg("cash", "USD", "debit", "2.00"), leg("equity", "USD", "credit", "2.00")]))
        self.assertFalse(rejected["ok"])
        self.assertEqual(rejected["error"], "duplicate_txn_id")
        self.assertEqual(engine.balance_as_of("cash", "USD"), "1.00")

    def test_06_as_of_boundary_inclusive_and_future_excluded(self) -> None:
        engine = self.make_engine()
        engine.post_transaction(txn("before", "2026-07-02T09:59:59Z", [leg("cash", "USD", "debit", "1.00"), leg("equity", "USD", "credit", "1.00")]))
        engine.post_transaction(txn("at", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "2.00"), leg("equity", "USD", "credit", "2.00")]))
        engine.post_transaction(txn("after", "2026-07-02T10:00:01Z", [leg("cash", "USD", "debit", "4.00"), leg("equity", "USD", "credit", "4.00")]))
        self.assertEqual(engine.balance_as_of("cash", "USD", "2026-07-02T09:59:59Z"), "1.00")
        self.assertEqual(engine.balance_as_of("cash", "USD", "2026-07-02T10:00:00Z"), "3.00")
        self.assertEqual(engine.balance_as_of("cash", "USD", "2026-07-02T10:00:00.500000Z"), "3.00")
        self.assertEqual(engine.balance_as_of("cash", "USD"), "7.00")

    def test_07_reversal_posts_inverse_at_reversal_timestamp(self) -> None:
        engine = self.make_engine()
        engine.post_transaction(txn("sale", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "50.00"), leg("revenue", "USD", "credit", "50.00")]))
        rev = engine.reverse("sale", ts="2026-07-03T00:00:00Z")
        self.assertTrue(rev["ok"])
        self.assertEqual(engine.balance_as_of("cash", "USD", "2026-07-02T23:59:59Z"), "50.00")
        self.assertEqual(engine.balance_as_of("cash", "USD", "2026-07-03T00:00:00Z"), "0.00")
        self.assertEqual(engine.balance_as_of("revenue", "USD"), "0.00")

    def test_08_double_reversal_rejected(self) -> None:
        engine = self.make_engine()
        engine.post_transaction(txn("sale", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "5.00"), leg("revenue", "USD", "credit", "5.00")]))
        self.assertTrue(engine.reverse("sale", ts="2026-07-03T00:00:00Z")["ok"])
        rejected = engine.reverse("sale", ts="2026-07-04T00:00:00Z")
        self.assertFalse(rejected["ok"])
        self.assertEqual(rejected["error"], "already_reversed")

    def test_09_rounding_only_at_reporting_boundary_bankers_rounding(self) -> None:
        engine = self.make_engine()
        engine.post_transaction(txn("round-a", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "10.005"), leg("equity", "USD", "credit", "10.005")]))
        self.assertEqual(engine.raw_balance_as_of("cash", "USD"), "10.005")
        self.assertEqual(engine.balance_as_of("cash", "USD"), "10.00")
        engine.post_transaction(txn("round-b", "2026-07-02T10:01:00Z", [leg("cash2", "USD", "debit", "10.015"), leg("equity2", "USD", "credit", "10.015")]))
        self.assertEqual(engine.balance_as_of("cash2", "USD"), "10.02")

    def test_10_wal_cold_start_replays_state_and_indexes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            wal = Path(tmp) / "ledger.wal"
            engine = ChronoLedgerEngine(wal)
            first = engine.post_transaction(txn("a", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "3.00"), leg("equity", "USD", "credit", "3.00")], "key-a"))
            engine.post_transaction(txn("b", "2026-07-02T11:00:00Z", [leg("cash", "USD", "debit", "4.00"), leg("equity", "USD", "credit", "4.00")]))
            restarted = ChronoLedgerEngine(wal)
            replay = restarted.post_transaction(txn("new-id", "2026-07-02T12:00:00Z", [leg("cash", "USD", "debit", "999.00"), leg("equity", "USD", "credit", "999.00")], "key-a"))
            self.assertEqual(restarted.balance_as_of("cash", "USD"), "7.00")
            self.assertEqual(replay["txn_id"], first["txn_id"])
            self.assertTrue(replay["duplicate"])

    def test_11_torn_final_wal_record_is_discarded(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            wal = Path(tmp) / "ledger.wal"
            engine = ChronoLedgerEngine(wal)
            engine.post_transaction(txn("good", "2026-07-02T10:00:00Z", [leg("cash", "USD", "debit", "8.00"), leg("equity", "USD", "credit", "8.00")]))
            with wal.open("ab") as f:
                f.write(b'{"kind":"transaction","txn_id":"torn","ts":"2026-07-02T10:01:00Z","legs":[')
            restarted = ChronoLedgerEngine(wal)
            self.assertEqual(restarted.balance_as_of("cash", "USD"), "8.00")
            ok = restarted.post_transaction(txn("after", "2026-07-02T10:02:00Z", [leg("cash", "USD", "debit", "2.00"), leg("equity", "USD", "credit", "2.00")]))
            self.assertTrue(ok["ok"])
            self.assertEqual(restarted.balance_as_of("cash", "USD"), "10.00")

    def test_12_public_api_imports_and_subprocess_visible_shape(self) -> None:
        repo = Path(__file__).resolve().parent
        env = os.environ.copy()
        env["PYTHONPATH"] = str(repo / "src")
        code = (
            "from chronoledger import ChronoLedgerEngine; "
            "import tempfile, pathlib; "
            "d=tempfile.TemporaryDirectory(); "
            "e=ChronoLedgerEngine(pathlib.Path(d.name)/'x.wal'); "
            "print(e.post_transaction({'txn_id':'x','ts':'2026-01-01T00:00:00Z','legs':[{'account':'a','currency':'USD','side':'debit','amount':'1.00'},{'account':'b','currency':'USD','side':'credit','amount':'1.00'}]})['ok']); "
            "print(e.balance_as_of('a','USD'))"
        )
        out = subprocess.check_output([sys.executable, "-c", code], cwd=repo, env=env, text=True)
        self.assertEqual(out.strip().splitlines(), ["True", "1.00"])


class CountingTextResult(unittest.TextTestResult):
    def addSuccess(self, test):  # type: ignore[no-untyped-def]
        super().addSuccess(test)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", help="Benchmark repo copy to verify")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    if not (repo / "src" / "chronoledger").exists():
        print(json.dumps({"ok": False, "error": f"not a chronoledger repo: {repo}"}))
        print("HIDDEN_VERIFIER_SUMMARY", json.dumps({"ok": False, "passed": 0, "total": 12, "failures": 0, "errors": 1}, sort_keys=True))
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
            timeout=120,
        )
    finally:
        target.unlink(missing_ok=True)
    print(proc.stdout, end="")
    return proc.returncode


def internal_run() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(HiddenContractTests)
    runner = unittest.TextTestRunner(verbosity=2, resultclass=CountingTextResult)
    result = runner.run(suite)
    failed_or_error = {case.id() for case, _ in result.failures + result.errors}
    total = result.testsRun if result.testsRun else 12
    passed = max(total - len(failed_or_error), 0)
    summary = {
        "ok": result.wasSuccessful(),
        "passed": passed,
        "total": total,
        "failures": len(result.failures),
        "errors": len(result.errors),
    }
    print("HIDDEN_VERIFIER_SUMMARY", json.dumps(summary, sort_keys=True))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    if "--internal-run" in sys.argv:
        raise SystemExit(internal_run())
    raise SystemExit(main())
