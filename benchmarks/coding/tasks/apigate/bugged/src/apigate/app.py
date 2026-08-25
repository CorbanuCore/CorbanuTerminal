from __future__ import annotations

import itertools
from typing import Any

from .auth import AuthError, TokenStore
from .context import RequestContext, context_scope, get_context
from .permissions import PermissionCache, PermissionStore
from .ratelimit import RateLimiter
from .router import Router
from .serialization import serialize_error, serialize_response
from .types import APIError, Request, Response


class APIGate:
    def __init__(self) -> None:
        self.router = Router()
        self.tokens = TokenStore()
        self.permission_store = PermissionStore()
        self.permissions = PermissionCache(self.permission_store)
        self.ratelimiter = RateLimiter()
        self._request_ids = itertools.count(1)

    def route(self, method: str, template: str, **options):  # type: ignore[no-untyped-def]
        def decorator(fn):
            self.router.add(method, template, fn, **options)
            return fn

        return decorator

    def handle(self, request: Request) -> dict[str, Any]:
        try:
            route, params = self.router.match(request)
            ctx = RequestContext(request_id=f"req-{next(self._request_ids)}", path_params=params)
            with context_scope(ctx):
                if route.auth_required:
                    record = self.tokens.authenticate(request.bearer_token, now=request.now)
                    ctx.user_id = record.user_id
                    ctx.token_id = record.token_id
                rate_key = ctx.token_id or ctx.user_id or request.header("x-client-id") or "anonymous"
                self.ratelimiter.check(route.rate_limit, rate_key, now=request.now)
                self.permissions.require(ctx.user_id, route.permission)
                result = route.handler(request)
                if isinstance(result, Response):
                    return serialize_response(result)
                return serialize_response(Response(status=200, body={"ok": True, "data": result}))
        except APIError as exc:
            return serialize_error(exc)

    def issue_token_response(self, user_id: str, scopes: list[str] | None = None, now=None) -> dict[str, Any]:  # type: ignore[no-untyped-def]
        return self.tokens.issue(user_id, scopes or [], now=now).to_public()


def create_demo_app() -> APIGate:
    app = APIGate()
    app.ratelimiter.set_policy("user-write", limit=2, window_seconds=60)
    app.permissions.grant("admin", "widgets:read")
    app.permissions.grant("admin", "widgets:write")
    app.permissions.grant("user", "widgets:read")

    @app.route("POST", "/auth/issue", auth_required=False)
    def issue(request: Request) -> dict[str, Any]:
        body = request.body or {}
        return app.issue_token_response(str(body.get("user_id", "user")), list(body.get("scopes", [])), now=request.now)

    @app.route("POST", "/auth/refresh", auth_required=False)
    def refresh(request: Request) -> dict[str, Any]:
        body = request.body or {}
        record = app.tokens.refresh(body.get("refresh_token"), now=request.now)
        return {
            "access_token": record.token,
            "refresh_token": record.refresh_token,
            "user_id": record.user_id,
            "scopes": record.scopes,
            "expires_at": record.expires_at,
            "token_id": record.token_id,
        }

    @app.route("GET", "/widgets/me", permission="widgets:read")
    def my_widgets(request: Request) -> dict[str, Any]:
        ctx = get_context()
        return {"owner": ctx.user_id, "widgets": ["alpha", "beta"]}

    @app.route("GET", "/widgets/{widget_id}", permission="widgets:read")
    def get_widget(request: Request) -> dict[str, Any]:
        ctx = get_context()
        return {"id": ctx.path_params["widget_id"], "owner": ctx.user_id}

    @app.route("POST", "/widgets/{widget_id}", permission="widgets:write", rate_limit="user-write")
    def update_widget(request: Request) -> dict[str, Any]:
        ctx = get_context()
        return {"id": ctx.path_params["widget_id"], "updated_by": ctx.user_id, "body": request.body or {}}

    @app.route("POST", "/admin/grant", auth_required=False)
    def grant(request: Request) -> dict[str, Any]:
        body = request.body or {}
        app.permissions.grant(str(body["user_id"]), str(body["permission"]))
        return {"granted": True}

    @app.route("POST", "/admin/revoke", auth_required=False)
    def revoke(request: Request) -> dict[str, Any]:
        body = request.body or {}
        app.permissions.revoke(str(body["user_id"]), str(body["permission"]))
        return {"revoked": True}

    return app
