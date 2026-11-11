from __future__ import annotations

import secrets
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Any

from .types import APIError, utc_iso


class AuthError(APIError):
    pass


@dataclass(frozen=True)
class TokenRecord:
    token: str
    refresh_token: str
    user_id: str
    scopes: tuple[str, ...]
    expires_at: datetime
    refresh_expires_at: datetime
    token_id: str

    def to_public(self) -> dict[str, Any]:
        return {
            "access_token": self.token,
            "refresh_token": self.refresh_token,
            "user_id": self.user_id,
            "scopes": list(self.scopes),
            "expires_at": utc_iso(self.expires_at),
            "token_id": self.token_id,
        }


class TokenStore:
    def __init__(self, ttl_seconds: int = 300, refresh_ttl_seconds: int = 3600):
        self.ttl = timedelta(seconds=ttl_seconds)
        self.refresh_ttl = timedelta(seconds=refresh_ttl_seconds)
        self._tokens: dict[str, TokenRecord] = {}
        self._refresh: dict[str, TokenRecord] = {}
        self._counter = 0

    def issue(self, user_id: str, scopes: list[str] | tuple[str, ...] = (), now: datetime | None = None) -> TokenRecord:
        now = self._normalize_now(now)
        self._counter += 1
        record = TokenRecord(
            token="atk_" + secrets.token_hex(12),
            refresh_token="rtk_" + secrets.token_hex(12),
            user_id=user_id,
            scopes=tuple(scopes),
            expires_at=now + self.ttl,
            refresh_expires_at=now + self.refresh_ttl,
            token_id=f"tok-{self._counter}",
        )
        self._tokens[record.token] = record
        self._refresh[record.refresh_token] = record
        return record

    def authenticate(self, token: str | None, now: datetime | None = None) -> TokenRecord:
        if not token:
            raise AuthError(401, "missing_token", "bearer token is required")
        record = self._tokens.get(token)
        if record is None:
            raise AuthError(401, "invalid_token", "token is invalid")
        now = self._normalize_now(now)
        if record.expires_at <= now:
            raise AuthError(401, "expired_token", "token is expired")
        return record

    def refresh(self, refresh_token: str | None, now: datetime | None = None) -> TokenRecord:
        if not refresh_token:
            raise AuthError(401, "missing_refresh_token", "refresh token is required")
        old = self._refresh.get(refresh_token)
        if old is None:
            raise AuthError(401, "invalid_refresh_token", "refresh token is invalid")
        now = self._normalize_now(now)
        if old.refresh_expires_at <= now:
            raise AuthError(401, "expired_refresh_token", "refresh token is expired")
        self._tokens.pop(old.token, None)
        self._refresh.pop(old.refresh_token, None)
        return self.issue(old.user_id, list(old.scopes), now=now)

    def revoke_user(self, user_id: str) -> int:
        removed = 0
        for token, record in list(self._tokens.items()):
            if record.user_id == user_id:
                removed += 1
                self._tokens.pop(token, None)
                self._refresh.pop(record.refresh_token, None)
        return removed

    @staticmethod
    def _normalize_now(now: datetime | None) -> datetime:
        if now is None:
            return datetime.now(timezone.utc)
        if now.tzinfo is None:
            return now.replace(tzinfo=timezone.utc)
        return now.astimezone(timezone.utc)
