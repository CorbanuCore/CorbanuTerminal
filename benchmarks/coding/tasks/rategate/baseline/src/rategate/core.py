"""RateGate limiter implementation.

The benchmark starts with deliberately incomplete code.
Implement the contract in ../../BENCHMARK_TASK.md.
"""

from __future__ import annotations

from collections.abc import Iterable
from typing import Any


def token_bucket(requests: Iterable[dict[str, Any]], rate: float, capacity: float) -> dict[str, Any]:
    """Evaluate per-key token buckets.

    This stub only exists so imports work before the benchmark agent starts.
    Replace it with a correct implementation.
    """

    return {"decisions": [], "diagnostics": []}


def sliding_window(
    requests: Iterable[dict[str, Any]], limit: int, window_seconds: float
) -> dict[str, Any]:
    """Evaluate per-key sliding-window limits.

    Replace it with a correct implementation.
    """

    return {"decisions": [], "diagnostics": []}
