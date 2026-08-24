from __future__ import annotations

import time
from dataclasses import dataclass


@dataclass(frozen=True)
class RetryPolicy:
    max_attempts: int = 1
    backoff_seconds: float = 0.0
    multiplier: float = 1.0

    @classmethod
    def from_config(cls, config: dict) -> "RetryPolicy":
        return cls(
            max_attempts=int(config.get("max_attempts", 1)),
            backoff_seconds=float(config.get("backoff_seconds", 0.0)),
            multiplier=float(config.get("multiplier", 1.0)),
        )

    def attempts(self) -> range:
        return range(1, self.max_attempts + 2)

    def should_retry(self, attempt: int) -> bool:
        return attempt <= self.max_attempts

    def delay_for(self, attempt: int) -> float:
        if attempt <= 0:
            return 0.0
        return self.backoff_seconds * (self.multiplier ** max(attempt - 1, 0))

    def sleep_before_retry(self, attempt: int) -> None:
        delay = self.delay_for(attempt)
        if delay > 0:
            time.sleep(delay)
