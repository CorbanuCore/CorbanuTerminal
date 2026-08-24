from __future__ import annotations

import unittest

from apigate.permissions import PermissionCache, PermissionStore
from apigate.types import APIError


class PermissionTests(unittest.TestCase):
    def test_store_returns_copy_of_permission_set(self) -> None:
        store = PermissionStore()
        store.grant("user", "widgets:read")

        permissions = store.permissions_for("user")
        permissions.add("widgets:write")

        self.assertEqual(store.permissions_for("user"), {"widgets:read"})

    def test_cache_loads_granted_permission(self) -> None:
        store = PermissionStore()
        cache = PermissionCache(store)

        cache.grant("user", "widgets:read")

        self.assertTrue(cache.has("user", "widgets:read"))

    def test_revoke_invalidates_cached_permission(self) -> None:
        store = PermissionStore()
        cache = PermissionCache(store)
        cache.grant("user", "widgets:read")
        self.assertTrue(cache.has("user", "widgets:read"))

        cache.revoke("user", "widgets:read")

        self.assertFalse(cache.has("user", "widgets:read"))

    def test_require_rejects_missing_user(self) -> None:
        cache = PermissionCache(PermissionStore())

        with self.assertRaises(APIError) as raised:
            cache.require(None, "widgets:read")

        self.assertEqual(raised.exception.status, 401)
        self.assertEqual(raised.exception.code, "missing_user")

    def test_require_rejects_missing_permission_without_leaking_other_user(self) -> None:
        cache = PermissionCache(PermissionStore())
        cache.grant("admin", "widgets:read")

        with self.assertRaises(APIError) as raised:
            cache.require("user", "widgets:read")

        self.assertEqual(raised.exception.status, 403)
        self.assertEqual(raised.exception.code, "forbidden")


if __name__ == "__main__":
    unittest.main()
