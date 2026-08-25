from __future__ import annotations

import unittest
from datetime import datetime, timedelta, timezone

from apigate.app import create_demo_app
from apigate.context import get_context
from apigate.types import Request


def issue_token(app, user_id: str, now: datetime) -> tuple[str, str]:
    response = app.handle(Request("POST", "/auth/issue", body={"user_id": user_id}, now=now))
    data = response["json"]["data"]
    return data["access_token"], data["refresh_token"]


class BugProbeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.now = datetime(2026, 7, 2, tzinfo=timezone.utc)

    def test_probe_bug1_refresh_returns_json_ready_body(self) -> None:
        app = create_demo_app()
        _, refresh_token = issue_token(app, "user", self.now)

        response = app.handle(
            Request(
                "POST",
                "/auth/refresh",
                body={"refresh_token": refresh_token},
                now=self.now + timedelta(seconds=1),
            )
        )

        self.assertEqual(response["status"], 200)
        self.assertIn("access_token", response["json"]["data"])
        self.assertTrue(response["json"]["data"]["expires_at"].endswith("Z"))

    def test_probe_bug2_permission_revoke_invalidates_cache_before_next_check(self) -> None:
        app = create_demo_app()
        token, _ = issue_token(app, "user", self.now)

        before = app.handle(
            Request("GET", "/widgets/me", headers={"authorization": f"Bearer {token}"}, now=self.now)
        )
        self.assertEqual(before["status"], 200)

        app.handle(Request("POST", "/admin/revoke", body={"user_id": "user", "permission": "widgets:read"}, now=self.now))
        after = app.handle(
            Request("GET", "/widgets/me", headers={"authorization": f"Bearer {token}"}, now=self.now)
        )

        self.assertEqual(after["status"], 403)
        self.assertEqual(after["json"]["error"], "forbidden")

    def test_probe_bug3_router_static_route_precedes_dynamic_param(self) -> None:
        app = create_demo_app()
        token, _ = issue_token(app, "user", self.now)

        response = app.handle(
            Request("GET", "/widgets/me", headers={"authorization": f"Bearer {token}"}, now=self.now)
        )

        self.assertEqual(response["status"], 200)
        data = response["json"]["data"]
        self.assertEqual(data["owner"], "user")
        self.assertEqual(data["widgets"], ["alpha", "beta"])
        self.assertNotIn("id", data)

    def test_probe_bug4_context_resets_between_requests(self) -> None:
        app = create_demo_app()
        token, _ = issue_token(app, "user", self.now)

        response = app.handle(
            Request("GET", "/widgets/me", headers={"authorization": f"Bearer {token}"}, now=self.now)
        )
        self.assertEqual(response["status"], 200)

        with self.assertRaises(RuntimeError):
            get_context()

    def test_probe_bug5_rate_limit_key_uses_authenticated_user_not_token(self) -> None:
        app = create_demo_app()
        first_token, _ = issue_token(app, "admin", self.now)
        second_token, _ = issue_token(app, "admin", self.now + timedelta(seconds=1))

        first = app.handle(
            Request(
                "POST",
                "/widgets/w1",
                headers={"authorization": f"Bearer {first_token}"},
                body={"name": "a"},
                now=self.now + timedelta(seconds=2),
            )
        )
        second = app.handle(
            Request(
                "POST",
                "/widgets/w1",
                headers={"authorization": f"Bearer {first_token}"},
                body={"name": "b"},
                now=self.now + timedelta(seconds=3),
            )
        )
        third = app.handle(
            Request(
                "POST",
                "/widgets/w1",
                headers={"authorization": f"Bearer {second_token}"},
                body={"name": "c"},
                now=self.now + timedelta(seconds=4),
            )
        )

        self.assertEqual(first["status"], 200)
        self.assertEqual(second["status"], 200)
        self.assertEqual(third["status"], 429)
        self.assertEqual(third["json"]["error"], "rate_limited")


if __name__ == "__main__":
    unittest.main()
