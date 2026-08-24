You are implementing the ChronoLedger gnarly benchmark task.

Work in this repository only. Do not remove tests. Do not bypass the verifier.

Goal:
Implement `chronoledger`, an append-only double-entry ledger engine with
multi-currency posting, idempotency, temporal balances, reversals, exact Decimal
internals, and crash-safe WAL replay.

Core API:

```python
from chronoledger import ChronoLedgerEngine, LedgerError

engine = ChronoLedgerEngine("ledger.wal")
result = engine.post_transaction(transaction)
balance = engine.balance_as_of("cash", "USD", "2026-07-02T10:00:00Z")
raw = engine.raw_balance_as_of("cash", "USD", "2026-07-02T10:00:00Z")
reversal = engine.reverse("txn-1", ts="2026-07-03T00:00:00Z")
```

Transaction input:

```python
{
  "txn_id": "txn-1",
  "ts": "2026-07-02T10:00:00Z",
  "idempotency_key": "client-key-1",
  "description": "optional",
  "legs": [
    {"account": "cash", "currency": "USD", "side": "debit", "amount": "100.00"},
    {"account": "revenue", "currency": "USD", "side": "credit", "amount": "100.00"}
  ]
}
```

Return shape for successful new posts:

```python
{"ok": True, "status": "posted", "txn_id": "txn-1", "sequence": 1, "duplicate": False}
```

Return shape for idempotent replays:

```python
{"ok": True, "status": "idempotent_replay", "txn_id": "txn-1", "sequence": 1, "duplicate": True}
```

Rejected operations must return `{"ok": False, "error": "<stable_code>", "message": "..."}`.
Raising `LedgerError` is also acceptable internally, but public methods should
prefer returning the rejected result above.

Required semantics:

1. Transactions are atomic. A rejected transaction must not change in-memory
   balances and must not append a WAL record.
2. Every transaction must have `txn_id`, `ts`, and a non-empty `legs` list.
3. Each leg must have `account`, `currency`, `side`, and `amount`.
4. `side` is exactly `debit` or `credit`.
5. `amount` must parse as a finite positive `Decimal`; zero, negative, NaN,
   infinity, booleans, blanks, and invalid strings are rejected.
6. Internal arithmetic must use `Decimal`, never float.
7. For every transaction, debits must equal credits independently per currency.
   A USD imbalance cannot be offset by an EUR imbalance.
8. Account posted balance convention: debits increase balances, credits decrease
   balances.
9. Multi-leg transactions are allowed; the balance invariant applies to the sum
   of all legs for each currency.
10. `txn_id` must be unique for new transactions.
11. If `idempotency_key` is supplied and has been seen before, the operation is a
    no-op and returns the original successful result for that key, even after a
    cold restart from WAL.
12. Idempotent replay must not append another WAL record and must not double-post
    balances.
13. If no idempotency key is supplied and `txn_id` already exists, reject with
    `duplicate_txn_id`.
14. `balance_as_of(account, currency, as_of)` returns the posted balance for that
    account/currency including transactions whose timestamp is exactly equal to
    `as_of`. Transactions with timestamps later than `as_of` are excluded.
15. If `as_of` is `None`, balance queries include all applied transactions.
16. `raw_balance_as_of` returns the exact unrounded Decimal value as a string.
17. `balance_as_of` rounds only at this reporting boundary using Decimal
    banker's rounding (`ROUND_HALF_EVEN`) to exactly two decimal places.
18. Reversal API: `reverse(txn_id, ts, reversal_id=None, idempotency_key=None)`.
19. A reversal posts the exact inverse of the referenced transaction's legs at
    the supplied reversal timestamp.
20. If `reversal_id` is omitted, use `rev:<txn_id>`.
21. Reversing a transaction that does not exist rejects with
    `missing_reversal_target`.
22. Reversing a transaction that has already been reversed rejects with
    `already_reversed`.
23. Reversals are normal append-only transactions, appear in balances at their
    own timestamps, and may use idempotency keys.
24. A reversal must never mutate or delete the original WAL record.
25. `ChronoLedgerEngine(wal_path)` must cold-start by replaying existing WAL
    records before accepting new operations.
26. The WAL is append-only JSON Lines. Each successful post/reversal appends one
    complete JSON object followed by a newline.
27. If the final WAL record is torn by a crash (invalid JSON or missing final
    newline), startup must detect and discard that final partial record.
28. A torn final record must not corrupt earlier valid state and must not prevent
    future appends.
29. Earlier malformed WAL records are corruption and may raise `LedgerError`.
30. No network dependencies or third-party packages.

Definition of done:
- `python3 -m unittest discover -s tests` passes.
- The benchmark harness's external verifier passes.
- Keep code readable and scoped.
