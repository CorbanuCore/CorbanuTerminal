from __future__ import annotations

import unittest

from confclerk.template import TemplateRenderer, render_config


class TemplateTests(unittest.TestCase):
    def test_basic_dotted_render(self) -> None:
        self.assertEqual(TemplateRenderer().render("hi {{ user.name }}", {"user": {"name": "Ada"}}), "hi Ada")

    def test_filters(self) -> None:
        self.assertEqual(TemplateRenderer().render("{{ user.name|upper }}", {"user": {"name": "Ada"}}), "ADA")

    def test_includes_are_context_sensitive(self) -> None:
        renderer = TemplateRenderer({"card": "{{ user.name }}"})
        self.assertEqual(renderer.render("{% include card %}", {"user": {"name": "A"}}), "A")
        self.assertEqual(renderer.render("{% include card %}", {"user": {"name": "B"}}), "B")

    def test_render_config(self) -> None:
        out = render_config({"app": {"name": "clerk"}}, {"title": "{{ app.name|upper }}"})
        self.assertEqual(out["title"], "CLERK")

    def test_missing_value_raises(self) -> None:
        with self.assertRaises(Exception):
            TemplateRenderer().render("{{ missing }}", {})
