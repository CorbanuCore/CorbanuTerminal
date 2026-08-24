from __future__ import annotations

from pathlib import Path
from typing import Any


class WriteAheadLog:
    """Tiny append-only JSONL WAL helper.

    The engine owns the serialization contract. This stub exists to give agents
    a natural place for replay/write logic.
    """

    def __init__(self, path: str | Path):
        self.path = Path(path)

    def append(self, record: dict[str, Any]) -> None:
        raise NotImplementedError

    def read_records(self) -> list[dict[str, Any]]:
        raise NotImplementedError
