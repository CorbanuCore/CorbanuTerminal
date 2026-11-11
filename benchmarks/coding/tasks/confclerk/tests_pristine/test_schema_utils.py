from __future__ import annotations

import unittest

from confclerk.loader import dotted_set
from confclerk.schema import FieldRule, Schema, build_rule_0, flatten, unflatten
from confclerk.template import dotted_get


class SchemaUtilsTests(unittest.TestCase):
    def test_schema_validation(self) -> None:
        schema = Schema([FieldRule("app.name", str), FieldRule("app.port", int, required=False, default=80)])
        self.assertEqual(schema.validate({"app": {"name": "x"}}), {"app.name": "x", "app.port": 80})

    def test_dotted_get_set(self) -> None:
        data = {}
        dotted_set(data, "a.b.c", 3)
        self.assertEqual(dotted_get(data, "a.b.c"), 3)

    def test_flatten_unflatten(self) -> None:
        self.assertEqual(unflatten(flatten({"a": {"b": 1}})), {"a": {"b": 1}})

    def test_generated_rule(self) -> None:
        self.assertEqual(build_rule_0("name").validate({"name": "ok"}), "ok")

    def test_optional_defaults(self) -> None:
        schema = Schema([FieldRule("x", int, required=False, default=4)])
        self.assertEqual(schema.optional_defaults(), {"x": 4})
