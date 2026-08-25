from __future__ import annotations


class ConfClerkError(Exception):
    """Base error for configuration loading, merging, and rendering."""


class ConfigLoadError(ConfClerkError):
    pass


class MergeError(ConfClerkError):
    pass


class TemplateRenderError(ConfClerkError):
    pass


class ValidationError(ConfClerkError):
    pass
