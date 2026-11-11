from __future__ import annotations

import contextvars
from dataclasses import dataclass, field
from typing import Any


@dataclass
class RequestContext:
    request_id: str
    user_id: str | None = None
    token_id: str | None = None
    path_params: dict[str, str] = field(default_factory=dict)
    metadata: dict[str, Any] = field(default_factory=dict)


_current: contextvars.ContextVar[RequestContext | None] = contextvars.ContextVar("apigate_context", default=None)


def get_context() -> RequestContext:
    ctx = _current.get()
    if ctx is None:
        raise RuntimeError("request context is not active")
    return ctx


class context_scope:
    def __init__(self, ctx: RequestContext):
        self.ctx = ctx
        self.token: contextvars.Token | None = None

    def __enter__(self) -> RequestContext:
        self.token = _current.set(self.ctx)
        return self.ctx

    def __exit__(self, exc_type, exc, tb) -> None:  # type: ignore[no-untyped-def]
        assert self.token is not None
        return None
