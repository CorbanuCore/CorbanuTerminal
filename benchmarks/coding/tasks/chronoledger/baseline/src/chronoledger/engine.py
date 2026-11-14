"""ChronoLedger append-only double-entry ledger engine.

The benchmark starts with deliberately incomplete code. Implement the contract.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any


class LedgerError(ValueError):
    """Raised for rejected ledger operations."""


class ChronoLedgerEngine:
    """Append-only double-entry ledger backed by a JSONL WAL."""

    def __init__(self, wal_path: str | Path):
        self.wal_path = Path(wal_path)

    def post_transaction(self, transaction: dict[str, Any]) -> dict[str, Any]:
        """Atomically validate, persist, and apply one transaction."""

        return {"ok": False, "error": "not_implemented"}

    def balance_as_of(self, account: str, currency: str, as_of: str | None = None) -> str:
        """Return a banker's-rounded 2dp posted balance as of timestamp."""

        return "0.00"

    def raw_balance_as_of(self, account: str, currency: str, as_of: str | None = None) -> str:
        """Return the exact unrounded Decimal balance string as of timestamp."""

        return "0"

    def reverse(self, txn_id: str, ts: str, reversal_id: str | None = None, idempotency_key: str | None = None) -> dict[str, Any]:
        """Post the exact inverse of a previously applied transaction."""

        return {"ok": False, "error": "not_implemented"}
