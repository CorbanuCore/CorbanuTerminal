from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any


class APIError(Exception):
    def __init__(self, status: int, code: str, message: str, details: dict[str, Any] | None = None):
        super().__init__(message)
        self.status = status
        self.code = code
        self.message = message
        self.details = details or {}

    def to_body(self) -> dict[str, Any]:
        body: dict[str, Any] = {"ok": False, "error": self.code, "message": self.message}
        if self.details:
            body["details"] = self.details
        return body


@dataclass
class Request:
    method: str
    path: str
    headers: dict[str, str] = field(default_factory=dict)
    body: Any = None
    query: dict[str, str] = field(default_factory=dict)
    now: datetime | None = None

    def __post_init__(self) -> None:
        self.method = self.method.upper()
        self.headers = {str(k).lower(): str(v) for k, v in self.headers.items()}
        if self.now is None:
            self.now = datetime.now(timezone.utc)
        elif self.now.tzinfo is None:
            self.now = self.now.replace(tzinfo=timezone.utc)
        else:
            self.now = self.now.astimezone(timezone.utc)

    def header(self, name: str, default: str | None = None) -> str | None:
        return self.headers.get(name.lower(), default)

    @property
    def bearer_token(self) -> str | None:
        value = self.header("authorization")
        if not value:
            return None
        prefix = "bearer "
        if not value.lower().startswith(prefix):
            return None
        token = value[len(prefix) :].strip()
        return token or None


@dataclass
class Response:
    status: int = 200
    body: Any = None
    headers: dict[str, str] = field(default_factory=dict)


def utc_iso(value: datetime) -> str:
    if value.tzinfo is None:
        value = value.replace(tzinfo=timezone.utc)
    value = value.astimezone(timezone.utc)
    return value.isoformat(timespec="seconds").replace("+00:00", "Z")
