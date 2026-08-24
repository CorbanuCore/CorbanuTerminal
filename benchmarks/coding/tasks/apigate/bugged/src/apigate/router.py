from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Any, Callable

from .types import APIError, Request, Response

Handler = Callable[[Request], Response | dict[str, Any] | list[Any] | str | int | None]


@dataclass
class Route:
    method: str
    template: str
    handler: Handler
    auth_required: bool = True
    permission: str | None = None
    rate_limit: str | None = None
    _regex: re.Pattern[str] = field(init=False, repr=False)
    _param_names: list[str] = field(init=False, repr=False)
    _specificity: tuple[int, int, int] = field(init=False, repr=False)

    def __post_init__(self) -> None:
        self.method = self.method.upper()
        parts = [part for part in self.template.strip("/").split("/") if part]
        regex_parts: list[str] = []
        names: list[str] = []
        static_count = 0
        for part in parts:
            if part.startswith("{") and part.endswith("}"):
                name = part[1:-1]
                if not name:
                    raise ValueError("empty path parameter")
                names.append(name)
                regex_parts.append(r"(?P<" + name + r">[^/]+)")
            else:
                static_count += 1
                regex_parts.append(re.escape(part))
        pattern = "^/" + "/".join(regex_parts) + "$"
        if not parts:
            pattern = "^/$"
        self._regex = re.compile(pattern)
        self._param_names = names
        self._specificity = (static_count, -len(names), len(parts))

    def match(self, method: str, path: str) -> dict[str, str] | None:
        if method.upper() != self.method:
            return None
        match = self._regex.match(path)
        if not match:
            return None
        return {key: value for key, value in match.groupdict().items()}


class Router:
    def __init__(self) -> None:
        self.routes: list[Route] = []

    def add(
        self,
        method: str,
        template: str,
        handler: Handler,
        *,
        auth_required: bool = True,
        permission: str | None = None,
        rate_limit: str | None = None,
    ) -> Route:
        route = Route(method, template, handler, auth_required=auth_required, permission=permission, rate_limit=rate_limit)
        self.routes.append(route)
        self.routes.sort(key=lambda item: item._specificity)
        return route

    def match(self, request: Request) -> tuple[Route, dict[str, str]]:
        method_seen = False
        for route in self.routes:
            path_match = route._regex.match(request.path)
            if path_match and route.method != request.method:
                method_seen = True
                continue
            params = route.match(request.method, request.path)
            if params is not None:
                return route, params
        if method_seen:
            raise APIError(405, "method_not_allowed", "method is not allowed for this path")
        raise APIError(404, "not_found", f"no route for {request.method} {request.path}")
