from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from chronoledger import ChronoLedgerEngine


class VisibleContractTests(unittest.TestCase):
    def test_post_transaction_and_query_balance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            engine = ChronoLedgerEngine(Path(tmp) / "ledger.wal")
            result = engine.post_transaction(
                {
                    "txn_id": "txn-1",
                    "ts": "2026-07-02T10:00:00Z",
                    "idempotency_key": "client-1",
                    "legs": [
                        {"account": "cash", "currency": "USD", "side": "debit", "amount": "125.00"},
                        {"account": "revenue", "currency": "USD", "side": "credit", "amount": "125.00"},
                    ],
                }
            )

            self.assertTrue(result["ok"])
            self.assertEqual(engine.balance_as_of("cash", "USD", "2026-07-02T10:00:00Z"), "125.00")
            self.assertEqual(engine.balance_as_of("revenue", "USD", "2026-07-02T10:00:00Z"), "-125.00")


if __name__ == "__main__":
    unittest.main()
