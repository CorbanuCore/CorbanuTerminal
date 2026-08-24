from __future__ import annotations

import json
import unittest
from datetime import datetime, timezone

from apigate.app import APIGate, create_demo_app
from apigate.serialization import serialize_error, serialize_response
from apigate.types import APIError, Request, Response


class SerializationAndAppTests(unittest.TestCase):
    def test_serialize_response_normalizes_tuples_and_sorts_json_keys(self) -> None:
        response = serialize_response(Response(status=201, body={"b": (1, 2), "a": True}))

        self.assertEqual(response["status"], 201)
        self.assertEqual(response["json"], {"a": True, "b": [1, 2]})
        self.assertEqual(response["body"], '{"a":true,"b":[1,2]}')

    def test_serializer_rejects_unsupported_datetime_objects(self) -> None:
        with self.assertRaises(TypeError):
            serialize_response(Response(body={"expires_at": datetime(2026, 7, 2, tzinfo=timezone.utc)}))

    def test_serialize_error_uses_stable_error_shape(self) -> None:
        response = serialize_error(APIError(403, "forbidden", "missing permission", {"permission": "x"}))

        self.assertEqual(response["status"], 403)
        self.assertEqual(response["json"]["ok"], False)
        self.assertEqual(response["json"]["error"], "forbidden")
        self.assertEqual(response["json"]["details"], {"permission": "x"})

    def test_app_maps_router_not_found_to_json_error(self) -> None:
        response = create_demo_app().handle(Request("GET", "/missing"))

        self.assertEqual(response["status"], 404)
        self.assertEqual(response["json"]["error"], "not_found")

    def test_app_serializes_custom_response_from_handler(self) -> None:
        app = APIGate()

        @app.route("GET", "/health", auth_required=False)
        def health(request: Request) -> Response:
            return Response(status=202, body={"ok": True}, headers={"x-test": "yes"})

        response = app.handle(Request("GET", "/health"))

        self.assertEqual(response["status"], 202)
        self.assertEqual(json.loads(response["body"]), {"ok": True})
        self.assertEqual(response["headers"]["x-test"], "yes")
        self.assertEqual(response["headers"]["content-type"], "application/json")


if __name__ == "__main__":
    unittest.main()
