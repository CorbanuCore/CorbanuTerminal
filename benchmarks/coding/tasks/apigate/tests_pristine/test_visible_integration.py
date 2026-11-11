from __future__ import annotations

import json
import unittest
from datetime import datetime, timedelta, timezone

from apigate.app import create_demo_app
from apigate.types import Request


class VisibleRefreshIntegrationTests(unittest.TestCase):
    def test_refresh_endpoint_returns_json_ready_token_payload(self) -> None:
        app = create_demo_app()
        now = datetime(2026, 7, 2, tzinfo=timezone.utc)

        issued = app.handle(Request("POST", "/auth/issue", body={"user_id": "user"}, now=now))
        refresh_token = issued["json"]["data"]["refresh_token"]

        refreshed = app.handle(
            Request(
                "POST",
                "/auth/refresh",
                body={"refresh_token": refresh_token},
                now=now + timedelta(seconds=10),
            )
        )

        self.assertEqual(refreshed["status"], 200)
        self.assertEqual(json.loads(refreshed["body"]), refreshed["json"])
        data = refreshed["json"]["data"]
        self.assertTrue(data["access_token"].startswith("atk_"))
        self.assertTrue(data["refresh_token"].startswith("rtk_"))
        self.assertTrue(data["expires_at"].endswith("Z"))


if __name__ == "__main__":
    unittest.main()
