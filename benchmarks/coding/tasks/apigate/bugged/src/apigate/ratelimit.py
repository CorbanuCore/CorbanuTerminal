from __future__ import annotations

from collections import defaultdict, deque
from datetime import datetime, timezone

from .types import APIError


class RateLimiter:
    def __init__(self, default_limit: int = 100, window_seconds: int = 60):
        self.default_limit = default_limit
        self.window_seconds = window_seconds
        self._events: dict[tuple[str, str], deque[float]] = defaultdict(deque)
        self.policies: dict[str, tuple[int, int]] = {}

    def set_policy(self, name: str, limit: int, window_seconds: int) -> None:
        self.policies[name] = (limit, window_seconds)

    def check(self, policy: str | None, key: str, now: datetime | None = None) -> None:
        if policy is None:
            return
        limit, window = self.policies.get(policy, (self.default_limit, self.window_seconds))
        ts = self._timestamp(now)
        bucket = self._events[(policy, key)]
        boundary = ts - window
        while bucket and bucket[0] <= boundary:
            bucket.popleft()
        if len(bucket) >= limit:
            raise APIError(429, "rate_limited", "rate limit exceeded", {"limit": limit, "window_seconds": window})
        bucket.append(ts)

    @staticmethod
    def _timestamp(now: datetime | None) -> float:
        if now is None:
            now = datetime.now(timezone.utc)
        if now.tzinfo is None:
            now = now.replace(tzinfo=timezone.utc)
        return now.timestamp()
