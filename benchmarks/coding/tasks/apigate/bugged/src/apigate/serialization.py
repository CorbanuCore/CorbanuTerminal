from __future__ import annotations

import json
from typing import Any

from .types import Response


def normalize_body(value: Any) -> Any:
    if value is None:
        return None
    if isinstance(value, (str, int, float, bool)):
        return value
    if isinstance(value, list):
        return [normalize_body(item) for item in value]
    if isinstance(value, tuple):
        return [normalize_body(item) for item in value]
    if isinstance(value, dict):
        return {str(key): normalize_body(child) for key, child in value.items()}
    raise TypeError(f"cannot serialize {type(value).__name__}")


def serialize_response(response: Response | dict[str, Any] | list[Any] | str | int | None) -> dict[str, Any]:
    if isinstance(response, Response):
        status = response.status
        headers = dict(response.headers)
        body = response.body
    else:
        status = 200
        headers = {}
        body = response
    normalized = normalize_body(body)
    wire = "" if normalized is None else json.dumps(normalized, sort_keys=True, separators=(",", ":"))
    headers.setdefault("content-type", "application/json")
    return {"status": status, "headers": headers, "body": wire, "json": normalized}


def serialize_error(error: APIError) -> dict[str, Any]:
    return serialize_response(Response(status=error.status, body=error.to_body()))
