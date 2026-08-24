from __future__ import annotations

import unittest

from apigate.router import Router
from apigate.types import APIError, Request


class RouterTests(unittest.TestCase):
    def test_dynamic_path_parameters_are_returned(self) -> None:
        router = Router()
        router.add("GET", "/teams/{team_id}/widgets/{widget_id}", lambda request: {})

        route, params = router.match(Request("GET", "/teams/red/widgets/w1"))

        self.assertEqual(route.template, "/teams/{team_id}/widgets/{widget_id}")
        self.assertEqual(params, {"team_id": "red", "widget_id": "w1"})

    def test_static_route_wins_even_if_registered_after_dynamic_route(self) -> None:
        router = Router()
        router.add("GET", "/widgets/{widget_id}", lambda request: {"kind": "dynamic"})
        router.add("GET", "/widgets/me", lambda request: {"kind": "static"})

        route, params = router.match(Request("GET", "/widgets/me"))

        self.assertEqual(route.template, "/widgets/me")
        self.assertEqual(params, {})

    def test_method_is_normalized_before_matching(self) -> None:
        router = Router()
        router.add("post", "/widgets/{widget_id}", lambda request: {})

        route, params = router.match(Request("post", "/widgets/abc"))

        self.assertEqual(route.method, "POST")
        self.assertEqual(params["widget_id"], "abc")

    def test_method_not_allowed_is_distinct_from_not_found(self) -> None:
        router = Router()
        router.add("GET", "/widgets/{widget_id}", lambda request: {})

        with self.assertRaises(APIError) as raised:
            router.match(Request("POST", "/widgets/abc"))

        self.assertEqual(raised.exception.status, 405)
        self.assertEqual(raised.exception.code, "method_not_allowed")

    def test_not_found_reports_requested_path(self) -> None:
        router = Router()
        router.add("GET", "/widgets/{widget_id}", lambda request: {})

        with self.assertRaises(APIError) as raised:
            router.match(Request("GET", "/missing/abc"))

        self.assertEqual(raised.exception.status, 404)
        self.assertIn("GET /missing/abc", raised.exception.message)


if __name__ == "__main__":
    unittest.main()
