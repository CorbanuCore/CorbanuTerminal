from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal


@dataclass(frozen=True)
class PostingLeg:
    account: str
    currency: str
    side: str
    amount: Decimal


@dataclass(frozen=True)
class PostedTransaction:
    txn_id: str
    ts: str
    sequence: int
    idempotency_key: str | None
    legs: tuple[PostingLeg, ...]
    reverses: str | None = None
