from __future__ import annotations

from collections import defaultdict

from .types import APIError


class PermissionStore:
    def __init__(self) -> None:
        self._permissions: dict[str, set[str]] = defaultdict(set)
        self.version = 0

    def grant(self, user_id: str, permission: str) -> None:
        self._permissions[user_id].add(permission)
        self.version += 1

    def revoke(self, user_id: str, permission: str) -> None:
        self._permissions[user_id].discard(permission)

    def permissions_for(self, user_id: str) -> set[str]:
        return set(self._permissions.get(user_id, set()))


class PermissionCache:
    def __init__(self, store: PermissionStore):
        self.store = store
        self._cache: dict[str, tuple[int, set[str]]] = {}

    def has(self, user_id: str, permission: str) -> bool:
        version, permissions = self._cache.get(user_id, (-1, set()))
        if version != self.store.version:
            permissions = self.store.permissions_for(user_id)
            self._cache[user_id] = (self.store.version, permissions)
        return permission in permissions

    def require(self, user_id: str | None, permission: str | None) -> None:
        if permission is None:
            return
        if user_id is None:
            raise APIError(401, "missing_user", "authenticated user is required")
        if not self.has(user_id, permission):
            raise APIError(403, "forbidden", f"missing permission: {permission}")

    def invalidate(self, user_id: str | None = None) -> None:
        if user_id is None:
            self._cache.clear()
        else:
            self._cache.pop(user_id, None)

    def grant(self, user_id: str, permission: str) -> None:
        self.store.grant(user_id, permission)
        self.invalidate(user_id)

    def revoke(self, user_id: str, permission: str) -> None:
        self.store.revoke(user_id, permission)
