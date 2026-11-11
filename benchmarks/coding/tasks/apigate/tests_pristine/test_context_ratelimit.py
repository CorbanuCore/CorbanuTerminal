from __future__ import annotations

import unittest
from datetime import datetime, timedelta, timezone

from apigate.app import create_demo_app
from apigate.context import RequestContext, context_scope, get_context
from apigate.ratelimit import RateLimiter
from apigate.types import APIError, Request


def issue_token(app, user_id: str, now: datetime) -> str:
    response = app.handle(Request("POST", "/auth/issue", body={"user_id": user_id}, now=now))
    return response["json"]["data"]["access_token"]


class ContextAndRateLimitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.now = datetime(2026, 7, 2, tzinfo=timezone.utc)

    def test_context_scope_resets_after_normal_exit(self) -> None:
        with context_scope(RequestContext(request_id="req-test", user_id="user")):
            self.assertEqual(get_context().user_id, "user")

        with self.assertRaises(RuntimeError):
            get_context()

    def test_context_scope_resets_after_exception(self) -> None:
        try:
            with context_scope(RequestContext(request_id="req-test", user_id="user")):
                raise ValueError("boom")
        except ValueError:
            pass

        with self.assertRaises(RuntimeError):
            get_context()

    def test_handle_leaves_no_active_context_between_requests(self) -> None:
        app = create_demo_app()
        token = issue_token(app, "user", self.now)

        first = app.handle(Request("GET", "/widgets/me", headers={"authorization": f"Bearer {token}"}, now=self.now))
        self.assertEqual(first["status"], 200)

        with self.assertRaises(RuntimeError):
            get_context()

    def test_rate_limiter_blocks_after_policy_limit_and_recovers_after_window(self) -> None:
        limiter = RateLimiter()
        limiter.set_policy("writes", limit=2, window_seconds=60)

        limiter.check("writes", "user", now=self.now)
        limiter.check("writes", "user", now=self.now + timedelta(seconds=1))
        with self.assertRaises(APIError) as raised:
            limiter.check("writes", "user", now=self.now + timedelta(seconds=2))
        limiter.check("writes", "user", now=self.now + timedelta(seconds=61))

        self.assertEqual(raised.exception.status, 429)
        self.assertEqual(raised.exception.code, "rate_limited")

    def test_app_write_limit_is_keyed_to_user_across_tokens(self) -> None:
        app = create_demo_app()
        first_token = issue_token(app, "admin", self.now)
        second_token = issue_token(app, "admin", self.now + timedelta(seconds=1))

        for token, offset in ((first_token, 2), (first_token, 3)):
            response = app.handle(
                Request(
                    "POST",
                    "/widgets/w1",
                    headers={"authorization": f"Bearer {token}"},
                    body={"name": "x"},
                    now=self.now + timedelta(seconds=offset),
                )
            )
            self.assertEqual(response["status"], 200)

        blocked = app.handle(
            Request(
                "POST",
                "/widgets/w1",
                headers={"authorization": f"Bearer {second_token}"},
                body={"name": "x"},
                now=self.now + timedelta(seconds=4),
            )
        )
        self.assertEqual(blocked["status"], 429)
        self.assertEqual(blocked["json"]["error"], "rate_limited")


if __name__ == "__main__":
    unittest.main()
