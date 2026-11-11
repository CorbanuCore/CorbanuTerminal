from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any


class CheckpointError(ValueError):
    pass


class JsonCheckpointStore:
    def __init__(self, path: str | Path | None):
        self.path = Path(path) if path else None

    def load(self) -> dict[str, dict[str, Any]]:
        if self.path is None or not self.path.exists():
            return {}
        try:
            data = json.loads(self.path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise CheckpointError(f"invalid checkpoint JSON: {self.path}") from exc
        if not isinstance(data, dict):
            raise CheckpointError("checkpoint root must be an object")
        completed = data.get("completed", {})
        if not isinstance(completed, dict):
            raise CheckpointError("checkpoint completed must be an object")
        return completed

    def save_task(self, task_name: str, output: Any, attempts: int, order_index: int) -> None:
        if self.path is None:
            return
        data = {"completed": self.load()}
        data["completed"][task_name] = {"output": output, "attempts": attempts, "order_index": order_index}
        self._atomic_write(data)

    def clear(self) -> None:
        if self.path is not None and self.path.exists():
            self.path.unlink()

    def _atomic_write(self, data: dict[str, Any]) -> None:
        assert self.path is not None
        self.path.parent.mkdir(parents=True, exist_ok=True)
        tmp = self.path.with_suffix(self.path.suffix + ".tmp")
        tmp.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        os.replace(tmp, self.path)
