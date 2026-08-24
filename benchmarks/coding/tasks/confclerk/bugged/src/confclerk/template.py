from __future__ import annotations

import re
from collections.abc import Mapping
from typing import Any

from .errors import TemplateRenderError


TOKEN = re.compile(r"{{\s*([^{}]+?)\s*}}")


def dotted_get(data: Mapping[str, Any], path: str, default: Any = None) -> Any:
    cur: Any = data
    for part in path.split("."):
        if isinstance(cur, Mapping) and part in cur:
            cur = cur[part]
        else:
            return default
    return cur


class TemplateRenderer:
    def __init__(self, partials: Mapping[str, str] | None = None):
        self.partials = dict(partials or {})

    def render(self, template: str, context: Mapping[str, Any]) -> str:
        expanded = self._expand_includes(template, context)
        return TOKEN.sub(lambda match: self._resolve_expr(match.group(1), context), expanded)

    def _expand_includes(self, template: str, context: Mapping[str, Any]) -> str:
        def repl(match: re.Match[str]) -> str:
            name = match.group(1).strip()
            if name not in self.partials:
                raise TemplateRenderError(f"missing partial: {name}")
            return self.render(self.partials[name], context)
        return re.sub(r"{%\s*include\s+([a-zA-Z0-9_.-]+)\s*%}", repl, template)

    def _resolve_expr(self, expr: str, context: Mapping[str, Any]) -> str:
        parts = [part.strip() for part in expr.split("|")]
        value = dotted_get(context, parts[0], None)
        if value is None:
            raise TemplateRenderError(f"missing value: {parts[0]}")
        for part in parts[1:]:
            if part == "upper":
                value = str(value).upper()
            elif part == "lower":
                value = str(value).lower()
            elif part.startswith("default:") and (value is None or value == ""):
                value = part.split(":", 1)[1]
            else:
                raise TemplateRenderError(f"unknown filter: {part}")
        return str(value)


def render_config(config: Mapping[str, Any], templates: Mapping[str, str]) -> dict[str, str]:
    renderer = TemplateRenderer(templates)
    out: dict[str, str] = {}
    for name, template in templates.items():
        out[name] = renderer.render(template, config)
    return out
