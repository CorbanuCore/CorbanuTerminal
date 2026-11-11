from __future__ import annotations

import unittest
from datetime import datetime, timedelta, timezone

from apigate.auth import AuthError, TokenStore


class AuthTests(unittest.TestCase):
    def setUp(self) -> None:
        self.now = datetime(2026, 7, 2, tzinfo=timezone.utc)

    def test_issued_record_public_shape_is_json_ready(self) -> None:
        store = TokenStore()
        public = store.issue("user", ["read"], now=self.now).to_public()

        self.assertEqual(public["user_id"], "user")
        self.assertEqual(public["scopes"], ["read"])
        self.assertTrue(public["access_token"].startswith("atk_"))
        self.assertTrue(public["refresh_token"].startswith("rtk_"))
        self.assertTrue(public["expires_at"].endswith("Z"))

    def test_authenticate_requires_token(self) -> None:
        store = TokenStore()

        with self.assertRaises(AuthError) as raised:
            store.authenticate(None, now=self.now)

        self.assertEqual(raised.exception.code, "missing_token")

    def test_authenticate_rejects_expired_access_token(self) -> None:
        store = TokenStore(ttl_seconds=5)
        record = store.issue("user", now=self.now)

        with self.assertRaises(AuthError) as raised:
            store.authenticate(record.token, now=self.now + timedelta(seconds=5))

        self.assertEqual(raised.exception.code, "expired_token")

    def test_refresh_rotates_access_and_refresh_tokens(self) -> None:
        store = TokenStore()
        record = store.issue("user", ["write"], now=self.now)

        refreshed = store.refresh(record.refresh_token, now=self.now + timedelta(seconds=1))

        self.assertNotEqual(refreshed.token, record.token)
        self.assertNotEqual(refreshed.refresh_token, record.refresh_token)
        self.assertEqual(refreshed.user_id, "user")
        self.assertEqual(refreshed.scopes, ("write",))

    def test_refresh_invalidates_old_access_token_and_refresh_token(self) -> None:
        store = TokenStore()
        record = store.issue("user", now=self.now)
        store.refresh(record.refresh_token, now=self.now + timedelta(seconds=1))

        with self.assertRaises(AuthError) as access_error:
            store.authenticate(record.token, now=self.now + timedelta(seconds=2))
        with self.assertRaises(AuthError) as refresh_error:
            store.refresh(record.refresh_token, now=self.now + timedelta(seconds=2))

        self.assertEqual(access_error.exception.code, "invalid_token")
        self.assertEqual(refresh_error.exception.code, "invalid_refresh_token")

    def test_refresh_can_happen_after_access_token_expiry(self) -> None:
        store = TokenStore(ttl_seconds=5, refresh_ttl_seconds=30)
        record = store.issue("user", now=self.now)

        refreshed = store.refresh(record.refresh_token, now=self.now + timedelta(seconds=6))

        self.assertEqual(refreshed.user_id, "user")

    def test_expired_refresh_token_is_rejected(self) -> None:
        store = TokenStore(ttl_seconds=5, refresh_ttl_seconds=10)
        record = store.issue("user", now=self.now)

        with self.assertRaises(AuthError) as raised:
            store.refresh(record.refresh_token, now=self.now + timedelta(seconds=10))

        self.assertEqual(raised.exception.code, "expired_refresh_token")

    def test_revoke_user_removes_only_matching_tokens(self) -> None:
        store = TokenStore()
        user_record = store.issue("user", now=self.now)
        admin_record = store.issue("admin", now=self.now)

        removed = store.revoke_user("user")

        self.assertEqual(removed, 1)
        with self.assertRaises(AuthError):
            store.authenticate(user_record.token, now=self.now)
        self.assertEqual(store.authenticate(admin_record.token, now=self.now).user_id, "admin")


if __name__ == "__main__":
    unittest.main()
